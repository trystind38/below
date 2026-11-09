use bevy::prelude::*;
use rand::Rng;

use crate :: {
    GameState, LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_RIGHT, TILE_SIZE,
    level::Brick,
    loading::{ LoadingAssets, despawn_with },
    stats::{ HIVEMIND, HitCooldown, Hitbox, MoveSpeed, Health, Damage, },
    player::Player,
    hivemind_controller::HIVE_SIZE,
    level::Brick,
};

const SPRITE_SCALE: f32 = 2.;
const HIVEMIND_SIZE: Vec2 = Vec2::new(32., 32.);
const AGGRO_RANGE: f32 = 180.;
const ATTACK_RANGE: f32 = 72.;

const PATROL_RADIUS: f32 = TILE_SIZE * 20.;

const PATROL_SPEED: f32 = 35.;

const CHASE_ACCELERATION: f32 = 270.;
const CHASE_MAX_SPEED: f32 = 350.;

const ATTACK_SPEED: f32 = 400.;
const ATTACK_DURATION: f32 = 0.4;
const WALL_DEFLECTION: f32 = 0.35;

#[derive(Component, Default)]
pub struct Hivemind {
    pub velocity: Vec2,
    pub grounded: bool,
    pub attack_timer: f32,
    pub animation_timer: f32,

    //hive location
    pub hive_origin: Vec2,

    //idle animation
    pub idle_timer: f32,
    pub idle_origin_y: f32,
    pub idle_amplitude: f32,
    pub idle_speed: f32,
    pub has_hit_player: bool,
    pub has_bounced: bool,
}

#[derive(Resource)]
pub struct HivemindSheet {
    sprite_sheet: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

#[derive(Component, Default, PartialEq, Eq)]
pub enum HivemindState {
    Idle,
    #[default]
    Patrol,
    Chase,
    Attack,
}

#[derive(Component)]
pub struct HivemindPatrol {
    pub distance_remaining: f32,
    pub direction: f32,
}

pub struct HivemindPlugin;
impl Plugin for HivemindPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_hivemind)
            .add_systems(OnEnter(GameState::Playing), spawn_hivemind)
            .add_systems(Update, (
                hivemind_ai,
                update_hivemind_sprite,
            ))
            .add_systems(OnExit(GameState::Playing), despawn_with::<Hivemind>);
    }
}

fn load_hivemind(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let sprite_sheet: Handle<Image> = asset_server.load("enemy/hivemind_sprite_sheet.png");

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), //frame dimensions
        2,                  //columns
        1,                  //rows
        None,
        None,
    );
    let layout = texture_atlas_layouts.add(layout);

    loading_assets.0.push(sprite_sheet.clone().untyped());

    commands.insert_resource(HivemindSheet {sprite_sheet, layout});
}

fn spawn_hivemind(mut commands: Commands, sheet: Res<HivemindSheet>) {
    spawn_hivemind_enemy(
        &mut commands,
        &sheet,
        Vec2::new(
            464.,
            LEVEL_BOTTOM + TILE_SIZE + HIVEMIND_SIZE.y / 2.,
        ),
        Vec2::new(
            512.,
            LEVEL_BOTTOM + TILE_SIZE + HIVE_SIZE.y,
        ),
    );
}

pub fn spawn_hivemind_enemy(
    commands: &mut Commands,
    sheet: &HivemindSheet,
    spawn_position: Vec2,
    hive_position: Vec2,
) {
    commands.spawn((
        Sprite::from_atlas_image(
            sheet.sprite_sheet.clone(),
            TextureAtlas {
                layout: sheet.layout.clone(),
                index: 0,
            },
        ),
        Transform::from_xyz(
            spawn_position.x,
            spawn_position.y,
            2.,
        ).with_scale(Vec3::splat(SPRITE_SCALE)),
        Hivemind {
            hive_origin: hive_position,
            idle_origin_y: spawn_position.y,
            idle_amplitude: 4.,
            idle_speed: 3.,
            ..default()
        },
        HIVEMIND.bundle(),
        Hitbox(HIVEMIND_SIZE),
        HitCooldown::default(),
        HivemindPatrol {
            distance_remaining: 0.,
            direction: 1.,
        },
        HivemindState::default(),
    ));
}

fn overlaps(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let d = (a_pos - b_pos).abs();
    let reach = (a_size + b_size) / 2.;
    d.x < reach.x && d.y < reach.y
}

fn resolve_brick_collision(
    tf: &mut Transform,
    velocity: &mut Vec2,
    brick_pos: Vec2,
) {
    let position = tf.translation.truncate();

    let delta = position - brick_pos;

    let overlap = (HIVEMIND_SIZE + Vec2::splat(TILE_SIZE)) / 2. - delta.abs();

    // No collision
    if overlap.x <= 0. || overlap.y <= 0. {
        return;
    }

    if overlap.x < overlap.y {
        let direction = if delta.x >= 0. { 1. } else { -1. };
        tf.translation.x += overlap.x * direction;

        if velocity.x * direction < 0. {
            velocity.x = -velocity.x * WALL_DEFLECTION;
        }
    } else {
        let direction = if delta.y >= 0. { 1. } else { -1. };
        tf.translation.y += overlap.y * direction;

        if velocity.y * direction < 0. {
            velocity.y = -velocity.y * WALL_DEFLECTION;
        }
    }
}

