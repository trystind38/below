use bevy::prelude::*;
use rand::Rng;

use crate :: {
    GameState, LEVEL_BOTTOM, TILE_SIZE,
    level::Brick,
    loading::{ LoadingAssets, despawn_with },
    stats::{ HIVEMIND, HitCooldown, Hitbox, MoveSpeed, Health, Damage, },
    player::Player,
    hivemind_controller::{HIVE_SIZE, Hive, HiveFlower},
};

const SPRITE_SCALE: f32 = 2.;
const HIVEMIND_SIZE: Vec2 = Vec2::new(32., 32.);
const AGGRO_RANGE: f32 = 180.;
const ATTACK_RANGE: f32 = 64.;

const PATROL_RADIUS: f32 = TILE_SIZE * 20.;

const PATROL_SPEED: f32 = 35.;

const CHASE_ACCELERATION: f32 = 150.;
const CHASE_MAX_SPEED: f32 = 250.;

const ATTACK_SPEED: f32 = 300.;
const ATTACK_DURATION: f32 = 0.4;
const WALL_DEFLECTION: f32 = 0.35;
const CLIMB_SPEED: f32 = 90.;
const CLIMB_CLEARANCE: f32 = 4.;

const FORAGE_SPEED: f32 = 90.;
const FLOWER_REACH: f32 = 20.;
const HARVEST_DURATION: f32 = 10.;
const FLOWER_COOLDOWN: f32 = 60.;
const UNLOAD_DURATION: f32 = 3.;

const GATHERER_ALTITUDE: f32 = 32.;
const PATROLLER_ALTITUDE: f32 = 48.;

const ALTITUDE_GAIN: f32 = 2.0;
const MAX_ALTITUDE_SPEED: f32 = 90.;
const MAX_PATROL_ALTITUDE: f32 = 96.;

const HIVEMIND_SEPARATION_SIZE: Vec2 = Vec2::new(1., 1.);

#[derive(Component, Default)]
pub struct Hivemind {
    pub velocity: Vec2,
    pub grounded: bool,
    pub attack_timer: f32,
    pub animation_timer: f32,

    //hive location
    pub hive_origin: Vec2,

    //idle animation
    pub idle_timer: f32,
    pub idle_origin_y: f32,
    pub idle_amplitude: f32,
    pub idle_speed: f32,
    pub has_hit_player: bool,
    pub has_bounced: bool,
   
    //for flower hunting
    pub flower_target: Option<Entity>,
    pub pollen_timer: f32,
    pub returning_to_hive: bool,
    pub unloading: bool,
    pub hive_lost: bool,
    pub pollen_particle_timer: f32,
    pub is_harvester: bool,

    pub climbing: bool,
    pub climb_direction: f32,
}

#[derive(Resource)]
pub struct HivemindSheet {
    sprite_sheet: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

#[derive(Component, Default, PartialEq, Eq, Clone, Copy)]
pub enum HivemindState {
    Idle,
    #[default]
    Patrol,
    Chase,
    Attack,
}

#[derive(Component)]
pub struct HivemindPatrol {
    pub distance_remaining: f32,
    pub direction: f32,
}

#[derive(Component)]
pub struct HiveMember {
    pub hive: Entity,
}

#[derive(Component)]
pub struct PollenParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct HivemindPlugin;
impl Plugin for HivemindPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_hivemind)
            .add_systems(
                Update,
                (
                    hivemind_ai,
                    hivemind_collision,
                    update_hivemind_sprite,
                    update_pollen_particles,
                )
                    .chain(),
            )
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

pub fn spawn_hivemind(
    commands: &mut Commands,
    sheet: &HivemindSheet,
    hive_position: Vec2,
    hive_entity: Entity,
) {
    spawn_hivemind_enemy(
        commands,
        sheet,
        Vec2::new(
            464.,
            LEVEL_BOTTOM + TILE_SIZE + HIVEMIND_SIZE.y / 2.,
        ),
        hive_position,
        hive_entity,
    );
}

