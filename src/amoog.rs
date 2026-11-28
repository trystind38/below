use bevy::prelude::*;
use rand::Rng;

use crate::{
    GameState, TILE_SIZE,
    level::{ActiveLevel, Brick, intro_only},
    loading::{LoadingAssets, despawn_with},
    player::{PLAYER_SIZE, Player},
    shining_road::{FLEE_SPEED_MULT, Scared},
    sound::{SoundEvent, SoundKind},
    stats::{AMOOG, Health, HitCooldown, Hitbox, MoveSpeed},
    flytrap::{FIREBALL_GRAVITY, Fireball},
};

const SPRITE_SCALE: f32 = 2.;
const FRAME_SIZE: u32 = 32;
const FRAME_COUNT: u32 = 15;
const AMOOG_SIZE: Vec2 = Vec2::new(40., 56.);

const GRAVITY: f32 = 2000.;
const MAX_FALL_SPEED: f32 = 1200.;
const SKIN: f32 = 0.01;
const PROBE: f32 = 8.;

const PATROL_MIN: f32 = TILE_SIZE * 2.;
const PATROL_MAX: f32 = TILE_SIZE * 6.;
const CHASE_SPEED_MULT: f32 = 1.5;

const LISTEN_TIME: f32 = 0.5;
const KEEP_AWAY: f32 = 200.;
const FIRE_RANGE: f32 = 500.;
const OVERHEAD_RANGE: f32 = TILE_SIZE;
const MEMORY_TIME: f32 = 4.;
const VELOCITY_WINDOW: f32 = 1.;
const MIN_SOUND_GAP: f32 = 0.15;
const MAX_LEAD_TIME: f32 = 1.5;

const FIRE_COOLDOWN: f32 = 1.5;
const WINDUP_TIME: f32 = 0.3;
const SHOT_TIME: f32 = 0.55;
const FIREBALL_SPEED: f32 = 400.;
const MIN_FLIGHT_TIME: f32 = 0.35;
const MAX_FLIGHT_TIME: f32 = 1.2;
const FIREBALL_DRAW_SIZE: Vec2 = Vec2::new(24., 24.);
const SIDE_EYE: Vec2 = Vec2::new(6., 2.);
const FRONT_EYE: Vec2 = Vec2::new(0., 2.);

const WALK_FRAME: usize = 0;
const LISTEN_FRAME: usize = 4;
const TWITCH_FRAME: usize = 5;
const SNAP_FRAME: usize = 6;
const SIDE_SHOOT_FRAME: usize = 7;
const FRONT_SHOOT_FRAME: usize = 11;
const SHOOT_FRAME_ENDS: [f32; 4] = [0.15, WINDUP_TIME, 0.4, SHOT_TIME];
const WALK_FRAME_DISTANCE: f32 = 9.;
const LISTEN_LOOP: f32 = 0.75;
const TWITCH_TIME: f32 = 0.15;
const SNAP_TIME: f32 = 0.2;

#[derive(Clone, Copy)]
pub struct Heard {
    pub pos: Vec2,
    pub at: f32,
}

#[derive(Clone, Copy)]
pub struct Shot {
    pub elapsed: f32,
    pub overhead: bool,
    fired: bool,
}

#[derive(Component, Default)]
pub struct Amoog {
    pub velocity: Vec2,
    pub facing: f32,
    pub heard: Option<Heard>,
    pub heard_speed: f32,
    pub shot: Option<Shot>,
    listen_timer: f32,
    fire_cooldown: f32,
    patrol_remaining: f32,
    patrol_dir: f32,
}

#[derive(Component, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum AmoogState {
    #[default]
    Patrol,
    Listen,
    Chase,
    Attack,
    Retreat,
}

#[derive(Component, Default)]
struct AmoogAnimation {
    walking: bool,
    timer: f32,
    frame: usize,
}

#[derive(Resource)]
struct AmoogSheet {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    fireball: Handle<Image>,
}

pub struct AmoogPlugin;
impl Plugin for AmoogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_amoog)
            .add_systems(
                OnEnter(GameState::Playing),
                // Only Level 2 has Amoog spawns
                spawn_level_amoogs.run_if(not(intro_only)),
            )
            .add_systems(
                FixedUpdate,
                (amoog_decision_tree, amoog_movement, amoog_shoot)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, animate_amoog.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), despawn_with::<Amoog>);
    }
}

