use bevy::prelude::*;

use crate::{
    GameState, LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_RIGHT, TILE_SIZE,
    camera::CameraTarget,
    level::Brick,
    loading::{LoadingAssets, despawn_with},
    stats::{Health, MoveSpeed, PLAYER_HEALTH, PLAYER_SPEED},
};

const GRAVITY: f32 = 2000.;
const JUMP_SPEED: f32 = 800.;
const MAX_FALL_SPEED: f32 = 1200.;
const JUMP_BUFFER: f32 = 0.1;
const FAST_FALL_MULT: f32 = 1.3;
const FAST_FALL_KICK: f32 = 150.;
const AIR_ACCEL: f32 = 3000.;

const FRAME_SIZE: u32 = 32;
const WALK_FRAMES: usize = 4;
const JUMP_FRAMES: u32 = 2;
const FRAME_TIME: f32 = 0.1;
const SPRITE_SCALE: f32 = 2.;

const PLAYER_SIZE: Vec2 = Vec2::new(40., 64.);

const SKIN: f32 = 0.01;

#[derive(Component, Default)]
pub struct Player {
    pub vel: Vec2,
    pub grounded: bool,
    jump_buffer: f32,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Resource)]
struct PlayerSheet {
    walk: Handle<Image>,
    walk_layout: Handle<TextureAtlasLayout>,
    jump: Handle<Image>,
    jump_layout: Handle<TextureAtlasLayout>,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_player)
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (move_player, animate_player)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), despawn_with::<Player>);
    }
}

fn load_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let walk: Handle<Image> = asset_server.load("player/player_full_walk.png");
    let jump: Handle<Image> = asset_server.load("player/player_full_jump.png");
    loading_assets.0.push(walk.clone().untyped());
    loading_assets.0.push(jump.clone().untyped());

    let walk_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(FRAME_SIZE),
        WALK_FRAMES as u32,
        1,
        None,
        None,
    ));
    let jump_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(FRAME_SIZE),
        JUMP_FRAMES,
        1,
        None,
        None,
    ));

    commands.insert_resource(PlayerSheet {
        walk,
        walk_layout,
        jump,
        jump_layout,
    });
}

fn spawn_player(mut commands: Commands, sheet: Res<PlayerSheet>) {
    commands.spawn((
        Sprite::from_atlas_image(
            sheet.walk.clone(),
            TextureAtlas {
                layout: sheet.walk_layout.clone(),
                index: 0,
            },
        ),
        // Standing on the floor at the center of the first screen
        Transform::from_xyz(0., LEVEL_BOTTOM + TILE_SIZE + PLAYER_SIZE.y / 2., 2.)
            .with_scale(Vec3::splat(SPRITE_SCALE)),
        Player::default(),
        AnimationTimer(Timer::from_seconds(FRAME_TIME, TimerMode::Repeating)),
        Health::new(PLAYER_HEALTH),
        MoveSpeed(PLAYER_SPEED),
        CameraTarget,
    ));
}

fn move_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut Transform, &mut Player, &MoveSpeed)>,
    bricks: Query<&Transform, (With<Brick>, Without<Player>)>,
) {
    let (mut tf, mut player, speed) = player.into_inner();
    let dt = time.delta_secs().min(1. / 30.);

    let mut dir = 0.;
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        dir -= 1.;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        dir += 1.;
    }
    let target = dir * speed.0;
    if player.grounded {
        player.vel.x = target;
    } else {
        let max_change = AIR_ACCEL * dt;
        player.vel.x += (target - player.vel.x).clamp(-max_change, max_change);
    }

    // Single jump: only from the floor
    if keys.any_just_pressed([KeyCode::Space, KeyCode::KeyW, KeyCode::ArrowUp]) {
        player.jump_buffer = JUMP_BUFFER;
    }
    if player.grounded && player.jump_buffer > 0. {
        player.vel.y = JUMP_SPEED;
        player.jump_buffer = 0.;
    }
    player.jump_buffer -= dt;
    if !player.grounded && keys.any_just_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        player.vel.y -= FAST_FALL_KICK;
    }
    let fall_mult = if !player.grounded && keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        FAST_FALL_MULT
    } else {
        1.
    };
    player.vel.y = (player.vel.y - GRAVITY * fall_mult * dt).max(-MAX_FALL_SPEED * fall_mult);

    tf.translation.x += player.vel.x * dt;
    for brick in &bricks {
        let (pen, push) = penetration(tf.translation, brick.translation);
        if pen.x > SKIN && pen.y > SKIN {
            tf.translation.x += pen.x * push.x;
            player.vel.x = 0.;
        }
    }

    tf.translation.y += player.vel.y * dt;
    player.grounded = false;
    for brick in &bricks {
        let (pen, push) = penetration(tf.translation, brick.translation);
        if pen.x > SKIN && pen.y > SKIN {
            tf.translation.y += pen.y * push.y;
            player.grounded |= push.y > 0.;
            player.vel.y = 0.;
        }
    }

    // Keep the player inside the level
    let half_w = PLAYER_SIZE.x / 2.;
    tf.translation.x = tf
        .translation
        .x
        .clamp(LEVEL_LEFT + half_w, LEVEL_RIGHT - half_w);
}

fn animate_player(
    time: Res<Time>,
    sheet: Res<PlayerSheet>,
    player: Single<(&Player, &mut Sprite, &mut AnimationTimer)>,
) {
    let (player, mut sprite, mut timer) = player.into_inner();

    if player.vel.x != 0. {
        sprite.flip_x = player.vel.x < 0.;
    }

    let (image, layout) = if player.grounded {
        (&sheet.walk, &sheet.walk_layout)
    } else {
        (&sheet.jump, &sheet.jump_layout)
    };
    sprite.image = image.clone();

    let Some(atlas) = &mut sprite.texture_atlas else {
        return;
    };
    atlas.layout = layout.clone();

    if !player.grounded {
        atlas.index = if player.vel.y > 0. { 0 } else { 1 };
        timer.reset();
    } else if player.vel.x != 0. {
        timer.tick(time.delta());
        if timer.just_finished() {
            atlas.index = (atlas.index + 1) % WALK_FRAMES;
        }
    } else {
        atlas.index = 0;
        timer.reset();
    }
}

fn penetration(player: Vec3, tile: Vec3) -> (Vec2, Vec2) {
    let d = (player - tile).truncate();
    let pen = (PLAYER_SIZE + Vec2::splat(TILE_SIZE)) / 2. - d.abs();
    (pen, d.signum())
}