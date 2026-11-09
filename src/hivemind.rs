use bevy::prelude::*;
use rand::Rng;

use crate :: {
    GameState, LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_RIGHT, TILE_SIZE,
    level::Brick,
    loading::{ LoadingAssets, despawn_with },
    stats::{ HIVEMIND, HitCooldown, Hitbox, MoveSpeed },
    player::Player,
    shining_road::{ FLEE_SPEED_MULT, Scared },
};

const SPRITE_SCALE: f32 = 2.;
const HIVEMIND_SIZE: Vec2 = Vec2::new(32., 32.);

#[derive(Component, Default)]
pub struct Hivemind {
    pub velocity: Vec2,
    pub grounded: bool,
    pub attack_timer: f32,
    pub animation_timer: f32,

    //idle animation
    pub idle_timer: f32,
    pub idle_origin_y: f32,
    pub idle_amplitude: f32,
    pub idle_speed: f32,
}

#[derive(Resource)]
struct HivemindSheet {
    sprite_sheet: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

#[derive(Component, Default, PartialEq, Eq)]
pub enum HivemindState {
    #[default]
    Idle,
    Patrol,
    Chase,
    Attack,
}

#[derive(Component)]
pub struct HivemindPatrol {
    pub distance_remaining: f32,
    pub direction: f32,
}

pub struct HivemindPlugin;
impl Plugin for HivemindPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_hivemind)
            .add_systems(OnEnter(GameState::Playing), spawn_hivemind)
            .add_systems(Update, (
                //hivemind_ai,
                update_hivemind_sprite,
                hivemind_idle,
                hivemind_flee,
            ))
            .add_systems(OnExit(GameState::Playing), despawn_with::<Hivemind>);
    }
}

fn load_hivemind(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let sprite_sheet: Handle<Image> = asset_server.load("enemy/hivemind_sprite_sheet.png");

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(16, 16), //frame dimensions
        2,                  //columns
        1,                  //rows
        None,
        None,
    );
    let layout = texture_atlas_layouts.add(layout);

    loading_assets.0.push(sprite_sheet.clone().untyped());

    commands.insert_resource(HivemindSheet {sprite_sheet, layout});
}

fn spawn_hivemind(mut commands: Commands, sheet: Res<HivemindSheet>) {
    let spawn_y = 25. + LEVEL_BOTTOM + TILE_SIZE + HIVEMIND_SIZE.y / 2.;

    commands.spawn((
        Sprite::from_atlas_image(
            sheet.sprite_sheet.clone(),
            TextureAtlas {
                layout: sheet.layout.clone(),
                index: 0,
            },
        ),
        Transform::from_xyz(196., LEVEL_BOTTOM + TILE_SIZE + HIVEMIND_SIZE.y / 2., 2.)
            .with_scale(Vec3::splat(SPRITE_SCALE)),
        Hivemind{
            idle_origin_y: spawn_y,
            idle_amplitude: 4.,     //pixel amplitude of the bounce
            idle_speed: 3.,         //radians per second
            ..default()
        },
        HIVEMIND.bundle(),
        Hitbox(HIVEMIND_SIZE),
        HitCooldown::default(),
        HivemindPatrol {
            distance_remaining: 0.,
            direction: 1.,
        },
        HivemindState::default(),
    ));
}

fn update_hivemind_sprite(
    time: Res<Time>,
    mut query: Query<(&mut Hivemind, &mut Sprite)>,
) {
    for (mut hivemind, mut sprite) in &mut query {
        hivemind.animation_timer += time.delta_secs();

        if hivemind.animation_timer >= 0.15 {
            hivemind.animation_timer = 0.;

            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % 2;
            }
        }
    }
}

fn hivemind_idle(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Hivemind, &HivemindState)>,
) {
    for (mut tf, mut hivemind, state) in &mut query {
        if *state != HivemindState::Idle {
            continue;
        }

        hivemind.idle_timer += time.delta_secs();

        tf.translation.y =
            hivemind.idle_origin_y
            + (hivemind.idle_timer * hivemind.idle_speed).sin()
            * hivemind.idle_amplitude;
    }
}

// Shining Road: a scared hivemind drifts away from the player.
// (The hivemind has no movement of its own yet, so this is its only way of moving.)
fn hivemind_flee(
    time: Res<Time>,
    player: Single<&Transform, With<Player>>,
    mut hiveminds: Query<
        (&mut Transform, &MoveSpeed),
        (With<Hivemind>, With<Scared>, Without<Player>),
    >,
    bricks: Query<&Transform, (With<Brick>, Without<Hivemind>, Without<Player>)>,
) {
    let dt = time.delta_secs().min(1. / 30.);

    for (mut tf, speed) in &mut hiveminds {
        let away = (tf.translation.x - player.translation.x).signum();
        tf.translation.x += away * speed.0 * FLEE_SPEED_MULT * dt;

        // Don't drift through the terrain
        for brick in &bricks {
            let d = (tf.translation - brick.translation).truncate();
            let pen = (HIVEMIND_SIZE + Vec2::splat(TILE_SIZE)) / 2. - d.abs();
            if pen.x > 0.01 && pen.y > 0.01 {
                tf.translation.x += pen.x * d.signum().x;
            }
        }

        // Don't leave the level
        let half_w = HIVEMIND_SIZE.x / 2.;
        tf.translation.x = tf
            .translation
            .x
            .clamp(LEVEL_LEFT + half_w, LEVEL_RIGHT - half_w);
    }
}
/*
fn hivemind_ai(
    time: Res<Time>,
    player: Single<&Transform, With<Player>>,
    mut hivemind: Single<
        (
            &mut Transform,
            &mut Hivemind,
            &mut HivemindPatrol,
            &mut HivemindState,
        ),
        Without<Player>,
    >,
) {
    let (mut tf, mut hivemind, mut direction, mut patrol, state) = hivemind.into_inner();
    let dt = time.delta_secs().min(1. / 30.);

    match *state {
        HivemindState::Idle => {
            
        }
        HivemindState::Patrol => {
            
        }

        HivemindState::Chase => {
            
        }

        HivemindState::Attack => {
            
        }
    }
}
*/