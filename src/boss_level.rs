use bevy::prelude::*;

use crate::{
    GameState, TILE_SIZE, WIN_H, WIN_W,
    level::{ActiveLevel, Brick, BrickImage, spawn_tile},
    player::{PLAYER_SIZE, Player},
    proc_gen::{
        enemy_placement::EnemySpawns,
        room_generation::{Room, RoomMap, TileType},
    },
};

// Keep in sync with CAMERA_SCALE in camera.rs: the arena is sized to fit exactly one screen
const VIEW_SCALE: f32 = 5. / 6.;

const DOOR_ROWS: usize = 3; // entrance height in tiles (the player is 64 tall)
const ARENA_LEFT: usize = 4; // first arena column; the doorway is the column right before it

// The door that leads here lives in level_progress.rs. This file builds the arena
// and locks the doorway behind the player.
pub struct BossLevelPlugin;
impl Plugin for BossLevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, seal_boss_door.run_if(in_state(GameState::Playing)));
    }
}

// ---------- the arena ----------

fn arena_size() -> (usize, usize) {
    (
        (WIN_W * VIEW_SCALE / TILE_SIZE).floor() as usize,
        (WIN_H * VIEW_SCALE / TILE_SIZE).floor() as usize,
    )
}

fn boss_spawn() -> Vec2 {
    // between the two entrance columns, standing on the floor
    Vec2::new(2. * TILE_SIZE, TILE_SIZE + PLAYER_SIZE.y / 2.)
}

// The doorway tiles that fill in behind the player
fn lock_tiles() -> Vec<Vec2> {
    let x = ((ARENA_LEFT - 1) as f32 + 0.5) * TILE_SIZE;
    (1..=DOOR_ROWS)
        .map(|y| Vec2::new(x, (y as f32 + 0.5) * TILE_SIZE))
        .collect()
}

fn is_boss_level(active: &ActiveLevel) -> bool {
    active.map.is_some() && active.spawn == boss_spawn()
}

// The final level: one closed room that fits on one screen, with a short entrance on the left
pub fn boss_level() -> ActiveLevel {
    let (width, height) = arena_size();
    let mut tiles = vec![TileType::Wall; width * height];

    // the arena
    for y in 1..height - 1 {
        for x in ARENA_LEFT..width - 1 {
            tiles[y * width + x] = TileType::Empty;
        }
    }
    // the entrance: columns 1 and 2 are where you arrive, column 3 is the doorway
    for y in 1..=DOOR_ROWS {
        for x in 1..ARENA_LEFT {
            tiles[y * width + x] = TileType::Empty;
        }
    }

    let arena_w = (width - 1 - ARENA_LEFT) as f64;
    let map = RoomMap {
        tiles,
        rooms: vec![Room::new(ARENA_LEFT as f64, 1., arena_w, (height - 2) as f64)],
        width,
        height,
    };

    let world_w = width as f32 * TILE_SIZE;
    ActiveLevel {
        map: Some(map),
        spawn: boss_spawn(),
        bounds_min: Vec2::ZERO,
        bounds_max: Vec2::new(world_w, height as f32 * TILE_SIZE),
        enemies: EnemySpawns::default(), // the boss comes later
        lamps: vec![
            Vec2::new(world_w * 0.35, 1.5 * TILE_SIZE),
            Vec2::new(world_w * 0.7, 1.5 * TILE_SIZE),
        ],
        orbs: Vec::new(),
    }
}

// ---------- the lock ----------

// Once the player has walked fully out of the doorway, fill it in for good
fn seal_boss_door(
    mut commands: Commands,
    active: Res<ActiveLevel>,
    brick_image: Res<BrickImage>,
    player: Single<&Transform, With<Player>>,
    bricks: Query<&Transform, With<Brick>>,
) {
    if !is_boss_level(&active) {
        return;
    }
    let tiles = lock_tiles();
    if player.translation.x < tiles[0].x + TILE_SIZE / 2. + PLAYER_SIZE.x / 2. {
        return;
    }
    // already sealed? (the doorway starts empty, so a brick there means it's filled)
    if bricks.iter().any(|t| t.translation.truncate() == tiles[0]) {
        return;
    }
    for pos in tiles {
        spawn_tile(&mut commands, &brick_image, pos.extend(1.));
    }
}