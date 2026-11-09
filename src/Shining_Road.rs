use bevy::prelude::*;

use crate::{
    GameState,
    burst_meter::{BurstMeter, POWER_RGB},
    loading::despawn_with,
    player::Player,
    stats::{Enemy, Frozen, Hitbox, MoveSpeed},
};

const ROAD_KEY: KeyCode = KeyCode::KeyH;
const ROAD_TIME: f32 = 4.; // seconds the light shines
const ROAD_METER_COST: f32 = 0.9; // fraction of the shared meter it uses

// The beam (world units)
const ROAD_START: f32 = 24.; // how far in front of the player the beam starts
const ROAD_LENGTH: f32 = 1000.;
const ROAD_WIDTH: f32 = 64.; // as tall as the player
const ROAD_LAYERS: u32 = 6; // soft edges come from stacking this many see-through strips
const ROAD_LAYER_ALPHA: f32 = 0.09;
const ROAD_FADE: f32 = 0.3; // seconds to fade in and out at the ends

// How long enemies keep running after they leave the light
const SCARED_LINGER: f32 = 1.5;

// How much faster than normal a scared enemy should run away (for the enemy movement code)
pub const FLEE_SPEED_MULT: f32 = 1.5;

// On the player while the road is shining. dir is locked when it starts: 1 = right, -1 = left.
#[derive(Component)]
pub struct ShiningRoad {
    time_left: f32,
    dir: f32,
}

// On an enemy that is running from the light. Enemy movement code should check for this.
#[derive(Component)]
pub struct Scared {
    pub time_left: f32,
}

#[derive(Component)]
struct RoadBeam;

pub struct ShiningRoadPlugin;
impl Plugin for ShiningRoadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (activate_road, update_road, scare_enemies, tick_scared)
                .chain()
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), despawn_with::<RoadBeam>);
    }
}

// Middle of the beam, given where the player is and which way they face
fn beam_center(player_pos: Vec2, dir: f32) -> Vec2 {
    Vec2::new(
        player_pos.x + dir * (ROAD_START + ROAD_LENGTH / 2.),
        player_pos.y,
    )
}

fn activate_road(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut meter: ResMut<BurstMeter>,
    player: Single<(Entity, &Transform, &Player, Has<ShiningRoad>, Has<Frozen>), With<Player>>,
) {
    let (entity, tf, player, active, frozen) = player.into_inner();

    // not while it's already shining, or during the light burst
    if !keys.just_pressed(ROAD_KEY) || active || frozen {
        return;
    }

    // it costs part of the meter: without enough, nothing happens
    if meter.0 < ROAD_METER_COST {
        return;
    }
    meter.0 -= ROAD_METER_COST;

    // the direction is locked in at the moment you press the key
    let dir = if player.facing < 0. { -1. } else { 1. };
    commands.entity(entity).insert(ShiningRoad {
        time_left: ROAD_TIME,
        dir,
    });

    let [r, g, b] = POWER_RGB;
    commands
        .spawn((
            Transform::from_translation(beam_center(tf.translation.truncate(), dir).extend(3.))
                .with_scale(Vec3::new(1., 0., 1.)),
            Visibility::default(),
            RoadBeam,
        ))
        .with_children(|beam| {
            // strips from wide to narrow: the middle ends up brightest
            for k in 0..ROAD_LAYERS {
                let width = ROAD_WIDTH * (ROAD_LAYERS - k) as f32 / ROAD_LAYERS as f32;
                beam.spawn((
                    Sprite::from_color(
                        Color::srgba(r, g, b, ROAD_LAYER_ALPHA),
                        Vec2::new(ROAD_LENGTH, width),
                    ),
                    Transform::from_xyz(0., 0., k as f32 * 0.001),
                ));
            }
        });
}

fn update_road(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<(Entity, &Transform, &mut ShiningRoad), With<Player>>,
    mut beams: Query<(Entity, &mut Transform), (With<RoadBeam>, Without<Player>)>,
) {
    let (entity, player_tf, mut road) = player.into_inner();
    road.time_left -= time.delta_secs();

    // time's up: turn the light off and let the player turn around again
    if road.time_left <= 0. {
        commands.entity(entity).remove::<ShiningRoad>();
        for (beam, _) in &beams {
            commands.entity(beam).despawn();
        }
        return;
    }

    // the beam follows the player and fades in at the start and out at the end
    let elapsed = ROAD_TIME - road.time_left;
    let level = (elapsed / ROAD_FADE).min(road.time_left / ROAD_FADE).clamp(0., 1.);
    let center = beam_center(player_tf.translation.truncate(), road.dir);

    for (_, mut tf) in &mut beams {
        tf.translation = center.extend(3.);
        tf.scale = Vec3::new(1., level, 1.);
    }
}

// Any enemy touching the beam gets scared
fn scare_enemies(
    mut commands: Commands,
    player: Single<(&Transform, &ShiningRoad), With<Player>>,
    mut enemies: Query<
        (Entity, &Transform, &Hitbox, &MoveSpeed, Option<&mut Scared>),
        (With<Enemy>, Without<Player>),
    >,
) {
    let (player_tf, road) = player.into_inner();
    let center = beam_center(player_tf.translation.truncate(), road.dir);
    let beam_size = Vec2::new(ROAD_LENGTH, ROAD_WIDTH);

    for (entity, tf, hitbox, speed, scared) in &mut enemies {
        // an enemy that can't move (Straight Teeth) can't run, so the light ignores it
        if speed.0 <= 0. {
            continue;
        }

        let d = (tf.translation.truncate() - center).abs();
        let reach = (beam_size + hitbox.0) / 2.;
        if d.x < reach.x && d.y < reach.y {
            match scared {
                Some(mut scared) => scared.time_left = SCARED_LINGER,
                None => {
                    commands.entity(entity).insert(Scared {
                        time_left: SCARED_LINGER,
                    });
                }
            }
        }
    }
}

fn tick_scared(mut commands: Commands, time: Res<Time>, mut scared: Query<(Entity, &mut Scared)>) {
    for (entity, mut scared) in &mut scared {
        scared.time_left -= time.delta_secs();
        if scared.time_left <= 0. {
            commands.entity(entity).remove::<Scared>();
        }
    }
}