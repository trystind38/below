use bevy::prelude::*;

use crate::{
    GameState,
    burst_meter::{BurstMeter, FORCEFIELD_METER_COST, POWER_RGB},
    player::Player,
};

const FORCEFIELD_KEY: KeyCode = KeyCode::KeyG;
const FORCEFIELD_TIME: f32 = 7.; // seconds it lasts

// Enemies deal this fraction of their normal damage while the field is up.
// Same survival as doubling the player's health, but the health bar is never touched.
pub const FORCEFIELD_DAMAGE_MULT: f32 = 0.5;

// After the field drops, the player is weakened for a while
const VULNERABLE_TIME: f32 = 4.; // seconds
pub const VULNERABLE_DAMAGE_MULT: f32 = 1.5; // enemies deal 1.5x damage

// The bubble around the player
// (sizes are in the player sprite's own units, which are drawn at 2x on screen)
const BUBBLE_RADIUS: f32 = 28.;
const BUBBLE_FILL_COLOR: Color = Color::srgba(POWER_RGB[0], POWER_RGB[1], POWER_RGB[2], 0.08); // faint, same color as the aura
const BUBBLE_RING_COLOR: Color = Color::srgba(POWER_RGB[0], POWER_RGB[1], POWER_RGB[2], 0.55); // brighter edge
const RING_THICKNESS: f32 = 0.07; // ring width as a fraction of the radius
const PULSE: f32 = 0.03; // how much the bubble breathes in and out
const PULSE_SPEED: f32 = 5.;

// In the last moments the bubble blinks to warn the field is about to drop
const WARN_TIME: f32 = 1.5;
const WARN_BLINK_RATE: f32 = 8.;

// On the player while the forcefield is up
#[derive(Component)]
pub struct ForceField {
    time_left: f32,
}

#[derive(Component)]
struct ForceFieldBubble;

// On the player for a few seconds after the forcefield ends
#[derive(Component)]
pub struct Vulnerable {
    time_left: f32,
}

// Use this wherever the player takes damage from an enemy:
// half while the field is up, 1.5x while weakened afterwards, normal otherwise
pub fn damage_taken(damage: f32, shielded: bool, vulnerable: bool) -> f32 {
    if shielded {
        damage * FORCEFIELD_DAMAGE_MULT
    } else if vulnerable {
        damage * VULNERABLE_DAMAGE_MULT
    } else {
        damage
    }
}

pub struct ForceFieldPlugin;
impl Plugin for ForceFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (activate_forcefield, update_forcefield, update_vulnerable)
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn activate_forcefield(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut meter: ResMut<BurstMeter>,
    player: Single<(Entity, Has<ForceField>), With<Player>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let (player, active) = player.into_inner();

    // pressing the key while it's already up does nothing
    if !keys.just_pressed(FORCEFIELD_KEY) || active {
        return;
    }

    // it costs part of the meter: without enough, nothing happens
    if meter.0 < FORCEFIELD_METER_COST {
        return;
    }
    meter.0 -= FORCEFIELD_METER_COST;

    let disc = meshes.add(Circle::new(1.).mesh().resolution(64));
    let ring = meshes.add(Annulus::new(1. - RING_THICKNESS, 1.).mesh().resolution(64));
    let fill = materials.add(BUBBLE_FILL_COLOR);
    let edge = materials.add(BUBBLE_RING_COLOR);

    commands
        .entity(player)
        .remove::<Vulnerable>() // a new shield cancels any leftover weakness
        .insert(ForceField {
            time_left: FORCEFIELD_TIME,
        })
        .with_children(|p| {
            // the bubble is a child of the player, so it follows them everywhere
            p.spawn((
                Transform::from_xyz(0., 0., 0.3).with_scale(Vec3::splat(BUBBLE_RADIUS)),
                Visibility::default(),
                ForceFieldBubble,
            ))
            .with_children(|bubble| {
                bubble.spawn((Mesh2d(disc), MeshMaterial2d(fill), Transform::default()));
                bubble.spawn((
                    Mesh2d(ring),
                    MeshMaterial2d(edge),
                    Transform::from_xyz(0., 0., 0.01),
                ));
            });
        });
}

fn update_forcefield(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<(Entity, &mut ForceField), With<Player>>,
    mut bubbles: Query<(Entity, &mut Transform, &mut Visibility), With<ForceFieldBubble>>,
) {
    let (player, mut field) = player.into_inner();
    field.time_left -= time.delta_secs();

    // time's up: drop the field and remove the bubble
    if field.time_left <= 0. {
        commands
            .entity(player)
            .remove::<ForceField>()
            .insert(Vulnerable {
                time_left: VULNERABLE_TIME,
            });
        for (bubble, _, _) in &bubbles {
            commands.entity(bubble).despawn();
        }
        return;
    }

    // blink in the last moments as a warning
    let blink_off =
        field.time_left < WARN_TIME && (field.time_left * WARN_BLINK_RATE) as u32 % 2 == 0;
    let pulse = 1. + PULSE * (time.elapsed_secs() * PULSE_SPEED).sin();

    for (_, mut tf, mut visibility) in &mut bubbles {
        tf.scale = Vec3::splat(BUBBLE_RADIUS * pulse);
        *visibility = if blink_off {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

fn update_vulnerable(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<(Entity, &mut Vulnerable), With<Player>>,
) {
    let (player, mut weakened) = player.into_inner();
    weakened.time_left -= time.delta_secs();

    if weakened.time_left <= 0. {
        commands.entity(player).remove::<Vulnerable>();
    }
}