use bevy::prelude::*;

use crate::{
    GameState, LEVEL_H, LEVEL_LEN, TILE_SIZE, WIN_H, WIN_W,
    loading::{LoadingAssets, despawn_with},
};

#[derive(Component)]
pub struct Brick;

#[derive(Component)]
pub struct Background;

#[derive(Resource)]
pub struct BackgroundImage(Handle<Image>);
#[derive(Resource)]
pub struct BrickImage(Handle<Image>);

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_level)
            .add_systems(OnEnter(GameState::Playing), setup_level)
            .add_systems(
                OnExit(GameState::Playing),
                (despawn_with::<Brick>, despawn_with::<Background>),
            );
    }
}

fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let bg_texture_handle = asset_server.load("backgrounds/noisey_background1.png");

    loading_assets.0.push(bg_texture_handle.clone().untyped());
    commands.insert_resource(BackgroundImage(bg_texture_handle));

    let brick_image_handle: Handle<Image> = asset_server.load("tiles/rocky_stone_cold_16x16.png");
    loading_assets.0.push(brick_image_handle.clone().untyped());

    commands.insert_resource(BrickImage(brick_image_handle));
}

fn setup_level(
    mut commands: Commands,
    background_image: Res<BackgroundImage>,
    brick_image: Res<BrickImage>,
) {
    let mut y_offset = 0.;
    while y_offset < LEVEL_H {
        let mut x_offset = 0.;
        while x_offset < LEVEL_LEN {
            commands.spawn((
                Sprite::from_image(background_image.0.clone()),
                Transform::from_xyz(x_offset, y_offset, 0.),
                Background,
            ));

            x_offset += WIN_W;
        }

        y_offset += WIN_H;
    }

    // Spawn floor
    let mut i = 0;
    let mut t = Vec3::new(
        -WIN_W / 2. + TILE_SIZE / 2.,
        -WIN_H / 2. + TILE_SIZE / 2.,
        1.,
    );
    while (i as f32) * TILE_SIZE < LEVEL_LEN {
        commands
            .spawn((
                Sprite {
                    image: brick_image.0.clone(),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform {
                    translation: t,
                    ..default()
                },
                Brick,
            ))
            .insert(Brick);

        i += 1;
        t += Vec3::new(TILE_SIZE, 0., 0.);
    }

    // Spawn some obstacle tiles

    // Tiles are 32 units in world/game space, despite being 16 pixel units in resolution

    // Left tower to player 2 blocks tall part 1
    t = Vec3::new(-160., (-WIN_H / 2.) + TILE_SIZE + 16., 1.);
    spawn_tile(&mut commands, &brick_image, t);
    t = Vec3::new(-160., (-WIN_H / 2.) + TILE_SIZE + 48., 1.);
    spawn_tile(&mut commands, &brick_image, t);
    // Left tower part 2
    t += Vec3::new(32., -32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    t += Vec3::new(0., 32., 0.);
    spawn_tile(&mut commands, &brick_image, t);

    // uh quick reset for t
    t = Vec3::new(48., (-WIN_H / 2.) + TILE_SIZE + 16., 1.);

    // Right tower 2 blocks tall part 1
    t += Vec3::new(256., 0., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    t += Vec3::new(0., 32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    // Right tower part 2
    t += Vec3::new(32., -32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    t += Vec3::new(0., 32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    // Right tower part 3
    t += Vec3::new(32., -32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    t += Vec3::new(0., 32., 0.);
    spawn_tile(&mut commands, &brick_image, t);

}

// Adapted from CS1666_examples
pub fn spawn_tile(
    commands: &mut Commands,
    brick_image: &Res<BrickImage>,
    translation: Vec3,
) {
    commands.spawn((
        Sprite {
            image: brick_image.0.clone(),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform {
            translation: translation,
            ..default()
        },
        Brick,
    ))
    .insert(Brick);
}
