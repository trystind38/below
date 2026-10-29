use bevy::prelude::*;

use crate::{
    GameState, LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_RIGHT, TILE_SIZE,
    camera::CameraTarget,
    combat::blink_color,
    gun::CursorWorld,
    level::Brick,
    loading::{LoadingAssets, despawn_with},
    sound::{SoundEvent, SoundKind},
    stats::{
        Damage, HIT_COOLDOWN, Health, HitCooldown, Hitbox, MoveSpeed, PLAYER_DAMAGE,
        PLAYER_HEALTH, PLAYER_SPEED,
    },
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

const FACING_DEADZONE: f32 = 4.;

const HURT_TINT: Color = Color::srgb(1., 0.4, 0.4);
const HURT_FLASH_TIME: f32 = 0.15;

const SKIN: f32 = 0.01;

// Seconds between footstep sounds while walking
const STEP_COOLDOWN: f32 = 0.6;
// Minimum fall speed for touching down to count as a landing sound
const LAND_SOUND_SPEED: f32 = 200.;

const LOW_HEALTH: f32 = 0.10;           
const LOW_HEALTH_BLINK_RATE: f32 = 4.;  
const LOW_HEALTH_TINT: Color = Color::srgb(1., 0.2, 0.2);

#[derive(Component, Default)]
pub struct Player {
    pub vel: Vec2,
    pub grounded: bool,
    pub can_fastfall: bool,
    pub facing: f32, // 1 = right, -1 = left
    pub stun: f32,   // seconds of knockback left
    jump_buffer: f32,
    step_cooldown: f32,
}

// Run after this set to see the player's pose for the current frame
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlayerSystems;

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
                (move_player, face_cursor, animate_player, damage_player, tint_player)
                    .chain()
                    .in_set(PlayerSystems)
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
    // The gun is drawn separately in gun.rs
    let walk: Handle<Image> = asset_server.load("player/player_body_walk.png");
    let jump: Handle<Image> = asset_server.load("player/player_body_jump.png");
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
        Player {
            facing: 1.,
            ..default()
        },
        AnimationTimer(Timer::from_seconds(FRAME_TIME, TimerMode::Repeating)),
        Health::new(PLAYER_HEALTH),
        Damage(PLAYER_DAMAGE),
        Hitbox(PLAYER_SIZE),
        HitCooldown::default(),
        MoveSpeed(PLAYER_SPEED),
        CameraTarget,
    ));
}

fn move_player(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    player: Single<(&mut Transform, &mut Player, &MoveSpeed)>,
    bricks: Query<&Transform, (With<Brick>, Without<Player>)>,
    mut sounds: MessageWriter<SoundEvent>,
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
    if player.stun > 0. {
        player.stun -= dt;
    } else if player.grounded {
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
    if !player.grounded && player.can_fastfall && keys.any_pressed([KeyCode::KeyS, KeyCode::ArrowDown]) {
        player.vel.y -= FAST_FALL_KICK;
        player.can_fastfall = false;
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

    let was_grounded = player.grounded;
    let fall_speed = -player.vel.y;
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
    if player.grounded == true {
        player.can_fastfall = true;
    }

    let feet = tf.translation.truncate() - Vec2::Y * PLAYER_SIZE.y / 2.;
    if !was_grounded && player.grounded && fall_speed > LAND_SOUND_SPEED {
        sounds.write(SoundEvent::new(feet, SoundKind::Land));
    }
    // First step sounds right away, then one per cooldown while walking
    player.step_cooldown = (player.step_cooldown - dt).max(0.);
    if player.grounded && player.vel.x != 0. && player.step_cooldown == 0. {
        sounds.write(SoundEvent::new(feet, SoundKind::Step));
        player.step_cooldown = STEP_COOLDOWN;
    }
    // Keep the player inside the level
    let half_w = PLAYER_SIZE.x / 2.;
    tf.translation.x = tf
        .translation
        .x
        .clamp(LEVEL_LEFT + half_w, LEVEL_RIGHT - half_w);
}

fn face_cursor(
    cursor: Res<CursorWorld>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    player: Single<(&Transform, &mut Player)>,
) {
    let (tf, mut player) = player.into_inner();

    if mouse.pressed(MouseButton::Left) {
        if let Some(cursor) = cursor.0 {
            let dx = cursor.x - tf.translation.x;
            if dx.abs() > FACING_DEADZONE {
                player.facing = dx.signum();
            }
        }
        return;
    }

    let mut dir = 0.;
    if keys.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]) {
        dir -= 1.;
    }
    if keys.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]) {
        dir += 1.;
    }
    if dir != 0. {
        player.facing = dir;
    }
}

fn animate_player(
    time: Res<Time>,
    sheet: Res<PlayerSheet>,
    player: Single<(&Player, &mut Sprite, &mut AnimationTimer)>,
) {
    let (player, mut sprite, mut timer) = player.into_inner();

    sprite.flip_x = player.facing < 0.;

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
            // Play the walk cycle backwards when moving away from the cursor
            let step = if player.vel.x * player.facing < 0. { WALK_FRAMES - 1 } else { 1 };
            atlas.index = (atlas.index + step) % WALK_FRAMES;
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

fn damage_player(
    health: Single<(&mut Health, &Player)>,
    keys: Res<ButtonInput<KeyCode>>
) {
    let (mut health, _player) = health.into_inner();
    if keys.any_just_pressed([KeyCode::KeyJ]) {
        health.current = (health.current - 20.0).max(0.0);
    }
    if keys.any_just_pressed([KeyCode::KeyK]) {
        health.current = health.max;
    }
    if keys.any_just_pressed([KeyCode::KeyL]) {
        health.max = (health.max + 20.0).min(200.0);
    }
}

fn tint_player(
    time: Res<Time>,
    player: Single<(&Health, &HitCooldown, &mut Sprite), With<Player>>
) {
    let (health, cooldown, mut sprite) = player.into_inner();
    sprite.color = if health.current <= 0. {
        Color::srgb(1., 0., 0.)
    } else if cooldown.0 > HIT_COOLDOWN - HURT_FLASH_TIME {
        HURT_TINT
    
    } else if health.current / health.max <= LOW_HEALTH
    && (time.elapsed_secs() * LOW_HEALTH_BLINK_RATE) as u32 % 2 == 0
    {
    LOW_HEALTH_TINT
    } else {
        blink_color(cooldown)
    };
}
