use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::{
    GameState, LEVEL_BOTTOM, TILE_SIZE,
    level::{ActiveLevel, Brick, intro_only},
    loading::{LoadingAssets, despawn_with},
    proc_gen::enemy_placement::FlytrapFacing,
    player::{Player, PlayerSystems},
    stats::{HIT_COOLDOWN, Health, HitCooldown, Hitbox, Invulnerable, STRAIGHT_FLYTRAP},
};

const FLYTRAP_DRAW_SIZE: Vec2 = Vec2::new(64., 64.);
const FLYTRAP_SIZE: Vec2 = Vec2::new(36., 50.);
const FLYTRAP_BOTTOM_PADDING: f32 = 6.;
const FLYTRAP_X: f32 = 640.;

const FIREBALL_DRAW_SIZE: Vec2 = Vec2::new(24., 24.);
const FIREBALL_SIZE: Vec2 = Vec2::new(14., 14.);
const FIRE_INTERVAL: f32 = 2.;
const FIREBALL_SPEED: f32 = 650.;
pub const FIREBALL_GRAVITY: f32 = 512.;
const FIREBALL_DAMAGE: f32 = 10.;

#[derive(Component)]
pub struct Flytrap {
    pub fire_timer: Timer,
    // Which way the mouth points; fireballs launch this way
    pub dir: Vec2,
}

#[derive(Component)]
pub struct Fireball {
    pub vel: Vec2,
}

#[derive(Resource)]
struct FlytrapImages {
    flytrap: Handle<Image>,
    fireball: Handle<Image>,
}

pub struct FlytrapPlugin;
impl Plugin for FlytrapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_flytrap)
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    spawn_flytrap.run_if(intro_only),
                    spawn_level_flytraps.run_if(not(intro_only)),
                ),
            )
            .add_systems(
                FixedUpdate,
                (flytrap_shoot, move_fireballs)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                fireball_hits_player
                    .after(PlayerSystems)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                OnExit(GameState::Playing),
                (despawn_with::<Flytrap>, despawn_with::<Fireball>),
            );
    }
}

fn load_flytrap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let flytrap: Handle<Image> = asset_server.load("enemy/flytrap.png");
    let fireball: Handle<Image> = asset_server.load("enemy/fireball.png");

    loading_assets.0.push(flytrap.clone().untyped());
    loading_assets.0.push(fireball.clone().untyped());

    commands.insert_resource(FlytrapImages { flytrap, fireball });
}

// Intro level: one Flytrap standing on the floor
fn spawn_flytrap(mut commands: Commands, images: Res<FlytrapImages>) {
    let pos = Vec2::new(
        FLYTRAP_X,
        LEVEL_BOTTOM + TILE_SIZE + FLYTRAP_DRAW_SIZE.y / 2. - FLYTRAP_BOTTOM_PADDING,
    );
    spawn_flytrap_at(&mut commands, &images, pos, Vec2::Y);
}

// Procedural level: hanging from hallway ceilings and stuck to shaft walls
fn spawn_level_flytraps(mut commands: Commands, images: Res<FlytrapImages>, active: Res<ActiveLevel>) {
    let inset = FLYTRAP_DRAW_SIZE.y / 2. - FLYTRAP_BOTTOM_PADDING;

    for spawn in &active.enemies.flytraps {
        let tile = spawn.tile.as_vec2();
        let (pos, dir) = match spawn.facing {
            // Ceiling is the top edge of the tile
            FlytrapFacing::Down => (
                Vec2::new((tile.x + 0.5) * TILE_SIZE, (tile.y + 1.) * TILE_SIZE - inset),
                Vec2::NEG_Y,
            ),
            // Wall is the tile's left edge
            FlytrapFacing::Right => (
                Vec2::new(tile.x * TILE_SIZE + inset, (tile.y + 0.5) * TILE_SIZE),
                Vec2::X,
            ),
            // Wall is the tile's right edge
            FlytrapFacing::Left => (
                Vec2::new((tile.x + 1.) * TILE_SIZE - inset, (tile.y + 0.5) * TILE_SIZE),
                Vec2::NEG_X,
            ),
        };
        spawn_flytrap_at(&mut commands, &images, pos, dir);
    }
}

