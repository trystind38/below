use bevy::prelude::*;

use crate::{GameState, loading::despawn_with, player::Player, stats::Health};

// How much of the player's full health the burst uses
pub const BURST_HEALTH_COST: f32 = 0.9;

// How much of the meter the forcefield uses (the light burst needs the whole meter)
pub const FORCEFIELD_METER_COST: f32 = 0.75;

// How fast the meter fills, as a fraction per second (0.025 = 40 seconds to fill)
const FILL_RATE: f32 = 0.025;

// Gauge position and look: sits just under the health segments, top right
const BAR_RIGHT: f32 = 20.;
const BAR_TOP: f32 = 46.;
const BAR_SIZE: Vec2 = Vec2::new(132., 10.);
const EMPTY_COLOR: Color = Color::srgb(0.15, 0.15, 0.2);
const FILLING_COLOR: Color = Color::srgb(0.6, 0.6, 0.35); // still filling, or not enough health
const READY_COLOR: Color = Color::srgb(1., 0.95, 0.5); // meter full and health high enough

// Glow around the player while the meter is full
// (sizes are in the player sprite's own units, which are drawn at 2x on screen)
const AURA_RADIUS: f32 = 17.;
const AURA_LAYERS: u32 = 8; // soft edge comes from stacking this many see-through circles
const AURA_FADE_SPEED: f32 = 3.; // how fast it fades in and out (1.0 = one second)
const AURA_PULSE: f32 = 0.08; // how much it breathes in and out
const AURA_PULSE_SPEED: f32 = 6.;

// The color of the power: used by the glow here and by the light burst in light_burst.rs
pub const POWER_RGB: [f32; 3] = [1., 0.85, 0.25]; // golden
pub const POWER_COLOR: Color = Color::srgb(POWER_RGB[0], POWER_RGB[1], POWER_RGB[2]);
const AURA_COLOR: Color = Color::srgba(POWER_RGB[0], POWER_RGB[1], POWER_RGB[2], 0.05); // very see-through

// 0.0 = empty, 1.0 = full
#[derive(Resource, Default)]
pub struct BurstMeter(pub f32);

impl BurstMeter {
    pub fn is_full(&self) -> bool {
        self.0 >= 1.
    }

    // Use this to fill the meter from anywhere (kills, damage dealt, pickups...)
    pub fn add(&mut self, amount: f32) {
        self.0 = (self.0 + amount).clamp(0., 1.);
    }
}

// The burst needs health ABOVE its cost, so using it never kills the player
pub fn enough_health(health: &Health) -> bool {
    health.current > health.max * BURST_HEALTH_COST
}

#[derive(Component)]
struct MeterBar;

#[derive(Component)]
struct MeterFill;

// The glow around the player. level: 0 = hidden, 1 = fully shown
#[derive(Component)]
struct ReadyAura {
    level: f32,
}

pub struct BurstMeterPlugin;
impl Plugin for BurstMeterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BurstMeter>()
            .add_systems(OnEnter(GameState::Playing), (reset_meter, spawn_meter))
            .add_systems(
                Update,
                (fill_meter, update_meter, attach_aura, update_aura)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), despawn_with::<MeterBar>);
    }
}

fn reset_meter(mut meter: ResMut<BurstMeter>) {
    meter.0 = 0.;
}

fn spawn_meter(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(BAR_RIGHT),
                top: Val::Px(BAR_TOP),
                width: Val::Px(BAR_SIZE.x),
                height: Val::Px(BAR_SIZE.y),
                ..default()
            },
            BackgroundColor(EMPTY_COLOR),
            MeterBar,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: Val::Percent(0.),
                    height: Val::Percent(100.),
                    ..default()
                },
                BackgroundColor(FILLING_COLOR),
                MeterFill,
            ));
        });
}

fn fill_meter(time: Res<Time>, mut meter: ResMut<BurstMeter>) {
    meter.add(FILL_RATE * time.delta_secs());
}

fn update_meter(
    meter: Res<BurstMeter>,
    player: Single<&Health, With<Player>>,
    mut fills: Query<(&mut Node, &mut BackgroundColor), With<MeterFill>>,
) {
    let ready = meter.is_full() && enough_health(&player);
    for (mut node, mut color) in &mut fills {
        node.width = Val::Percent(meter.0 * 100.);
        color.0 = if ready { READY_COLOR } else { FILLING_COLOR };
    }
}

// Runs once when the player appears: adds an invisible glow as a child of the player
fn attach_aura(
    mut commands: Commands,
    players: Query<Entity, Added<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for player in &players {
        let circle = meshes.add(Circle::new(1.).mesh().resolution(48));
        let aura_color = materials.add(AURA_COLOR);

        commands.entity(player).with_children(|p| {
            p.spawn((
                // sits just behind the player's body, and starts at size zero
                Transform::from_xyz(0., 0., -0.1).with_scale(Vec3::ZERO),
                Visibility::default(),
                ReadyAura { level: 0. },
            ))
            .with_children(|aura| {
                // stacked circles from big to small: the middle ends up brightest
                for k in 0..AURA_LAYERS {
                    let radius = AURA_RADIUS * (AURA_LAYERS - k) as f32 / AURA_LAYERS as f32;
                    aura.spawn((
                        Mesh2d(circle.clone()),
                        MeshMaterial2d(aura_color.clone()),
                        Transform::from_xyz(0., 0., k as f32 * 0.001)
                            .with_scale(Vec3::splat(radius)),
                    ));
                }
            });
        });
    }
}

fn update_aura(
    time: Res<Time>,
    meter: Res<BurstMeter>,
    mut auras: Query<(&mut ReadyAura, &mut Transform)>,
) {
    // glow while the meter is full, fade away when it isn't
    let target = if meter.is_full() { 1. } else { 0. };
    let step = AURA_FADE_SPEED * time.delta_secs();

    for (mut aura, mut tf) in &mut auras {
        aura.level += (target - aura.level).clamp(-step, step);

        // a slow breathing pulse
        let pulse = 1. + AURA_PULSE * (time.elapsed_secs() * AURA_PULSE_SPEED).sin();
        tf.scale = Vec3::splat(aura.level * pulse);
    }
}