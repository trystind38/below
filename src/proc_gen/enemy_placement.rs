// Picks where enemies go in a finished dungeon (after hallways, platforms and terrain).
// Pure function: no Bevy entities, just tile positions, so it can be tested and previewed.

use bevy::math::IVec2;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::proc_gen::{
    hallway_gen::{HorzHallway, VertHallway},
    room_generation::{RoomMap, TileType},
};

// ---------- tuning ----------
const TEETH_CHANCE: f64 = 0.6; // chance each hallway gets a Teeth
const TEETH_SPACING: i32 = 6; // min tiles between two Teeth (hallways can overlap)
const CEILING_CLEARANCE: i32 = 4; // open rows under a ceiling Teeth: 2 for it, 2 for the player
const GERALD_SPACING: i32 = 5; // min tiles between two Geralds in a room
const WIDE_ROOM: f64 = 16.; // rooms this wide can get a second Gerald
const EXTRA_GERALD_CHANCE: f64 = 0.5;
const ROOM_MARGIN: i32 = 1; // keep Teeth this far outside room rectangles
// Mixed into the dungeon seed so enemy rolls don't share a stream with room/hallway rolls
const SEED_SALT: u64 = 0x7EE7_4E1D_E5A1_7000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeethFacing {
    Down,  // hanging from a horizontal hallway's ceiling
    Left,  // on a vertical shaft's right wall
    Right, // on a vertical shaft's left wall
}

// `tile` is the empty tile touching the ceiling/wall the Teeth is attached to
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeethSpawn {
    pub tile: IVec2,
    pub facing: TeethFacing,
}

// `tile` is the empty tile Gerald stands in (solid floor directly below it)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeraldSpawn {
    pub tile: IVec2,
}

#[derive(Default)]
pub struct EnemySpawns {
    pub teeth: Vec<TeethSpawn>,
    pub geralds: Vec<GeraldSpawn>,
}

pub fn place_enemies(
    map: &RoomMap,
    horz_hallways: &[HorzHallway],
    vert_hallways: &[VertHallway],
    start_room: Option<usize>,
    seed: u64,
) -> EnemySpawns {
    let mut rng = StdRng::seed_from_u64(seed ^ SEED_SALT);
    let mut spawns = EnemySpawns::default();

    // Upside-down Teeth on horizontal hallway ceilings
    for hallway in horz_hallways {
        let spots: Vec<IVec2> = (hallway.x_start..=hallway.x_end)
            .filter(|&x| is_ceiling_spot(map, x, hallway.y_top))
            .map(|x| IVec2::new(x, hallway.y_top))
            .collect();
        try_place_teeth(&mut rng, &spots, TeethFacing::Down, &mut spawns.teeth);
    }

    // Sideways Teeth on vertical shaft walls, one random side per shaft
    for shaft in vert_hallways {
        let (side, facing) = if rng.gen_bool(0.5) {
            (-1, TeethFacing::Right)
        } else {
            (1, TeethFacing::Left)
        };
        let half = shaft.width / 2;
        let x = shaft.x + side * half; // the open column against the wall

        let spots: Vec<IVec2> = (shaft.y_start..=shaft.y_end)
            .filter(|&y| is_wall_spot(map, shaft, side, y))
            .map(|y| IVec2::new(x, y))
            .collect();
        try_place_teeth(&mut rng, &spots, facing, &mut spawns.teeth);
    }

    // Geralds on room floors
    for (i, room) in map.rooms.iter().enumerate() {
        if Some(i) == start_room {
            continue;
        }

        let x_min = room.x as i32 + 1;
        let x_max = (room.x + room.width) as i32 - 2;
        let y_min = room.y as i32;
        let y_max = (room.y + room.height) as i32 - 1;

        let spots: Vec<IVec2> = (y_min..=y_max)
            .flat_map(|y| (x_min..=x_max).map(move |x| IVec2::new(x, y)))
            .filter(|t| is_floor_spot(map, t.x, t.y))
            .collect();

        let count = if room.width >= WIDE_ROOM && rng.gen_bool(EXTRA_GERALD_CHANCE) {
            2
        } else {
            1
        };
        let mut placed: Vec<IVec2> = Vec::new();
        for _ in 0..count {
            if let Some(tile) = pick_spaced(&mut rng, &spots, &placed, GERALD_SPACING) {
                placed.push(tile);
                spawns.geralds.push(GeraldSpawn { tile });
            }
        }
    }

    spawns
}

fn try_place_teeth(
    rng: &mut StdRng,
    spots: &[IVec2],
    facing: TeethFacing,
    teeth: &mut Vec<TeethSpawn>,
) {
    if spots.is_empty() || !rng.gen_bool(TEETH_CHANCE) {
        return;
    }
    let taken: Vec<IVec2> = teeth.iter().map(|t| t.tile).collect();
    if let Some(tile) = pick_spaced(rng, spots, &taken, TEETH_SPACING) {
        teeth.push(TeethSpawn { tile, facing });
    }
}

// Random spot that isn't within `spacing` tiles of anything already taken
fn pick_spaced(rng: &mut StdRng, spots: &[IVec2], taken: &[IVec2], spacing: i32) -> Option<IVec2> {
    let free: Vec<IVec2> = spots
        .iter()
        .copied()
        .filter(|s| taken.iter().all(|t| (*s - *t).abs().max_element() >= spacing))
        .collect();
    if free.is_empty() {
        None
    } else {
        Some(free[rng.gen_range(0..free.len())])
    }
}

