use bevy::prelude::*;

use crate::{
    GameState, LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_RIGHT, LEVEL_TOP, TILE_SIZE,
    level::Brick,
    loading::{ LoadingAssets, despawn_with },
    player::{ Player, PlayerSystems },
    stats::{ HIT_COOLDOWN, Health, HitCooldown, Hitbox, Invulnerable, STRAIGHT_TEETH },
};

// teeth.png is 160x160 with transparent padding, drawn at 64x64 in world space
const TEETH_DRAW_SIZE: Vec2 = Vec2::new(64., 64.);
const TEETH_SIZE: Vec2 = Vec2::new(36., 50.); // Hitbox around the visible pixels
const TEETH_BOTTOM_PADDING: f32 = 6.; // Transparent rows under his base at draw size
const TEETH_X: f32 = 640.; // Past the right tower in the demo level

const FIREBALL_DRAW_SIZE: Vec2 = Vec2::new(24., 24.);
const FIREBALL_SIZE: Vec2 = Vec2::new(14., 14.);
const FIRE_INTERVAL: f32 = 2.; // Seconds between shots
const FIREBALL_SPEED: f32 = 650.; // Initial upward speed
const FIREBALL_GRAVITY: f32 = 512.; // Slows the fireball so it arcs back down
const FIREBALL_DAMAGE: f32 = 10.;

#[derive(Component)]
pub struct Teeth {
    pub fire_timer: Timer,
}

#[derive(Component)]
pub struct Fireball {
    pub vel: Vec2,
}

#[derive(Resource)]
struct TeethImages {
    teeth: Handle<Image>,
    fireball: Handle<Image>,
}

pub struct TeethPlugin;
impl Plugin for TeethPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_teeth)
            .add_systems(OnEnter(GameState::Playing), spawn_teeth)
            .add_systems(
                FixedUpdate,
                (teeth_shoot, move_fireballs)
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
                (despawn_with::<Teeth>, despawn_with::<Fireball>),
            );
    }
}

fn load_teeth(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let teeth: Handle<Image> = asset_server.load("enemy/teeth.png");
    let fireball: Handle<Image> = asset_server.load("enemy/fireball.png");

    loading_assets.0.push(teeth.clone().untyped());
    loading_assets.0.push(fireball.clone().untyped());

    commands.insert_resource(TeethImages { teeth, fireball });
}

fn spawn_teeth(mut commands: Commands, images: Res<TeethImages>) {
    commands.spawn((
        Sprite {
            image: images.teeth.clone(),
            custom_size: Some(TEETH_DRAW_SIZE),
            ..default()
        },
        // Standing on the floor
        Transform::from_xyz(
            TEETH_X,
            LEVEL_BOTTOM + TILE_SIZE + TEETH_DRAW_SIZE.y / 2. - TEETH_BOTTOM_PADDING,
            2.,
        ),
        Teeth {
            fire_timer: Timer::from_seconds(FIRE_INTERVAL, TimerMode::Repeating),
        },
        // Can't be shot or stomped, but hurts on contact
        STRAIGHT_TEETH.bundle(),
        Invulnerable,
        Hitbox(TEETH_SIZE),
        HitCooldown::default(),
    ));
}

fn teeth_shoot(
    mut commands: Commands,
    time: Res<Time>,
    images: Res<TeethImages>,
    mut teeth: Query<(&Transform, &mut Teeth)>,
) {
    for (tf, mut teeth) in &mut teeth {
        if !teeth.fire_timer.tick(time.delta()).just_finished() {
            continue;
        }

        // Fire out of his mouth at the top of the sprite
        let mouth = tf.translation.truncate() + Vec2::new(0., TEETH_SIZE.y / 2.);
        commands.spawn((
            Sprite {
                image: images.fireball.clone(),
                custom_size: Some(FIREBALL_DRAW_SIZE),
                ..default()
            },
            Transform::from_translation(mouth.extend(3.)),
            Fireball {
                vel: Vec2::new(0., FIREBALL_SPEED),
            },
        ));
    }
}

fn move_fireballs(
    mut commands: Commands,
    time: Res<Time>,
    mut fireballs: Query<(Entity, &mut Transform, &mut Sprite, &mut Fireball)>,
    bricks: Query<&Transform, (With<Brick>, Without<Fireball>)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut fireball) in &mut fireballs {
        fireball.vel.y -= FIREBALL_GRAVITY * dt;
        tf.translation += (fireball.vel * dt).extend(0.);

        // Flame trails behind it, so flip once it starts falling
        sprite.flip_y = fireball.vel.y < 0.;

        let pos = tf.translation.truncate();
        let left_level = pos.x < LEVEL_LEFT
            || pos.x > LEVEL_RIGHT
            || pos.y < LEVEL_BOTTOM
            || pos.y > LEVEL_TOP;
        let hit_wall = bricks.iter().any(|brick| {
            overlaps(pos, FIREBALL_SIZE, brick.translation.truncate(), Vec2::splat(TILE_SIZE))
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
        if !overlaps(tf.translation.truncate(), FIREBALL_SIZE, player_pos, player_box.0) {
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
