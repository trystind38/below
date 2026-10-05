use bevy::prelude::*;

use crate::{
    GameState, LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_RIGHT, LEVEL_TOP, TILE_SIZE,
    camera::CameraTarget, loading::despawn_with,
};

// PLACEHOLDER player: a 32x32 square that walks and jumps so the camera can be seen working.
// To replace it, swap the sprite and movement for the real ones, and keep the `CameraTarget`
// component on the player so the camera keeps following it.
const PLAYER_SIZE: f32 = 32.;
const PLAYER_SPEED: f32 = 500.;
const JUMP_SPEED: f32 = 700.;
const GRAVITY: f32 = 1800.;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Velocity(Vec2);

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, move_player.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), despawn_with::<Player>);
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Sprite::from_color(Color::srgb(0.9, 0.2, 0.2), Vec2::splat(PLAYER_SIZE)),
        // Standing on the floor at the center of the first screen
        Transform::from_xyz(0., LEVEL_BOTTOM + TILE_SIZE + PLAYER_SIZE / 2., 2.),
        Player,
        Velocity::default(),
        CameraTarget,
    ));
}

fn move_player(
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player: Single<(&mut Transform, &mut Velocity), With<Player>>,
) {
    let (mut transform, mut velocity) = player.into_inner();
    let dt = time.delta_secs();

    let half = PLAYER_SIZE / 2.;
    let floor = LEVEL_BOTTOM + TILE_SIZE + half;
    let ceiling = LEVEL_TOP - half;
    let grounded = transform.translation.y <= floor;

    let mut dir = 0.;
    if input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        dir -= 1.;
    }
    if input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        dir += 1.;
    }
    velocity.0.x = dir * PLAYER_SPEED;

    // Single jump: only from the floor
    if grounded && input.any_just_pressed([KeyCode::Space, KeyCode::KeyW, KeyCode::ArrowUp]) {
        velocity.0.y = JUMP_SPEED;
    }
    velocity.0.y -= GRAVITY * dt;

    transform.translation += (velocity.0 * dt).extend(0.);

    // Keep the player inside the level and land on the floor
    transform.translation.x = transform
        .translation
        .x
        .clamp(LEVEL_LEFT + half, LEVEL_RIGHT - half);
    if transform.translation.y <= floor {
        transform.translation.y = floor;
        velocity.0.y = 0.;
    } else if transform.translation.y >= ceiling {
        transform.translation.y = ceiling;
        velocity.0.y = velocity.0.y.min(0.);
    }
}
