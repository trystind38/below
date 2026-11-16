use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    GameState,
    combat::Bullet,
    loading::LoadingAssets,
    player::{Player, PlayerSystems},
    sound::{SoundEvent, SoundKind},
    stats::Damage,
};

const FIRE_COOLDOWN: f32 = 0.2;
const BULLET_SPEED: f32 = 900.;
const BULLET_LIFETIME: f32 = 1.5;
const BULLET_SIZE: Vec2 = Vec2::new(10., 4.);
const BULLET_COLOR: Color = Color::srgb(235. / 255., 211. / 255., 32. / 255.);

// Hand position from the body sprite's center, in sprite pixels
const PIVOT: Vec2 = Vec2::new(3., -2.);
const WALK_BOB: [Vec2; 4] = [
    Vec2::ZERO,
    Vec2::new(1., 1.),
    Vec2::ZERO,
    Vec2::new(-2., 1.),
];
const JUMP_BOB: [Vec2; 2] = [Vec2::new(1., 1.), Vec2::new(-2., 1.)];
// Barrel tip from the hand, pointing right
const MUZZLE: Vec2 = Vec2::new(8., 2.);

#[derive(Resource, Default)]
pub struct CursorWorld(pub Option<Vec2>);

#[derive(Component, Default)]
pub struct Gun {
    angle: f32,
    muzzle: Vec2, // world space
    cooldown: f32,
}

#[derive(Resource)]
struct GunImage(Handle<Image>);

pub struct GunPlugin;
impl Plugin for GunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorld>()
            .add_systems(Startup, load_gun)
            .add_systems(PreUpdate, track_cursor.run_if(in_state(GameState::Playing)))
            .add_systems(
                Update,
                (attach_gun, (aim_gun, fire_gun).chain().after(PlayerSystems))
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn load_gun(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let image: Handle<Image> = asset_server.load("player/player_gun.png");
    loading_assets.0.push(image.clone().untyped());
    commands.insert_resource(GunImage(image));
}

fn track_cursor(
    mut cursor: ResMut<CursorWorld>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let (camera, camera_tf) = camera.into_inner();
    if let Some(screen_pos) = window.cursor_position() {
        if let Ok(world_pos) = camera.viewport_to_world_2d(camera_tf, screen_pos) {
            cursor.0 = Some(world_pos);
        }
    }
}

fn attach_gun(
    mut commands: Commands,
    image: Res<GunImage>,
    new_players: Query<Entity, Added<Player>>,
) {
    for player in &new_players {
        commands.entity(player).with_children(|parent| {
            parent.spawn((
                Sprite::from_image(image.0.clone()),
                Transform::from_translation(PIVOT.extend(0.1)),
                Gun::default(),
            ));
        });
    }
}

fn aim_gun(
    cursor: Res<CursorWorld>,
    player: Single<(&Transform, &Player, &Sprite), Without<Gun>>,
    gun: Single<(&mut Transform, &mut Sprite, &mut Gun), Without<Player>>,
) {
    let (player_tf, player, body) = player.into_inner();
    let (mut gun_tf, mut sprite, mut gun) = gun.into_inner();

    let frame = body.texture_atlas.as_ref().map_or(0, |atlas| atlas.index);
    let bob = if player.grounded {
        WALK_BOB[frame % WALK_BOB.len()]
    } else {
        JUMP_BOB[frame % JUMP_BOB.len()]
    };
    let facing = if player.facing < 0. { -1. } else { 1. };
    let pivot = Vec2::new((PIVOT.x + bob.x) * facing, PIVOT.y + bob.y);
    let pivot_world = player_tf.transform_point(pivot.extend(0.)).truncate();

    // Clamp to the front half so the gun never points back through the player
    let aim = cursor.0.map_or(Vec2::X * facing, |c| c - pivot_world);
    let forward_angle = aim.y.atan2(aim.x * facing).clamp(-FRAC_PI_2, FRAC_PI_2);
    let angle = if facing > 0. {
        forward_angle
    } else {
        PI - forward_angle
    };

    gun_tf.translation = pivot.extend(gun_tf.translation.z);
    gun_tf.rotation = Quat::from_rotation_z(angle);
    // Keeps the gun right side up when facing left
    sprite.flip_y = facing < 0.;
    sprite.color = body.color;

    let muzzle = Vec2::new(MUZZLE.x, MUZZLE.y * facing) * player_tf.scale.truncate();
    gun.angle = angle;
    gun.muzzle = pivot_world + Vec2::from_angle(angle).rotate(muzzle);
}

fn fire_gun(
    mut commands: Commands,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    damage: Single<&Damage, With<Player>>,
    gun: Single<&mut Gun>,
    mut sounds: MessageWriter<SoundEvent>,
) {
    let mut gun = gun.into_inner();
    gun.cooldown = (gun.cooldown - time.delta_secs()).max(0.);
    if !mouse.pressed(MouseButton::Left) || gun.cooldown > 0. {
        return;
    }
    gun.cooldown = FIRE_COOLDOWN;

    commands.spawn((
        Sprite {
            color: BULLET_COLOR,
            custom_size: Some(BULLET_SIZE),
            ..default()
        },
        Transform::from_translation(gun.muzzle.extend(3.))
            .with_rotation(Quat::from_rotation_z(gun.angle)),
        Bullet {
            vel: Vec2::from_angle(gun.angle) * BULLET_SPEED,
            damage: damage.0,
            life: BULLET_LIFETIME,
        },
    ));
    sounds.write(SoundEvent::new(gun.muzzle, SoundKind::Shot));
}
