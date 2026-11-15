use bevy::prelude::*;
use rand::Rng;

use crate::{
    GameState, LEVEL_BOTTOM, TILE_SIZE,
    loading::{LoadingAssets, despawn_with},
    stats::{Hitbox, Health, HitCooldown, Enemy},
    hivemind::{spawn_hivemind_enemy, HivemindSheet, HivemindState,},
    level::Brick,
    player::Player,
};

pub const HIVE_SCALE: f32 = 2.;
pub const HIVE_SIZE: Vec2 = Vec2::new(32., 32.);

const FLOWER_RADIUS: f32 = TILE_SIZE * 25.;
const FLOWER_SPACING: f32 = TILE_SIZE * 2.;

#[derive(Component)]
pub struct Hive;

#[derive(Resource)]
struct HiveAssets {
    image: Handle<Image>,
    flower_image: Handle<Image>,
}

pub struct HivemindControllerPlugin;

#[derive(Component)]
pub struct HiveFlower {
    pub rustle_timer: f32,
    pub cooldown_remaining: f32,
    pub reserved_by: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct HiveSpawnQueue(pub Vec<Entity>);

impl Plugin for HivemindControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_hive)
            .add_systems(OnEnter(GameState::Playing), spawn_hive)
            .add_systems(
                Update,(   
                    spawn_hive_replacements,
                    populate_hive_flowers,
                    rustle_flowers,
                    update_flower_cooldowns,
                ).run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (
                    despawn_with::<Hive>,
                    despawn_with::<HiveFlower>,
                ),
            )
            .init_resource::<HiveSpawnQueue>();
    }
}

fn load_hive(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let image = asset_server.load("enemy/hive.png");
    let flower_image = asset_server.load("enemy/HivemindFlower.png");

    loading_assets.0.push(image.clone().untyped());
    loading_assets.0.push(flower_image.clone().untyped());

    commands.insert_resource(HiveAssets {
        image,
        flower_image,
    });
}

fn spawn_hive(
    mut commands: Commands,
    assets: Res<HiveAssets>,
    sheet: Res<HivemindSheet>,
) {
    let hive_position = Vec2::new(
        1250.,
        LEVEL_BOTTOM + TILE_SIZE + HIVE_SIZE.y,
    );

    let hive_entity = commands.spawn((
        Sprite::from_image(assets.image.clone()),
        Transform::from_xyz(
            hive_position.x,
            hive_position.y,
            1.,
        ).with_scale(Vec3::splat(HIVE_SCALE)),
        Hive,
        Enemy,
        Health {
            current: 100.,
            max: 100.,
        },
        HitCooldown::default(),
        Hitbox(HIVE_SIZE),
    )).id();

    for i in 0..12 {
        let spawn_position = hive_position + Vec2::new((i as f32 - 5.5) * 36., HIVE_SIZE.y);

        spawn_hivemind_enemy(
            &mut commands,
            &sheet,
            spawn_position,
            hive_position,
            hive_entity,
        );
    }
}

fn populate_hive_flowers(
    mut commands: Commands,
    hives: Query<&Transform, Added<Hive>>,
    bricks: Query<&Transform, With<Brick>>,
    assets: Res<HiveAssets>,
) {
    let mut rng = rand::thread_rng();

    for hive_tf in &hives {
        let hive_position = hive_tf.translation.truncate();

        let flower_count = rng.gen_range(12..=18);

        let mut spawned_positions: Vec<f32> = Vec::new();

        //tries random positions around the hive
        for _ in 0..flower_count * 20 {
            if spawned_positions.len() >= flower_count {
                break;
            }

            let sample_x = hive_position.x + rng.gen_range(-FLOWER_RADIUS..=FLOWER_RADIUS);

            //finds the highest suitable brick underneath this position
            let surface = bricks
                .iter()
                .filter(|brick_tf| {
                    (brick_tf.translation.x - sample_x).abs() <= TILE_SIZE / 2. && brick_tf.translation.y + TILE_SIZE / 2. <= hive_position.y + TILE_SIZE * 4.
                })
                .max_by(|a, b| {
                    a.translation.y.total_cmp(&b.translation.y)
                });

            let Some(surface) = surface else {
                continue;
            };

            let flower_x = surface.translation.x;

            //avoids placing flowers too close together
            if spawned_positions.iter().any(|x| {
                (x - flower_x).abs() < FLOWER_SPACING
            }) {
                continue;
            }

            let ground_top = surface.translation.y + TILE_SIZE / 2.;

            let flower_position = Vec3::new(
                flower_x,
                ground_top + TILE_SIZE / 2.,
                1.5,
            );

            commands.spawn((
                Sprite {
                    image: assets.flower_image.clone(),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_translation(flower_position),
                HiveFlower {
                    rustle_timer: 0.,
                    cooldown_remaining: 0.,
                    reserved_by: None,
                },
            ));

            spawned_positions.push(flower_x);
        }
    }
}

fn rustle_flowers(
    time: Res<Time>,
    player: Single<(&Transform, &Hitbox, &Player), With<Player>>,
    mut flowers: Query<(&Transform, &mut Sprite, &mut HiveFlower)>,
) {
    const RUSTLE_INTERVAL: f32 = 0.12;

    let (player_tf, player_hitbox, player) = *player;
    let player_pos = player_tf.translation.truncate();
    let is_moving = player.vel.length_squared() > 0.0;

    for (flower_tf, mut sprite, mut flower) in &mut flowers {
        let flower_pos = flower_tf.translation.truncate();
        let flower_size = Vec2::splat(TILE_SIZE);
        let distance = (player_pos - flower_pos).abs();
        let reach = (player_hitbox.0 + flower_size) / 2.;
        let touching = distance.x < reach.x && distance.y < reach.y;

        if touching && is_moving{
            flower.rustle_timer += time.delta_secs();

            if flower.rustle_timer >= RUSTLE_INTERVAL {
                flower.rustle_timer = 0.;
                sprite.flip_x = !sprite.flip_x;
            }
        } else {
            flower.rustle_timer = 0.;
            sprite.flip_x = false;
        }
    }
}

//cooldown so bugs don't keep taking from the same flowers over and over again
fn update_flower_cooldowns(
    time: Res<Time>,
    mut flowers: Query<&mut HiveFlower>,
) {
    for mut flower in &mut flowers {
        if flower.cooldown_remaining > 0. {
            flower.cooldown_remaining = (flower.cooldown_remaining - time.delta_secs()).max(0.);
        }
    }
}

fn spawn_hive_replacements(
    mut commands: Commands,
    mut spawn_queue: ResMut<HiveSpawnQueue>,
    hives: Query<(&Transform, &Health), With<Hive>>,
    sheet: Res<HivemindSheet>,
) {
    for hive_entity in spawn_queue.0.drain(..) {
        if let Ok((hive_tf, health)) = hives.get(hive_entity) {
            if health.current <= 0. {
                continue;
            }

            let hive_position = hive_tf.translation.truncate();
            let spawn_position =
                hive_position + Vec2::new(0., HIVE_SIZE.y);

            let new_bug = spawn_hivemind_enemy(
                &mut commands,
                &sheet,
                spawn_position,
                hive_position,
                hive_entity,
            );

            commands.entity(new_bug).insert(HivemindState::Chase);
        }
    }
}