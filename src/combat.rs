use bevy::prelude::*;

use crate::{
    GameState, LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_RIGHT, LEVEL_TOP, TILE_SIZE,
    level::Brick,
    loading::despawn_with,
    player::{Player, PlayerSystems},
    stats::{
        Damage, ENEMY_HIT_COOLDOWN, Enemy, HIT_COOLDOWN, Health, HitCooldown, Hitbox,
        Invulnerable, PLAYER_STOMP_DAMAGE, Stompable,
    },
};

const BULLET_HIT_SIZE: Vec2 = Vec2::splat(6.);

const FLASH_TIME: f32 = 0.1;
// Above 1.0 brightens the sprite instead of tinting it
const FLASH_COLOR: Color = Color::linear_rgb(4., 4., 4.);
const BLINK_RATE: f32 = 12.; // toggles per second
const BLINK_ALPHA: f32 = 0.35;

const KNOCKBACK: Vec2 = Vec2::new(350., 400.);
const KNOCKBACK_STUN: f32 = 0.25;

const STOMP_BOUNCE: f32 = 550.;
// How far into the hitbox the feet can start and still count as a stomp
const STOMP_TOLERANCE: f32 = 4.;

#[derive(Component)]
pub struct Bullet {
    pub vel: Vec2,
    pub damage: f32,
    pub life: f32,
}

pub struct CombatPlugin;
impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                move_bullets,
                bullet_hits,
                player_enemy_contact,
                tick_hit_cooldowns,
                show_enemy_hit_feedback,
                despawn_dead_enemies,
            )
                .chain()
                // Stomp check needs this frame's player movement
                .after(PlayerSystems)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), despawn_with::<Bullet>);
    }
}

fn overlaps(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let d = (a_pos - b_pos).abs();
    let reach = (a_size + b_size) / 2.;
    d.x < reach.x && d.y < reach.y
}

fn move_bullets(
    mut commands: Commands,
    time: Res<Time>,
    mut bullets: Query<(Entity, &mut Transform, &mut Bullet)>,
    bricks: Query<&Transform, (With<Brick>, Without<Bullet>)>,
) {
    let dt = time.delta_secs().min(1. / 30.);
    for (entity, mut tf, mut bullet) in &mut bullets {
        tf.translation += (bullet.vel * dt).extend(0.);
        bullet.life -= dt;

        let pos = tf.translation.truncate();
        let left_level = pos.x < LEVEL_LEFT
            || pos.x > LEVEL_RIGHT
            || pos.y < LEVEL_BOTTOM
            || pos.y > LEVEL_TOP;
        let hit_wall = bricks.iter().any(|brick| {
            overlaps(
                pos,
                BULLET_HIT_SIZE,
                brick.translation.truncate(),
                Vec2::splat(TILE_SIZE),
            )
        });

        if bullet.life <= 0. || left_level || hit_wall {
            commands.entity(entity).despawn();
        }
    }
}

fn bullet_hits(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &Bullet)>,
    mut enemies: Query<
        (&Transform, &Hitbox, &mut Health, &mut HitCooldown, Has<Invulnerable>),
        (With<Enemy>, Without<Bullet>),
    >,
) {
    for (bullet_entity, bullet_tf, bullet) in &bullets {
        let pos = bullet_tf.translation.truncate();
        for (enemy_tf, hitbox, mut health, mut cooldown, invulnerable) in &mut enemies {
            // Bullets pass through during i-frames
            if cooldown.0 > 0.
                || !overlaps(pos, BULLET_HIT_SIZE, enemy_tf.translation.truncate(), hitbox.0)
            {
                continue;
            }

            // Invulnerable enemies block bullets but take no damage
            commands.entity(bullet_entity).despawn();
            if !invulnerable {
                health.current = (health.current - bullet.damage).max(0.);
                cooldown.0 = ENEMY_HIT_COOLDOWN;
            }
            break;
        }
    }
}

