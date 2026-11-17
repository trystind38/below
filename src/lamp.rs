use bevy::prelude::*;

use crate::{
    GameState, TILE_SIZE,
    combat::overlaps,
    level::ActiveLevel,
    loading::{LoadingAssets, despawn_with},
    player::{PLAYER_SIZE, Player},
    stats::{Health, Hitbox},
};

const SPRITE_SCALE: f32 = 2.;
const LAMP_SIZE: Vec2 = Vec2::new(32., 32.);

#[derive(Component)]
pub struct Lamp;

#[derive(Resource)]
struct LampSheet {
    sprite_sheet: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

pub struct LampPlugin;
impl Plugin for LampPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_lamp)
            .add_systems(OnEnter(GameState::Playing), spawn_lamps)
            .add_systems(Update, touch_lamp.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), despawn_with::<Lamp>);
    }
}

fn load_lamp(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let sprite_sheet: Handle<Image> = asset_server.load("tiles/rocky_stone_boss_16x16.png"); 
    let layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::new(16, 16), 1, 1, None, None));
    loading_assets.0.push(sprite_sheet.clone().untyped());
    commands.insert_resource(LampSheet { sprite_sheet, layout });
}

fn spawn_lamp_at(
    commands: &mut Commands, 
    sheet: &LampSheet, 
    pos: Vec2) 
{
    commands.spawn((
        Sprite::from_atlas_image(
            sheet.sprite_sheet.clone(),
            TextureAtlas { layout: sheet.layout.clone(), index: 0 },
        ),
        Transform::from_translation(pos.extend(1.)).with_scale(Vec3::splat(SPRITE_SCALE)),
        Lamp,
        Hitbox(LAMP_SIZE),
    ));
}

fn touch_lamp(
    player: Single<(&Transform, &mut Player, &mut Health, &Hitbox)>,
    lamps: Query<(&Transform, &Hitbox), With<Lamp>>,
) {
    let (player_tf, mut player, mut health, player_box) = player.into_inner();
    let player_pos = player_tf.translation.truncate();

    for (lamp_tf, lamp_box) in &lamps {
        let lamp_pos = lamp_tf.translation.truncate();
        if overlaps(player_pos, player_box.0, lamp_pos, lamp_box.0) {
            health.current = health.max;
            player.last_checkpoint = lamp_pos;
        }
    }
}

fn spawn_lamps(
    mut commands: Commands, 
    sheet: Res<LampSheet>, 
    active: Res<ActiveLevel>) 
{
    for &pos in &active.lamps {
        spawn_lamp_at(&mut commands, &sheet, pos);
    }
}