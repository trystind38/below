use bevy::prelude::*;

use crate::{
    GameState,
    combat::overlaps,
    level::ActiveLevel,
    loading::{LoadingAssets, despawn_with},
    player::Player,
    stats::{Health, Hitbox},
};

const SPRITE_SCALE: f32 = 2.;
const ORB_SIZE: Vec2 = Vec2::new(32., 32.);
const HEAL_FRACTION: f32 = 0.05;

#[derive(Component)]
pub struct Orb;

#[derive(Resource)]
struct OrbImage(Handle<Image>);

pub struct OrbPlugin;
impl Plugin for OrbPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_orb)
            .add_systems(OnEnter(GameState::Playing), spawn_orbs)
            .add_systems(Update, collect_orbs.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), despawn_with::<Orb>);
    }
}

fn load_orb(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let image: Handle<Image> = asset_server.load("ui/orb.png");
    loading_assets.0.push(image.clone().untyped());
    commands.insert_resource(OrbImage(image));
}

fn spawn_orbs(mut commands: Commands, image: Res<OrbImage>, active: Res<ActiveLevel>) {
    for &pos in &active.orbs {
        commands.spawn((
            Sprite::from_image(image.0.clone()),
            Transform::from_translation(pos.extend(1.5)).with_scale(Vec3::splat(SPRITE_SCALE)),
            Orb,
            Hitbox(ORB_SIZE),
        ));
    }
}

fn collect_orbs(
    mut commands: Commands,
    player: Single<(&Transform, &Hitbox, &mut Health), With<Player>>,
    orbs: Query<(Entity, &Transform, &Hitbox), With<Orb>>,
) {
    let (player_tf, player_box, mut health) = player.into_inner();
    let player_pos = player_tf.translation.truncate();

    for (entity, orb_tf, orb_box) in &orbs {
        if overlaps(player_pos, player_box.0, orb_tf.translation.truncate(), orb_box.0) {
            health.current = (health.current + health.max * HEAL_FRACTION).min(health.max);
            commands.entity(entity).despawn();
        }
    }
}