fn player_enemy_contact(
    time: Res<Time>,
    player: Single<
        (&Transform, &Hitbox, &mut Player, &mut Health, &mut HitCooldown),
        Without<Enemy>,
    >,
    mut enemies: Query<
        (&mut Transform, &Hitbox, &Damage, &mut Health, &mut HitCooldown, Has<Invulnerable>, Has<Stompable>),
        (With<Enemy>, Without<Player>),
    >,
) {
    let (player_tf, player_box, mut player, mut health, mut cooldown) = player.into_inner();
    if health.current <= 0. {
        return;
    }

    let dt = time.delta_secs().min(1. / 30.);
    let player_pos = player_tf.translation.truncate();
    let feet = player_pos.y - player_box.0.y / 2.;
    // Where the feet were before this frame's movement
    let prev_feet = feet - player.vel.y * dt;
    let falling = player.vel.y < 0.;
    let mut stomped = false;

    for (mut enemy_tf, enemy_box, damage, mut enemy_health, mut enemy_cooldown, invulnerable, stompable) in
        &mut enemies
    {
        let enemy_pos = enemy_tf.translation.truncate();
        // Dead enemies stay around for their death flash
        if enemy_health.current <= 0.
            || !overlaps(player_pos, player_box.0, enemy_pos, enemy_box.0)
        {
            continue;
        }

        let enemy_top = enemy_pos.y + enemy_box.0.y / 2.;
        if stompable && falling && prev_feet >= enemy_top - STOMP_TOLERANCE {
            stomped = true;
            player.vel.y = STOMP_BOUNCE;
            player.can_fastfall = true;
            // Stomps ignore the enemy's i-frames
            if !invulnerable {
                enemy_health.current = (enemy_health.current - PLAYER_STOMP_DAMAGE).max(0.);
                enemy_cooldown.0 = ENEMY_HIT_COOLDOWN;
                if enemy_health.current <= 0. {
                    // Squash flat against the floor
                    enemy_tf.scale.y *= 0.5;
                    enemy_tf.translation.y -= enemy_box.0.y / 4.;
                }
            }
        } else if !stomped && cooldown.0 <= 0. {
            health.current = (health.current - damage.0).max(0.);
            cooldown.0 = HIT_COOLDOWN;

            let away = if player_pos.x == enemy_pos.x {
                -player.facing
            } else {
                (player_pos.x - enemy_pos.x).signum()
            };
            player.vel = Vec2::new(away * KNOCKBACK.x, KNOCKBACK.y);
            player.stun = KNOCKBACK_STUN;
        }
    }
}

fn tick_hit_cooldowns(time: Res<Time>, mut cooldowns: Query<&mut HitCooldown>) {
    for mut cooldown in &mut cooldowns {
        cooldown.0 = (cooldown.0 - time.delta_secs()).max(0.);
    }
}

pub fn blink_color(cooldown: &HitCooldown) -> Color {
    if cooldown.0 > 0. && (cooldown.0 * BLINK_RATE) as u32 % 2 == 0 {
        Color::srgba(1., 1., 1., BLINK_ALPHA)
    } else {
        Color::WHITE
    }
}

// The flash is the start of the i-frames
fn is_flashing(cooldown: &HitCooldown) -> bool {
    cooldown.0 > ENEMY_HIT_COOLDOWN - FLASH_TIME
}

fn show_enemy_hit_feedback(
    mut enemies: Query<(&HitCooldown, &mut Sprite), With<Enemy>>,
) {
    for (cooldown, mut sprite) in &mut enemies {
        sprite.color = if is_flashing(cooldown) { FLASH_COLOR } else { blink_color(cooldown) };
    }
}

// Waits for the flash so the killing hit is still visible
fn despawn_dead_enemies(
    mut commands: Commands,
    enemies: Query<(Entity, &Health, Option<&HitCooldown>), With<Enemy>>,
) {
    for (entity, health, cooldown) in &enemies {
        if health.current <= 0. && !cooldown.is_some_and(is_flashing) {
            commands.entity(entity).despawn();
        }
    }
}