fn load_amoog(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let image: Handle<Image> = asset_server.load("enemy/amoog_sheet.png");
    let fireball: Handle<Image> = asset_server.load("enemy/fireball.png");
    loading_assets.0.push(image.clone().untyped());
    loading_assets.0.push(fireball.clone().untyped());

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(FRAME_SIZE),
        FRAME_COUNT,
        1,
        None,
        None,
    ));

    commands.insert_resource(AmoogSheet {
        image,
        layout,
        fireball,
    });
}

// Same spots and placement as spawn_level_geralds, standing on the floor
fn spawn_level_amoogs(mut commands: Commands, sheet: Res<AmoogSheet>, active: Res<ActiveLevel>) {
    for spawn in &active.enemies.amoogs {
        let tile = spawn.tile.as_vec2();
        // Feet on top of the floor tile below, plus a pixel so gravity settles it
        let pos = Vec2::new(
            (tile.x + 0.5) * TILE_SIZE,
            tile.y * TILE_SIZE + AMOOG_SIZE.y / 2. + 1.,
        );
        spawn_amoog_at(&mut commands, &sheet, pos);
    }
}

fn spawn_amoog_at(commands: &mut Commands, sheet: &AmoogSheet, pos: Vec2) {
    commands.spawn((
        Sprite::from_atlas_image(
            sheet.image.clone(),
            TextureAtlas {
                layout: sheet.layout.clone(),
                index: LISTEN_FRAME,
            },
        ),
        Transform::from_translation(pos.extend(2.))
            .with_scale(Vec3::splat(SPRITE_SCALE)),
        Amoog {
            facing: 1.,
            ..default()
        },
        AmoogState::default(),
        AmoogAnimation::default(),
        AMOOG.bundle(),
        Hitbox(AMOOG_SIZE),
        HitCooldown::default(),
    ));
}

fn amoog_decision_tree(
    time: Res<Time>,
    active: Res<ActiveLevel>,
    mut sounds: MessageReader<SoundEvent>,
    player: Single<&Health, With<Player>>,
    mut amoogs: Query<(&Transform, &mut Amoog, &mut AmoogState)>,
    bricks: Query<&Transform, (With<Brick>, Without<Amoog>)>,
) {
    let now = time.elapsed_secs();
    let dt = time.delta_secs();
    let player_alive = player.current > 0.;
    let sounds: Vec<SoundEvent> = sounds.read().copied().collect();

    for (tf, mut amoog, mut state) in &mut amoogs {
        let pos = tf.translation.truncate();

        let heard = sounds
            .iter()
            .filter(|sound| player_alive && sound.pos.distance(pos) <= sound.kind.loudness())
            .last();
        if let Some(sound) = heard {
            hear(&mut amoog, body_position(sound), now);
        }

        amoog.listen_timer = (amoog.listen_timer - dt).max(0.);
        let recent = amoog.heard.filter(|heard| now - heard.at <= MEMORY_TIME);
        let next = match recent {
            Some(heard) if player_alive => {
                let distance = heard.pos.distance(pos);
                let away = (pos.x - heard.pos.x).signum();
                if *state == AmoogState::Patrol {
                    amoog.listen_timer = LISTEN_TIME;
                    AmoogState::Listen
                } else if amoog.listen_timer > 0. {
                    AmoogState::Listen
                } else if distance < KEEP_AWAY
                    && !overhead(pos, heard.pos)
                    && !blocked(pos, away, &bricks, &active)
                {
                    AmoogState::Retreat
                } else if distance <= FIRE_RANGE {
                    AmoogState::Attack
                } else {
                    AmoogState::Chase
                }
            }
            _ => AmoogState::Patrol,
        };

        if *state != next {
            *state = next;
        }
    }
}

fn hear(amoog: &mut Amoog, pos: Vec2, now: f32) {
    if let Some(last) = amoog.heard {
        let gap = now - last.at;
        if gap > VELOCITY_WINDOW {
            amoog.heard_speed = 0.;
        } else if gap >= MIN_SOUND_GAP {
            amoog.heard_speed = (pos.x - last.pos.x) / gap;
        }
    }
    amoog.heard = Some(Heard { pos, at: now });
}

fn body_position(sound: &SoundEvent) -> Vec2 {
    // Steps and landings are heard at the player's feet
    if matches!(sound.kind, SoundKind::Step | SoundKind::Land) {
        sound.pos + Vec2::Y * PLAYER_SIZE.y / 2.
    } else {
        sound.pos
    }
}