// Out of bounds counts as solid
fn solid(map: &RoomMap, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32 {
        return true;
    }
    matches!(map.tiles[y as usize * map.width + x as usize], TileType::Wall)
}

fn in_any_room(map: &RoomMap, x: i32, y: i32, margin: i32) -> bool {
    map.rooms.iter().any(|r| {
        x >= r.x as i32 - margin
            && x < (r.x + r.width) as i32 + margin
            && y >= r.y as i32 - margin
            && y < (r.y + r.height) as i32 + margin
    })
}

// Hallways are carved center to center, so part of each one runs through rooms and junctions.
// Only accept tiles with a real flat ceiling overhead and enough air below for the player.
fn is_ceiling_spot(map: &RoomMap, x: i32, y_top: i32) -> bool {
    if in_any_room(map, x, y_top, ROOM_MARGIN) {
        return false;
    }
    // Flat ceiling over this column and both neighbours
    let flat_ceiling = (x - 1..=x + 1).all(|cx| solid(map, cx, y_top + 1) && !solid(map, cx, y_top));
    if !flat_ceiling {
        return false;
    }
    // Terrain steps shrink the gap, so measure the actual open rows down to the floor
    (0..CEILING_CLEARANCE).all(|dy| !solid(map, x, y_top - dy))
}

// `side` is -1 for the left wall, 1 for the right wall
fn is_wall_spot(map: &RoomMap, shaft: &VertHallway, side: i32, y: i32) -> bool {
    let half = shaft.width / 2;
    let wall_x = shaft.x + side * (half + 1);
    let open_x = shaft.x + side * half;

    if in_any_room(map, open_x, y, ROOM_MARGIN) {
        return false;
    }
    // Solid wall behind it (not a junction opening)...
    let wall = (y - 1..=y + 1).all(|wy| solid(map, wall_x, wy));
    // ...and the whole shaft clear around it, which keeps it off platforms
    let clear = (y - 1..=y + 1)
        .all(|cy| (shaft.x - half..=shaft.x + half).all(|cx| !solid(map, cx, cy)));
    wall && clear
}

// Empty tile with a flat floor under it and its neighbours, and headroom above
fn is_floor_spot(map: &RoomMap, x: i32, y: i32) -> bool {
    (x - 1..=x + 1).all(|cx| solid(map, cx, y - 1) && !solid(map, cx, y))
        && !solid(map, x, y + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proc_gen::dungeon_generation::generate_dungeon;

    const SEEDS: [u64; 6] = [1, 2, 3, 42, 1234, 98765];

    fn spawns_for(seed: u64) -> (crate::proc_gen::dungeon_generation::DungeonLayout, EnemySpawns) {
        let d = generate_dungeon(seed);
        let s = place_enemies(&d.map, &d.horz_hallways, &d.vert_hallways, Some(0), seed);
        (d, s)
    }

    #[test]
    fn same_seed_same_enemies() {
        for seed in SEEDS {
            let (_, a) = spawns_for(seed);
            let (_, b) = spawns_for(seed);
            assert_eq!(a.teeth, b.teeth);
            assert_eq!(a.geralds, b.geralds);
        }
    }

    #[test]
    fn teeth_are_attached_to_something() {
        for seed in SEEDS {
            let (d, s) = spawns_for(seed);
            for t in &s.teeth {
                let (x, y) = (t.tile.x, t.tile.y);
                assert!(!solid(&d.map, x, y), "seed {seed}: teeth inside a wall at {x},{y}");
                let anchor = match t.facing {
                    TeethFacing::Down => solid(&d.map, x, y + 1),
                    TeethFacing::Right => solid(&d.map, x - 1, y),
                    TeethFacing::Left => solid(&d.map, x + 1, y),
                };
                assert!(anchor, "seed {seed}: teeth at {x},{y} isn't on a wall");
                assert!(!in_any_room(&d.map, x, y, 0), "seed {seed}: teeth inside a room");
            }
        }
    }

    #[test]
    fn every_teeth_facing_gets_used() {
        let (mut down, mut sideways, mut geralds) = (0, 0, 0);
        for seed in 0..50 {
            let (_, s) = spawns_for(seed);
            down += s.teeth.iter().filter(|t| t.facing == TeethFacing::Down).count();
            sideways += s.teeth.iter().filter(|t| t.facing != TeethFacing::Down).count();
            geralds += s.geralds.len();
        }
        println!("50 seeds: {down} ceiling teeth, {sideways} wall teeth, {geralds} geralds");
        assert!(down > 0 && sideways > 0);
    }

    #[test]
    fn geralds_stand_on_floor_outside_start_room() {
        for seed in SEEDS {
            let (d, s) = spawns_for(seed);
            assert!(!s.geralds.is_empty(), "seed {seed}: no geralds");
            let start = &d.map.rooms[0];
            for g in &s.geralds {
                let (x, y) = (g.tile.x, g.tile.y);
                assert!(!solid(&d.map, x, y) && solid(&d.map, x, y - 1));
                let in_start = x >= start.x as i32
                    && x < (start.x + start.width) as i32
                    && y >= start.y as i32
                    && y < (start.y + start.height) as i32;
                assert!(!in_start, "seed {seed}: gerald in start room");
            }
        }
    }
}
