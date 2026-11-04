use bevy::prelude::*;
use std::f32::consts::TAU;

use crate::{GameState, loading::despawn_with, player::Player, stats::Frozen};

const BURST_KEY: KeyCode = KeyCode::KeyF;

// Rays of light shooting out evenly in every direction
const RAY_COUNT: u32 = 48; // keep this even so long and short rays alternate cleanly
const RAY_TIME: f32 = 1.0; // seconds for a ray to shoot out and vanish
const RAY_LENGTH: f32 = 1500.; // length of the long rays
const RAY_SHORT: f32 = 0.6; // every other ray is this fraction as long (and as wide)
const RAY_WIDTH: f32 = 60.; // width of the long rays at the base (shrinks to nothing)

// Bright ball of light at the center
const CORE_RADIUS: f32 = 160.;
const CORE_TIME: f32 = 0.8;

// Screen going white. The player is frozen until this has fully faded out.
const WHITE_DELAY: f32 = 0.5; // seconds before the screen starts going white
const WHITE_RAMP: f32 = 0.25; // seconds to reach full white
const WHITE_HOLD: f32 = 0.5; // seconds fully white
const WHITE_FADE: f32 = 0.5; // seconds to fade back to the game

#[derive(Component)]
struct LightRay {
    age: f32,
    max_len: f32,
    max_width: f32,
}

#[derive(Component)]
struct BurstCore {
    age: f32,
}

#[derive(Component)]
struct Whiteout {
    age: f32,
}

fn spawn_burst(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    player: Single<(Entity, &Transform, Has<Frozen>), With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (player_entity, player_tf, frozen) = player.into_inner();

    // can't start a new burst while one is already playing
    if !keys.just_pressed(BURST_KEY) || frozen {
        return;
    }

    // freeze the player until the light is gone
    commands.entity(player_entity).insert(Frozen);

    let origin = player_tf.translation.with_z(50.);
    let white = materials.add(Color::WHITE);

    // one thin triangle: wide base at the origin, tip at x = 1
    let ray_mesh = meshes.add(Triangle2d::new(
        Vec2::new(0., -0.5),
        Vec2::new(1., 0.),
        Vec2::new(0., 0.5),
    ));

    for i in 0..RAY_COUNT {
        // perfectly even spacing around the circle
        let angle = i as f32 / RAY_COUNT as f32 * TAU;

        // long and short rays alternate, so the pattern is symmetrical
        let size = if i % 2 == 0 { 1. } else { RAY_SHORT };

        commands.spawn((
            Mesh2d(ray_mesh.clone()),
            MeshMaterial2d(white.clone()),
            Transform::from_translation(origin)
                .with_rotation(Quat::from_rotation_z(angle))
                .with_scale(Vec3::ZERO),
            LightRay {
                age: 0.,
                max_len: RAY_LENGTH * size,
                max_width: RAY_WIDTH * size,
            },
        ));
    }

    commands.spawn((
        Mesh2d(meshes.add(Circle::new(1.).mesh().resolution(64))),
        MeshMaterial2d(white),
        Transform::from_translation(origin).with_scale(Vec3::ZERO),
        BurstCore { age: 0. },
    ));

    // full-screen white layer, starts invisible
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        BackgroundColor(Color::srgba(1., 1., 1., 0.)),
        GlobalZIndex(1000),
        Whiteout { age: 0. },
    ));
}

fn update_rays(
    mut commands: Commands,
    time: Res<Time>,
    mut rays: Query<(Entity, &mut LightRay, &mut Transform)>,
) {
    for (e, mut ray, mut tf) in &mut rays {
        ray.age += time.delta_secs();
        let t = (ray.age / RAY_TIME).min(1.);

        // moves out at a steady pace, easing off near the end; gets thinner as it goes
        let out = 1. - (1. - t).powi(2);
        tf.scale = Vec3::new(ray.max_len * out, ray.max_width * (1. - t), 1.);

        if t >= 1. {
            commands.entity(e).despawn();
        }
    }
}

fn update_core(
    mut commands: Commands,
    time: Res<Time>,
    mut cores: Query<(Entity, &mut BurstCore, &mut Transform)>,
) {
    for (e, mut core, mut tf) in &mut cores {
        core.age += time.delta_secs();
        let t = (core.age / CORE_TIME).min(1.);

        // swells outward, then shrinks away
        let grow = 1. - (1. - t).powi(2);
        tf.scale = Vec3::splat(CORE_RADIUS * grow * (1. - t));

        if t >= 1. {
            commands.entity(e).despawn();
        }
    }
}

fn update_whiteout(
    mut commands: Commands,
    time: Res<Time>,
    players: Query<Entity, With<Player>>,
    mut layers: Query<(Entity, &mut Whiteout, &mut BackgroundColor)>,
) {
    for (e, mut layer, mut bg) in &mut layers {
        layer.age += time.delta_secs();
        let t = layer.age - WHITE_DELAY;

        let alpha = if t < 0. {
            0.
        } else if t < WHITE_RAMP {
            t / WHITE_RAMP
        } else if t < WHITE_RAMP + WHITE_HOLD {
            1.
        } else {
            1. - (t - WHITE_RAMP - WHITE_HOLD) / WHITE_FADE
        };
        bg.0 = Color::srgba(1., 1., 1., alpha.clamp(0., 1.));

        // the light is gone: let the player move again
        if t >= WHITE_RAMP + WHITE_HOLD + WHITE_FADE {
            commands.entity(e).despawn();
            for player in &players {
                commands.entity(player).remove::<Frozen>();
            }
        }
    }
}

pub struct LightBurstPlugin;
impl Plugin for LightBurstPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_burst, update_rays, update_core, update_whiteout)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            OnExit(GameState::Playing),
            (
                despawn_with::<LightRay>,
                despawn_with::<BurstCore>,
                despawn_with::<Whiteout>,
            ),
        );
    }
}