fn overhead(pos: Vec2, target: Vec2) -> bool {
    target.y - pos.y > AMOOG_SIZE.y / 2. && (target.x - pos.x).abs() <= OVERHEAD_RANGE
}

fn blocked(
    pos: Vec2,
    dir: f32,
    bricks: &Query<&Transform, (With<Brick>, Without<Amoog>)>,
    active: &ActiveLevel,
) -> bool {
    let probe_pos = pos + Vec2::X * dir * (AMOOG_SIZE.x + PROBE) / 2.;
    let probe_size = Vec2::new(PROBE, AMOOG_SIZE.y - TILE_SIZE);
    if probe_pos.x < active.bounds_min.x || probe_pos.x > active.bounds_max.x {
        return true;
    }
    bricks.iter().any(|brick| {
        overlaps(
            probe_pos,
            probe_size,
            brick.translation.truncate(),
            Vec2::splat(TILE_SIZE),
        )
    })
}

fn amoog_movement(
    time: Res<Time>,
    active: Res<ActiveLevel>,
    player: Single<&Transform, With<Player>>,
    mut amoogs: Query<
        (&mut Transform, &mut Amoog, &AmoogState, &MoveSpeed, Has<Scared>),
        Without<Player>,
    >,
    bricks: Query<&Transform, (With<Brick>, Without<Amoog>)>,
) {
    let dt = time.delta_secs();

    for (mut tf, mut amoog, state, speed, scared) in &mut amoogs {
        let pos = tf.translation.truncate();
        let toward = amoog
            .heard
            .map_or(amoog.facing, |heard| (heard.pos.x - pos.x).signum());

        let vx = if scared {
            (pos.x - player.translation.x).signum() * speed.0 * FLEE_SPEED_MULT
        } else if amoog.shot.is_some() {
            0.
        } else {
            match state {
                AmoogState::Patrol => patrol(&mut amoog, speed.0, dt),
                AmoogState::Chase => toward * speed.0 * CHASE_SPEED_MULT,
                AmoogState::Retreat => -toward * speed.0 * CHASE_SPEED_MULT,
                AmoogState::Listen | AmoogState::Attack => 0.,
            }
        };
        amoog.velocity.x = vx;
        amoog.facing = if vx != 0. { vx.signum() } else { toward };
        amoog.velocity.y = (amoog.velocity.y - GRAVITY * dt).max(-MAX_FALL_SPEED);

        tf.translation.x += amoog.velocity.x * dt;
        for brick in &bricks {
            let (pen, push) = penetration(tf.translation, brick.translation);
            if pen.x > SKIN && pen.y > SKIN {
                tf.translation.x += pen.x * push.x;
                amoog.velocity.x = 0.;
                amoog.patrol_remaining = 0.;
            }
        }

        tf.translation.y += amoog.velocity.y * dt;
        for brick in &bricks {
            let (pen, push) = penetration(tf.translation, brick.translation);
            if pen.x > SKIN && pen.y > SKIN {
                tf.translation.y += pen.y * push.y;
                amoog.velocity.y = 0.;
            }
        }

        let half_w = AMOOG_SIZE.x / 2.;
        let clamped = tf
            .translation
            .x
            .clamp(active.bounds_min.x + half_w, active.bounds_max.x - half_w);
        if clamped != tf.translation.x {
            tf.translation.x = clamped;
            amoog.velocity.x = 0.;
            amoog.patrol_remaining = 0.;
        }
    }
}

fn patrol(amoog: &mut Amoog, speed: f32, dt: f32) -> f32 {
    if amoog.patrol_remaining <= 0. {
        amoog.patrol_dir = if amoog.patrol_dir > 0. { -1. } else { 1. };
        amoog.patrol_remaining = rand::thread_rng().gen_range(PATROL_MIN..=PATROL_MAX);
    }
    amoog.patrol_remaining -= speed * dt;
    amoog.patrol_dir * speed
}