// `dir` is the way the mouth faces; the sprite is drawn facing up
fn spawn_flytrap_at(commands: &mut Commands, images: &FlytrapImages, pos: Vec2, dir: Vec2) {
    let rotation = Quat::from_rotation_z(dir.to_angle() - FRAC_PI_2);
    let hitbox = if dir.x != 0. {
        Vec2::new(FLYTRAP_SIZE.y, FLYTRAP_SIZE.x)
    } else {
        FLYTRAP_SIZE
    };

    commands.spawn((
        Sprite {
            image: images.flytrap.clone(),
            custom_size: Some(FLYTRAP_DRAW_SIZE),
            ..default()
        },
        Transform::from_translation(pos.extend(2.)).with_rotation(rotation),
        Flytrap {
            fire_timer: Timer::from_seconds(FIRE_INTERVAL, TimerMode::Repeating),
            dir,
        },
        // Can't be shot or stomped, but hurts on contact
        STRAIGHT_FLYTRAP.bundle(),
        Invulnerable,
        Hitbox(hitbox),
        HitCooldown::default(),
    ));
}

fn flytrap_shoot(
    mut commands: Commands,
    time: Res<Time>,
    images: Res<FlytrapImages>,
    mut flytraps: Query<(&Transform, &mut Flytrap)>,
) {
    for (tf, mut flytrap) in &mut flytraps {
        if !flytrap.fire_timer.tick(time.delta()).just_finished() {
            continue;
        }

        // Fire out of his mouth, whichever way it's facing
        let mouth = tf.translation.truncate() + flytrap.dir * FLYTRAP_SIZE.y / 2.;
        commands.spawn((
            Sprite {
                image: images.fireball.clone(),
                custom_size: Some(FIREBALL_DRAW_SIZE),
                ..default()
            },
            Transform::from_translation(mouth.extend(3.)),
            Fireball {
                vel: flytrap.dir * FIREBALL_SPEED,
            },
        ));
    }
}

fn move_fireballs(
    mut commands: Commands,
    time: Res<Time>,
    active: Res<ActiveLevel>,
    mut fireballs: Query<(Entity, &mut Transform, &mut Fireball)>,
    bricks: Query<&Transform, (With<Brick>, Without<Fireball>)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut fireball) in &mut fireballs {
        fireball.vel.y -= FIREBALL_GRAVITY * dt;
        tf.translation += (fireball.vel * dt).extend(0.);

        // Flame trails behind it, so point the sprite along its velocity
        if fireball.vel.length_squared() > 1. {
            tf.rotation = Quat::from_rotation_z(fireball.vel.to_angle() - FRAC_PI_2);
        }

        let pos = tf.translation.truncate();
        let (min, max) = (active.bounds_min, active.bounds_max);
        let left_level = pos.x < min.x || pos.x > max.x || pos.y < min.y || pos.y > max.y;
        let hit_wall = bricks.iter().any(|brick| {
            overlaps(
                pos,
                FIREBALL_SIZE,
                brick.translation.truncate(),
                Vec2::splat(TILE_SIZE),
            )
        });

        if left_level || hit_wall {
            commands.entity(entity).despawn();
        }
    }
}

fn fireball_hits_player(
    mut commands: Commands,
    fireballs: Query<(Entity, &Transform), With<Fireball>>,
    player: Single<(&Transform, &Hitbox, &mut Health, &mut HitCooldown), With<Player>>,
) {
    let (player_tf, player_box, mut health, mut cooldown) = player.into_inner();
    if health.current <= 0. {
        return;
    }

    let player_pos = player_tf.translation.truncate();
    for (entity, tf) in &fireballs {
        if !overlaps(
            tf.translation.truncate(),
            FIREBALL_SIZE,
            player_pos,
            player_box.0,
        ) {
            continue;
        }

        // Fireballs fizzle on the player even during i-frames
        commands.entity(entity).despawn();
        if cooldown.0 <= 0. {
            health.current = (health.current - FIREBALL_DAMAGE).max(0.);
            cooldown.0 = HIT_COOLDOWN;
        }
    }
}

fn overlaps(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let d = (a_pos - b_pos).abs();
    let reach = (a_size + b_size) / 2.;
    d.x < reach.x && d.y < reach.y
}