pub fn spawn_hivemind_enemy(
    commands: &mut Commands,
    sheet: &HivemindSheet,
    spawn_position: Vec2,
    hive_position: Vec2,
    hive_entity: Entity,
) -> Entity {
    commands.spawn((
        Sprite::from_atlas_image(
            sheet.sprite_sheet.clone(),
            TextureAtlas {
                layout: sheet.layout.clone(),
                index: 0,
            },
        ),
        Transform::from_xyz(
            spawn_position.x,
            spawn_position.y,
            2.,
        ).with_scale(Vec3::splat(SPRITE_SCALE)),
        Hivemind {
            hive_origin: hive_position,
            idle_origin_y: spawn_position.y,
            idle_amplitude: 4.,
            idle_speed: 3.,
            is_harvester: rand::random::<f32>() < 0.5,  //half of hiveminds will focus on harvesting, other half will just patrol
            ..default()
        },
        HiveMember{hive: hive_entity,},
        HIVEMIND.bundle(),
        Hitbox(HIVEMIND_SIZE),
        HitCooldown::default(),
        HivemindPatrol {
            distance_remaining: 0.,
            direction: 1.,
        },
        HivemindState::default(),
    )).id()
}

fn overlaps(a_pos: Vec2, a_size: Vec2, b_pos: Vec2, b_size: Vec2) -> bool {
    let d = (a_pos - b_pos).abs();
    let reach = (a_size + b_size) / 2.;
    d.x < reach.x && d.y < reach.y
}