fn amoog_shoot(
    mut commands: Commands,
    time: Res<Time>,
    sheet: Res<AmoogSheet>,
    mut amoogs: Query<(&Transform, &mut Amoog, &AmoogState, Has<Scared>)>,
) {
    let dt = time.delta_secs();
    let now = time.elapsed_secs();

    for (tf, mut amoog, state, scared) in &mut amoogs {
        amoog.fire_cooldown = (amoog.fire_cooldown - dt).max(0.);
        let pos = tf.translation.truncate();

        if scared {
            amoog.shot = None;
            continue;
        }

        if let Some(mut shot) = amoog.shot {
            shot.elapsed += dt;
            if !shot.fired && shot.elapsed >= WINDUP_TIME {
                shot.fired = true;
                let eye = if shot.overhead {
                    FRONT_EYE
                } else {
                    SIDE_EYE * Vec2::new(amoog.facing, 1.)
                };
                if let Some(heard) = amoog.heard {
                    let from = pos + eye;
                    commands.spawn((
                        Sprite {
                            image: sheet.fireball.clone(),
                            custom_size: Some(FIREBALL_DRAW_SIZE),
                            ..default()
                        },
                        Transform::from_translation(from.extend(3.)),
                        Fireball {
                            vel: aim(heard, amoog.heard_speed, from, now, dt),
                        },
                    ));
                }
            }
            amoog.shot = (shot.elapsed < SHOT_TIME).then_some(shot);
            continue;
        }

        let Some(heard) = amoog.heard else {
            continue;
        };
        if *state == AmoogState::Attack && amoog.fire_cooldown <= 0. {
            amoog.shot = Some(Shot {
                elapsed: 0.,
                overhead: overhead(pos, heard.pos),
                fired: false,
            });
            amoog.fire_cooldown = FIRE_COOLDOWN;
        }
    }
}

fn aim(heard: Heard, heard_speed: f32, from: Vec2, now: f32, dt: f32) -> Vec2 {
    let since = now - heard.at;
    let estimate = heard.pos + Vec2::X * heard_speed * since.min(MAX_LEAD_TIME);
    let flight =
        (estimate.distance(from) / FIREBALL_SPEED).clamp(MIN_FLIGHT_TIME, MAX_FLIGHT_TIME);
    let flight = (flight / dt).ceil() * dt;
    let lead = (since + flight).min(MAX_LEAD_TIME);
    let target = heard.pos + Vec2::X * heard_speed * lead;
    launch_velocity(target - from, flight, dt)
}

// Solved for move_fireballs' fixed step (velocity, then position), so it lands exactly on target
fn launch_velocity(offset: Vec2, flight: f32, dt: f32) -> Vec2 {
    Vec2::new(
        offset.x / flight,
        offset.y / flight + FIREBALL_GRAVITY * (flight + dt) / 2.,
    )
}

fn animate_amoog(
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    mut amoogs: Query<(&Amoog, &mut AmoogAnimation, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (amoog, mut anim, mut sprite) in &mut amoogs {
        let walking = amoog.shot.is_none() && amoog.velocity.x != 0.;
        if walking != anim.walking {
            anim.walking = walking;
            anim.timer = 0.;
            anim.frame = 0;
        }

        let index = if let Some(shot) = amoog.shot {
            let frame = SHOOT_FRAME_ENDS
                .iter()
                .position(|end| shot.elapsed < *end)
                .unwrap_or(SHOOT_FRAME_ENDS.len() - 1);
            if shot.overhead {
                FRONT_SHOOT_FRAME + frame
            } else {
                SIDE_SHOOT_FRAME + frame
            }
        } else if walking {
            anim.timer += amoog.velocity.x.abs() * dt;
            while anim.timer >= WALK_FRAME_DISTANCE {
                anim.timer -= WALK_FRAME_DISTANCE;
                anim.frame = (anim.frame + 1) % 4;
            }
            WALK_FRAME + anim.frame
        } else {
            anim.timer = (anim.timer + dt) % LISTEN_LOOP;
            let snapped = amoog
                .heard
                .is_some_and(|heard| fixed.elapsed_secs() - heard.at < SNAP_TIME);
            if snapped {
                SNAP_FRAME
            } else if anim.timer < LISTEN_LOOP - TWITCH_TIME {
                LISTEN_FRAME
            } else {
                TWITCH_FRAME
            }
        };

        let overhead = amoog.shot.is_some_and(|shot| shot.overhead);
        sprite.flip_x = amoog.facing < 0. && !overhead;
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = index;
        }
    }
}

fn overlaps(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let d = (a_pos - b_pos).abs();
    let reach = (a_size + b_size) / 2.;
    d.x < reach.x && d.y < reach.y
}

fn penetration(amoog: Vec3, tile: Vec3) -> (Vec2, Vec2) {
    let d = (amoog - tile).truncate();
    let pen = (AMOOG_SIZE + Vec2::splat(TILE_SIZE)) / 2. - d.abs();
    (pen, d.signum())
}