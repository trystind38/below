use bevy::prelude::*;

use crate::{
    GameState, LEVEL_LEN, TILE_SIZE, WIN_H, WIN_W,
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
    let bg_texture_handle = asset_server.load("JakeBiondolillo.png");

    loading_assets.0.push(bg_texture_handle.clone().untyped());
    commands.insert_resource(BackgroundImage(bg_texture_handle));

    let brick_image_handle: Handle<Image> = asset_server.load("basic_stone_sprite.png");
    loading_assets.0.push(brick_image_handle.clone().untyped());

    commands.insert_resource(BrickImage(brick_image_handle));
}

fn setup_level(
    mut commands: Commands,
    background_image: Res<BackgroundImage>,
    brick_image: Res<BrickImage>,
) {
    let mut x_offset = 0.;
    while x_offset < LEVEL_LEN {
        commands.spawn((
            Sprite::from_image(background_image.0.clone()),
            Transform::from_xyz(x_offset, 0., 0.),
            Background,
        ));

        x_offset += WIN_W;
    }

    let mut i = 0;
    let mut t = Vec3::new(
        -WIN_W / 2. + TILE_SIZE / 2.,
        -WIN_H / 2. + TILE_SIZE / 2.,
        1.,
    );
    while (i as f32) * TILE_SIZE < LEVEL_LEN {
        commands
            .spawn((
                Sprite::from_image(brick_image.0.clone()),
                Transform {
                    translation: t,
                    scale: Vec3::splat(2.0),
                    ..default()
                },
                Brick,
            ))
            .insert(Brick);

        i += 1;
        t += Vec3::new(TILE_SIZE, 0., 0.);
    }
}