fn resolve_brick_collision(
    tf: &mut Transform,
    velocity: &mut Vec2,
    brick_pos: Vec2,
) -> bool {
    let position = tf.translation.truncate();
    let delta = position - brick_pos;

    let overlap = (HIVEMIND_SIZE + Vec2::splat(TILE_SIZE)) / 2. - delta.abs();

    //no collisions
    if overlap.x <= 0. || overlap.y <= 0. {
        return false;
    }

    if overlap.x < overlap.y {
        let direction = if delta.x >= 0. { 1. } else { -1. };
        tf.translation.x += overlap.x * direction;
        velocity.x = 0.;
        true
    } else {
        let direction = if delta.y >= 0. { 1. } else { -1. };
        tf.translation.y += overlap.y * direction;

        if velocity.y * direction < 0. {
            velocity.y = 0.;
        }
        false
    }
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

fn hivemind_ai(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<(&Transform, &Hitbox), With<Player>>,
    mut hivemind: Query<
        (
            Entity,
            &mut Transform,
            &mut Hivemind,
            &mut HivemindPatrol,
            &mut HivemindState,
            &HiveMember,
        ),
        (Without<Player>, Without<HiveFlower>),
    >,
    hives: Query<&Health, With<Hive>>,
    mut flowers: Query<
        (Entity, &Transform, &mut HiveFlower),
        (With<HiveFlower>, Without<Hivemind>),
    >,
    bricks: Query<
        &Transform,
        (With<Brick>, Without<Hivemind>),
    >,
) {
    let dt = time.delta_secs().min(1. / 30.);
    let (player_tf, player_box) = *player;
    let player_pos = player_tf.translation.truncate();

    //if ANY hivemind can see the player, alert the entire group.
    let player_spotted = hivemind.iter().any(
        |(_, tf, _, _, _, _)| {
            tf.translation.truncate().distance(player_pos) <= AGGRO_RANGE
        }
    );

    for (entity, mut tf, mut hivemind, mut patrol, mut state, hive_member) in &mut hivemind {
        let position = tf.translation.truncate();
        //stops patrol climbing if the bug is already too high.
        if *state == HivemindState::Patrol {
            let desired_altitude = if hivemind.is_harvester {
                GATHERER_ALTITUDE
            } else {
                PATROLLER_ALTITUDE
            };

            let ground_below = bricks.iter()
                .filter_map(|brick_tf| {
                    let brick_pos = brick_tf.translation.truncate();
                    let horizontal_overlap = (brick_pos.x - position.x).abs() < (HIVEMIND_SIZE.x + TILE_SIZE) / 2.;
                    let ground_top = brick_pos.y + TILE_SIZE / 2.;

                    if horizontal_overlap && ground_top <= position.y {
                        Some(ground_top)
                    } else {
                        None
                    }
                })
                .max_by(|a, b| a.total_cmp(b));

            if let Some(ground_top) = ground_below {
                let altitude = position.y - HIVEMIND_SIZE.y / 2. - ground_top;

                if altitude >= MAX_PATROL_ALTITUDE {
                    hivemind.climbing = false;
                    hivemind.climb_direction = 0.;
                    hivemind.velocity.y = 0.;
                }
            }
        }

        let to_player = player_pos - position;
        let player_distance = to_player.length();

        //checks whether the hive still exists and has health.
        let hive_alive = hives.get(hive_member.hive).is_ok_and(|health| health.current > 0.);

        //if the hive has been destroyed, aimlessly patrol \
        if !hive_alive && !hivemind.hive_lost {
            //gives up harvesting flowers
            if let Some(target) = hivemind.flower_target.take() {
                if let Ok((_, _, mut flower)) = flowers.get_mut(target) {
                    if flower.reserved_by == Some(entity) {
                        flower.reserved_by = None;
                    }
                }
            }

            //cancels any hive-related activities.
            hivemind.returning_to_hive = false;
            hivemind.unloading = false;
            hivemind.pollen_timer = 0.;

            hivemind.climbing = false;
            hivemind.climb_direction = 0.;

            //make it patrol around where it was
            hivemind.hive_origin = position;
            hivemind.idle_origin_y = position.y;

            hivemind.velocity = Vec2::ZERO;
            *state = HivemindState::Patrol;
            hivemind.hive_lost = true;
        }

        //global alert. only triggers if hive is still alive
        let should_chase = player_distance <= AGGRO_RANGE || (hive_alive && player_spotted);

        if should_chase {
            //give up harvesting and go kick some butt.
            if let Some(target) = hivemind.flower_target.take() {
                if let Ok((_, _, mut flower)) = flowers.get_mut(target) {
                    if flower.reserved_by == Some(entity) {
                        flower.reserved_by = None;
                    }
                }
            }

            //stops harvesting to kick some butt.
            hivemind.returning_to_hive = false;
            hivemind.unloading = false;
            hivemind.pollen_timer = 0.;

            if hivemind.climbing {
                hivemind.climbing = false;
                hivemind.climb_direction = 0.;
                hivemind.velocity = Vec2::ZERO;
            }

            //"Come on, Hiveminds, we have some butt to kick" -- this hivemind enemy, probably.
            *state = HivemindState::Chase;
        }

        let mut should_bob = true;

        //cool particle effect when it has pollen
        let has_pollen = (hivemind.flower_target.is_some() && hivemind.pollen_timer > 0.) || hivemind.returning_to_hive;

        if has_pollen {
            hivemind.pollen_particle_timer += dt;

            if hivemind.pollen_particle_timer >= 0.12 {
                hivemind.pollen_particle_timer = 0.;

                let wobble = (hivemind.animation_timer * 8.).sin();

                //spawns a little purple pixel to shoot out of the bug
                commands.spawn((
                    Sprite::from_color(
                        Color::srgba(0.7, 0.25, 0.95, 0.9),
                        Vec2::splat(3.),
                    ),
                    Transform::from_translation(
                        tf.translation + Vec3::new(wobble * 4., -3., 0.2),
                    ),
                    PollenParticle {
                        velocity: Vec2::new(wobble * 12., 18.),
                        lifetime: 0.5,
                        max_lifetime: 0.5,
                    },
                ));
            }
        } else {
            hivemind.pollen_particle_timer = 0.;
        }

        if !hivemind.climbing {
            match *state {
                //IDLE
                HivemindState::Idle => {
                    hivemind.velocity = Vec2::ZERO;
                    //has a flower to harvest
                    if let Some(target) = hivemind.flower_target {
                        if let Ok((_, flower_tf, mut flower)) = flowers.get_mut(target) {
                            if flower.reserved_by == Some(entity) {
                                let flower_pos = flower_tf.translation.truncate();
                                let distance = flower_pos - position;

                                if distance.length() <= 2. {
                                    hivemind.velocity = Vec2::ZERO;

                                    tf.translation.x = flower_pos.x;
                                    tf.translation.y = flower_pos.y;

                                    hivemind.idle_origin_y = flower_pos.y;

                                    hivemind.pollen_timer += dt;

                                    if hivemind.pollen_timer >= HARVEST_DURATION {
                                        flower.cooldown_remaining = FLOWER_COOLDOWN;
                                        flower.reserved_by = None;
                                        hivemind.flower_target = None;
                                        hivemind.returning_to_hive = true;
                                        hivemind.pollen_timer = 0.;
                                        should_bob = false;
                                    }
                                } else {
                                    hivemind.velocity = distance.normalize_or_zero() * FORAGE_SPEED;
                                    should_bob = false;
                                }
                            } else {
                                hivemind.flower_target = None;
                                *state = HivemindState::Patrol;
                            }
                        } else {
                            //flower no longer exists, for whatever reason.
                            hivemind.flower_target = None;
                            *state = HivemindState::Patrol;
                        }
                    }
                    else if hivemind.returning_to_hive { //returns to hive after harvesting
                        let distance = hivemind.hive_origin - position;

                        if distance.length() <= FLOWER_REACH {
                            hivemind.velocity = Vec2::ZERO;
                            hivemind.returning_to_hive = false;
                            hivemind.unloading = true;
                            hivemind.pollen_timer = 0.;
                            hivemind.idle_origin_y = hivemind.hive_origin.y;
                        } else {
                            hivemind.velocity = distance.normalize_or_zero() * FORAGE_SPEED;
                            should_bob = false;
                        }
                    }
                    else if hivemind.unloading {    //unloads pollen
                        hivemind.velocity = Vec2::ZERO;
                        hivemind.pollen_timer += dt;

                        if hivemind.pollen_timer >= UNLOAD_DURATION {
                            hivemind.unloading = false;
                            hivemind.pollen_timer = 0.;
                            *state = HivemindState::Patrol;
                        }
                    }
                    else {  //nothing better to do
                        hivemind.velocity = Vec2::ZERO;
                    }
                }
                //PATROL
                HivemindState::Patrol => {
                    should_bob = false;
                    //player detection takes priority
                    if player_distance <= AGGRO_RANGE {
                        *state = HivemindState::Chase;
                        hivemind.velocity = Vec2::ZERO;
                    } else {
                        //goes for nearest flower, if available
                        let nearest_flower = if hivemind.is_harvester {
                            let mut nearest: Option<(Entity, Vec2, f32)> = None;

                            for (flower_entity, flower_tf, flower) in &mut flowers {
                                if flower.cooldown_remaining > 0. || flower.reserved_by.is_some(){
                                    continue;
                                }

                                let flower_pos = flower_tf.translation.truncate();
                                let distance = position.distance(flower_pos);

                                if distance <= AGGRO_RANGE {
                                    if nearest.is_none() || distance < nearest.unwrap().2 {
                                        nearest = Some((
                                            flower_entity,
                                            flower_pos,
                                            distance,
                                        ));
                                    }
                                }
                            }
                            nearest
                        } else {
                            None
                        };

                        //if there's actually a flower to take
                        if let Some((flower_entity, _, _)) = nearest_flower {
                            if let Ok((_, _, mut flower)) = flowers.get_mut(flower_entity) {
                                flower.reserved_by = Some(entity);
                                hivemind.flower_target = Some(flower_entity);
                                hivemind.pollen_timer = 0.;
                                hivemind.returning_to_hive = false;
                                hivemind.unloading = false;
                                hivemind.velocity = Vec2::ZERO;
                                *state = HivemindState::Idle;
                            }
                        } else {
                            //no flowers nearby; continue patrolling
                            let left_bound = hivemind.hive_origin.x - PATROL_RADIUS;
                            let right_bound = hivemind.hive_origin.x + PATROL_RADIUS;

                            if position.x <= left_bound {
                                patrol.direction = 1.;
                            } else if position.x >= right_bound {
                                patrol.direction = -1.;
                            }

                            //looks ahead for walls, measuring from the edges of the bug.
                            let obstacle_ahead = bricks.iter().any(|brick_tf| {
                                let brick_pos = brick_tf.translation.truncate();

                                // Distance from the bug's center to the brick's center.
                                let center_ahead = (brick_pos.x - position.x) * patrol.direction;
                                //converts center distance into actual edge-to-edge distance.
                                let horizontal_gap = center_ahead - (HIVEMIND_SIZE.x + TILE_SIZE) / 2.;
                                let vertical_overlap = (brick_pos.y - position.y).abs() < (HIVEMIND_SIZE.y + TILE_SIZE) / 2.;

                                center_ahead > 0. && horizontal_gap <= 25. && vertical_overlap
                            });

                            if obstacle_ahead {
                                hivemind.climbing = true;
                                hivemind.climb_direction = patrol.direction;
                                hivemind.velocity = Vec2::ZERO;
                            } else {
                                //continues patrolling horizontally.
                                hivemind.velocity.x = patrol.direction * PATROL_SPEED;

                                //chooses the desired altitude based on the bug's role.
                                let desired_altitude = if hivemind.is_harvester {
                                    GATHERER_ALTITUDE
                                } else {
                                    PATROLLER_ALTITUDE
                                };

                                //finds the highest brick surface directly beneath the bug.
                                let bug_bottom = position.y - HIVEMIND_SIZE.y / 2.;
                                let ground_below = bricks.iter()
                                    .filter_map(|brick_tf| {
                                        let brick_pos = brick_tf.translation.truncate();
                                        let horizontal_overlap = (brick_pos.x - position.x).abs() < (HIVEMIND_SIZE.x + TILE_SIZE) / 2.;
                                        let ground_top = brick_pos.y + TILE_SIZE / 2.;
                                        let beneath_bug = ground_top <= bug_bottom + 4.;

                                        if horizontal_overlap && beneath_bug {
                                            Some(ground_top)
                                        } else {
                                            None
                                        }
                                    })
                                    .max_by(|a, b| a.total_cmp(b));

                                if let Some(ground_top) = ground_below {
                                    let target_y = ground_top + HIVEMIND_SIZE.y / 2. + desired_altitude;
                                    let altitude_error = target_y - position.y;
                                    let current_altitude = position.y - HIVEMIND_SIZE.y / 2. - ground_top;

                                    if current_altitude >= MAX_PATROL_ALTITUDE {
                                        // Never allow upward movement beyond the altitude limit.
                                        hivemind.velocity.y = altitude_error.min(0.).max(-MAX_ALTITUDE_SPEED);
                                    } else {
                                        hivemind.velocity.y = (altitude_error * ALTITUDE_GAIN).clamp(-MAX_ALTITUDE_SPEED, MAX_ALTITUDE_SPEED);
                                    }
                                } else {
                                    //stop flying into the sky, dang it!
                                    hivemind.velocity.y = 0.;
                                }
                            }

                            patrol.distance_remaining =
                                if patrol.direction > 0. {
                                    right_bound - position.x
                                } else {
                                    position.x - left_bound
                                };
                        }
                    }
                }

                //CHASE
                HivemindState::Chase => {
                    should_bob = false;

                    //switches to attack if the player is close
                    if player_distance <= ATTACK_RANGE {
                        *state = HivemindState::Attack;

                        hivemind.attack_timer = 0.;
                        hivemind.has_hit_player = false;
                        hivemind.has_bounced = false;

                        hivemind.velocity = to_player.normalize_or_zero() * ATTACK_SPEED;
                    } else {
                        let direction = to_player.normalize_or_zero();
                        hivemind.velocity += direction * CHASE_ACCELERATION * dt;
                        hivemind.velocity = hivemind.velocity.clamp_length_max(CHASE_MAX_SPEED);
                    }
                }

                //ATTACK
                HivemindState::Attack => {
                    should_bob = false;

                    hivemind.attack_timer += dt;

                    if !hivemind.has_hit_player
                        && player_distance <= ATTACK_RANGE
                    {
                        hivemind.velocity = -hivemind.velocity;
                        hivemind.has_hit_player = true;
                    }

                    hivemind.velocity *= (-7. * dt).exp();

                    if hivemind.attack_timer >= ATTACK_DURATION {
                        *state = HivemindState::Chase;
                    }
                }
            }
        }
        
        //so the lil guy bobs up and down
        if should_bob && !hivemind.climbing {
            if *state != HivemindState::Chase {
                hivemind.idle_amplitude = 4.;
                hivemind.idle_speed = 3.;
            }

            hivemind.idle_timer += dt;
            tf.translation.y = hivemind.idle_origin_y + (hivemind.idle_timer * hivemind.idle_speed).sin() * hivemind.idle_amplitude;
        }
    }
}

fn hivemind_collision(
    time: Res<Time>,
    mut hiveminds: ParamSet<(
        Query<(Entity, &Transform), With<Hivemind>>,
        Query<(
            Entity,
            &mut Transform,
            &mut Hivemind,
            &mut HivemindPatrol,
            &HivemindState,
        ), With<Hivemind>>,
    )>,
    bricks: Query<&Transform, (With<Brick>, Without<Hivemind>)>,
) {
    let dt = time.delta_secs().min(1. / 30.);

    //positions of all hiveminds
    let positions: Vec<(Entity, Vec2)> = hiveminds
        .p0()
        .iter()
        .map(|(entity, tf)| {
            (entity, tf.translation.truncate())
        })
        .collect();

    //positions of all bricks
    let brick_positions: Vec<Vec2> = bricks
        .iter()
        .map(|tf| tf.translation.truncate())
        .collect();

    for (entity, mut tf, mut hivemind, mut patrol, state) in hiveminds.p1().iter_mut() {
        handle_hivemind_collisions(
            entity,
            &mut tf,
            &mut hivemind,
            &mut patrol,
            state,
            &positions,
            &brick_positions,
            dt,
        );
    }
}

//helper to handle colissions
fn handle_hivemind_collisions(
    entity: Entity,
    tf: &mut Transform,
    hivemind: &mut Hivemind,
    patrol: &mut HivemindPatrol,
    state: &HivemindState,
    positions: &[(Entity, Vec2)],
    brick_positions: &[Vec2],
    dt: f32,
) {
    //remembers the intended horizontal direction before collisions.
    let horizontal_direction = hivemind.velocity.x.signum();
    let was_moving_horizontally = horizontal_direction != 0.;

    tf.translation.x += hivemind.velocity.x * dt;
    tf.translation.y += hivemind.velocity.y * dt;

    //separates overlapping hiveminds.
    for (other_entity, other_pos) in positions {
        if *other_entity == entity {
            continue;
        }

        let position = tf.translation.truncate();
        let delta = position - *other_pos;

        let overlap = HIVEMIND_SEPARATION_SIZE - delta.abs();

        if overlap.x > 0. && overlap.y > 0. {
            if overlap.x < overlap.y {
                let direction = if delta.x >= 0. { 1. } else { -1. };
                tf.translation.x += overlap.x * direction;
            } else {
                let direction = if delta.y >= 0. { 1. } else { -1. };
                tf.translation.y += overlap.y * direction;
            }
        }
    }

    //checks collisions against bricks.
    let mut blocked_horizontally = false;

    for brick_pos in brick_positions {
        if resolve_brick_collision(
            tf,
            &mut hivemind.velocity,
            *brick_pos,
        ) {
            blocked_horizontally = true;
        }
    }

    //starts climbing when a brick blocks horizontal movement.
    if !hivemind.climbing
        && *state == HivemindState::Patrol
        && was_moving_horizontally
        && blocked_horizontally
    {
        hivemind.climbing = true;
        hivemind.climb_direction = horizontal_direction;
    }

    //handles continued climbing.
    if hivemind.climbing {
        let climb_amount = CLIMB_SPEED * dt;
        let climb_position = tf.translation.truncate() + Vec2::Y * climb_amount;

        let blocked_above = brick_positions.iter().any(|brick_pos| {
            overlaps(
                climb_position,
                HIVEMIND_SIZE,
                *brick_pos,
                Vec2::splat(TILE_SIZE),
            )
        });

        if !blocked_above {
            tf.translation.y += climb_amount;

            let bug_bottom = tf.translation.y - HIVEMIND_SIZE.y / 2.;
            let mut above_obstacle = true;

            for brick_pos in brick_positions {
                let horizontal_gap = (brick_pos.x - tf.translation.x) * hivemind.climb_direction;
                let brick_top = brick_pos.y + TILE_SIZE / 2.;

                if horizontal_gap > 0.
                    && horizontal_gap < (HIVEMIND_SIZE.x + TILE_SIZE) / 2. + CLIMB_CLEARANCE
                    && bug_bottom <= brick_top + CLIMB_CLEARANCE
                {
                    above_obstacle = false;
                    break;
                }
            }

            if above_obstacle {
                tf.translation.x += hivemind.climb_direction * PATROL_SPEED * dt;

                //checks whether we've passed the obstacle.
                let obstacle_still_ahead = brick_positions.iter().any(|brick_pos| {
                    let horizontal_gap = (brick_pos.x - tf.translation.x) * hivemind.climb_direction;

                    horizontal_gap > 0. && horizontal_gap < (HIVEMIND_SIZE.x + TILE_SIZE) / 2.
                });

                if !obstacle_still_ahead {
                    hivemind.climbing = false;
                    hivemind.climb_direction = 0.;
                }
            }

            hivemind.velocity = Vec2::ZERO;
        } else {
            hivemind.climbing = false;

            //reverses patrol direction so it doesn't immediately attempt the same impossible climb again.
            patrol.direction = -hivemind.climb_direction;
            hivemind.velocity.x = patrol.direction * PATROL_SPEED;
        }
    }
}

fn update_pollen_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(
        Entity,
        &mut Transform,
        &mut Sprite,
        &mut PollenParticle,
    )>,
) {
    let dt = time.delta_secs();

    for (entity, mut tf, mut sprite, mut particle) in &mut particles {
        particle.lifetime -= dt;

        if particle.lifetime <= 0. {
            commands.entity(entity).despawn();
            continue;
        }

        //moves the particle.
        tf.translation.x += particle.velocity.x * dt;
        tf.translation.y += particle.velocity.y * dt;

        //gradually fade it out.
        let alpha = particle.lifetime / particle.max_lifetime;

        sprite.color = Color::srgba(0.7, 0.25, 0.95, alpha * 0.9);
    }
}