fn update_hivemind_sprite(
    time: Res<Time>,
    mut query: Query<(&mut Hivemind, &mut Sprite)>,
) {
    for (mut hivemind, mut sprite) in &mut query {
        hivemind.animation_timer += time.delta_secs();

        if hivemind.animation_timer >= 0.15 {
            hivemind.animation_timer = 0.;

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % 2;
            }
        }
    }
}

fn hivemind_ai(
    time: Res<Time>,
    player: Single<(&Transform, &Hitbox), With<Player>>,
    mut hivemind: Query<
        (
            &mut Transform,
            &mut Hivemind,
            &mut HivemindPatrol,
            &mut HivemindState,
        ),
        Without<Player>,
    >,
    bricks: Query<&Transform, (With<Brick>, Without<Hivemind>)>, // for testing collisions
) {
    let dt = time.delta_secs().min(1. / 30.);
    let (player_tf, player_box) = *player;
    let player_pos = player_tf.translation.truncate();

    for (mut tf, mut hivemind, mut patrol, mut state) in &mut hivemind {
        let position = tf.translation.truncate();

        let to_player = player_pos - position;
        let player_distance = to_player.length();

        let mut should_bob = true;

        match *state {
            //Idle
            HivemindState::Idle => {
                hivemind.velocity = Vec2::ZERO;

                if player_distance <= AGGRO_RANGE {
                    *state = HivemindState::Chase;
                }
            }

            //Patrol
            HivemindState::Patrol => {
                //checks if player is close enough
                if player_distance <= AGGRO_RANGE {
                    *state = HivemindState::Chase;
                    hivemind.velocity = Vec2::ZERO;
                } else {
                    let left_bound = hivemind.hive_origin.x - PATROL_RADIUS;
                    let right_bound = hivemind.hive_origin.x + PATROL_RADIUS;

                    //turns around at patrol boundaries
                    if position.x <= left_bound {
                        patrol.direction = 1.;
                    } else if position.x >= right_bound {
                        patrol.direction = -1.;
                    }

                    hivemind.velocity.x = patrol.direction * PATROL_SPEED;

                    //keeps track of remaining patrol bounds
                    patrol.distance_remaining = if patrol.direction > 0. {
                        right_bound - position.x
                    } else {
                        position.x - left_bound
                    };
                }
            }

            //Chase
            HivemindState::Chase => {
                if player_distance <= ATTACK_RANGE {
                    *state = HivemindState::Attack;

                    hivemind.attack_timer = 0.;
                    hivemind.has_bounced = false;

                    //launchs toward the player's current position
                    hivemind.velocity = to_player.normalize_or_zero() * ATTACK_SPEED;
                } else {
                    //accelerates toward the player
                    let direction = to_player.normalize_or_zero();
                    hivemind.velocity += direction * CHASE_ACCELERATION * dt;
                    hivemind.velocity = hivemind.velocity.clamp_length_max(CHASE_MAX_SPEED);
                }

                //changes bob speed n'at
                hivemind.idle_timer += dt;
                hivemind.idle_amplitude = 6.;
                hivemind.idle_speed = 8.;
            }

            //Attack
            HivemindState::Attack => {
                should_bob = false;

                hivemind.attack_timer += dt;

                if !hivemind.has_hit_player && player_distance <= ATTACK_RANGE {
                    hivemind.velocity = -hivemind.velocity;

                    hivemind.has_hit_player = true;
                }

                //gradually slows down after the attack
                hivemind.velocity *= (-7. * dt).exp();

                //back to business
                if hivemind.attack_timer >= ATTACK_DURATION {
                    *state = HivemindState::Chase;
                }
            }
        }

        //moves the lil guy
        tf.translation.x += hivemind.velocity.x * dt;
        tf.translation.y += hivemind.velocity.y * dt;

        //checks whether we've hit the player
        let new_position = tf.translation.truncate();

        if *state == HivemindState::Attack && !hivemind.has_bounced {
            if overlaps(new_position, HIVEMIND_SIZE, player_pos, player_box.0) {
                hivemind.velocity = -hivemind.velocity;
                hivemind.has_bounced = true;
            }
        }

        //animation handler
        if should_bob {
            if *state != HivemindState::Chase {
                hivemind.idle_amplitude = 4.;
                hivemind.idle_speed = 3.;
            }

            hivemind.idle_timer += dt;
        }

        for brick in &bricks {
            resolve_brick_collision(
                &mut tf,
                &mut hivemind.velocity,
                brick.translation.truncate(),
            );
        }
    }
}