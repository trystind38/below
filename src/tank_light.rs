use bevy::prelude::*;
 
use crate::{
    GameState,
    loading::LoadingAssets,
    player::{Player, PlayerSystems},
    stats::Health,
};
 
const LEVELS: usize = 13; // frames in the sheet: 0 = full, 12 = empty
const DRAIN_RATE: f32 = 0.5; // fraction of the tank per second
 
#[derive(Component)]
struct TankLight {
    shown: f32, // 0..1, how full the tank looks right now
}
 
#[derive(Resource)]
struct TankSheet {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}
 
pub struct TankLightPlugin;
impl Plugin for TankLightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_tank).add_systems(
            Update,
            (attach_tank_light, update_tank_light)
                .chain()
                .after(PlayerSystems)
                .run_if(in_state(GameState::Playing)),
        );
    }
}
 
fn load_tank(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let image: Handle<Image> = asset_server.load("player/player_tank_levels.png");
    loading_assets.0.push(image.clone().untyped()); // loading screen waits for it
 
    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(32), // each frame is 32x32
        LEVELS as u32,    // 13 frames across
        1,                // 1 row
        None,
        None,
    ));
 
    commands.insert_resource(TankSheet { image, layout });
}
 
// Runs once when the player appears: adds the tank overlay as a child sprite
fn attach_tank_light(
    mut commands: Commands,
    sheet: Res<TankSheet>,
    players: Query<Entity, Added<Player>>,
) {
    for e in &players {
        commands.entity(e).with_children(|p| {
            p.spawn((
                Sprite::from_atlas_image(
                    sheet.image.clone(),
                    TextureAtlas {
                        layout: sheet.layout.clone(),
                        index: 0,
                    },
                ),
                Transform::from_xyz(0., 0., 0.1), // same spot as the player, drawn on top
                TankLight { shown: 1. },
            ));
        });
    }
}
 
fn update_tank_light(
    time: Res<Time>,
    player: Single<(&Health, &Player, &Sprite), Without<TankLight>>,
    mut tanks: Query<(&mut TankLight, &mut Sprite, &mut Transform), With<TankLight>>,
) {
    let (health, player, player_sprite) = player.into_inner();
    let target = health.current / health.max;
    let step = DRAIN_RATE * time.delta_secs();
 
    // the body's tank sits 1px higher in walk frames 1 and 3, and in both jump frames
    let walk_frame = player_sprite.texture_atlas.as_ref().map_or(0, |a| a.index);
    let bob = if !player.grounded || walk_frame == 1 || walk_frame == 3 {
        1.
    } else {
        0.
    };
 
    for (mut tank, mut sprite, mut tf) in &mut tanks {
        // ease toward the real health instead of snapping
        tank.shown += (target - tank.shown).clamp(-step, step);
 
        // pick the frame: full = 0, empty = LEVELS - 1
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = ((1. - tank.shown) * (LEVELS - 1) as f32).round() as usize;
        }
 
        // the child doesn't flip on its own when the player turns
        sprite.flip_x = player.facing < 0.;
        tf.translation.y = bob;
    }
}