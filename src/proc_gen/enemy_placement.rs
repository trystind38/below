
use bevy::math::IVec2;
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::proc_gen::{
    hallway_gen::{HorzHallway, VertHallway},
    room_generation::{RoomMap, TileType},
};

const TEETH_CHANCE: f64 = 0.6; 
const TEETH_SPACING: i32 = 6; 
const CEILING_CLEARANCE: i32 = 4;  
const GERALD_SPACING: i32 = 5; 
const WIDE_ROOM: f64 = 16.; 
const EXTRA_GERALD_CHANCE: f64 = 0.5;
const ROOM_MARGIN: i32 = 1; 
const HIVE_CHANCE: f64 = 0.15;
const HIVE_LAMP_GAP: i32 = 3; 
const HIVE_SALT: u64 = 0x41E5_B0B0_0000_0001;
const SEED_SALT: u64 = 0x7EE7_4E1D_E5A1_7000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeethFacing {
    Down,  
    Left,  
    Right, 
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeethSpawn {
    pub tile: IVec2,
    pub facing: TeethFacing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeraldSpawn {
    pub tile: IVec2,
}

pub type AmoogSpawn = GeraldSpawn;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HiveSpawn {
    pub tile: IVec2,
}

#[derive(Default)]
pub struct EnemySpawns {
    pub teeth: Vec<TeethSpawn>,
    pub geralds: Vec<GeraldSpawn>,
    pub amoogs: Vec<AmoogSpawn>,
    pub hives: Vec<HiveSpawn>,
}

impl EnemySpawns {
    // Turn every other Gerald into an Amoog, so a room with two Geralds gets one of each
    // and single-Gerald rooms alternate. 
    pub fn swap_half_geralds_for_amoogs(&mut self) {
        let (amoogs, geralds): (Vec<_>, Vec<_>) = self
            .geralds
            .drain(..)
            .enumerate()
            .partition(|(i, _)| i % 2 == 1);
        self.geralds = geralds.into_iter().map(|(_, g)| g).collect();
        self.amoogs.extend(amoogs.into_iter().map(|(_, a)| a));
    }

    // Give a few rooms a hive. A hive room is the hive's alone
    pub fn place_hives(&mut self, map: &RoomMap, start_room: Option<usize>, seed: u64) {
        let mut rng = StdRng::seed_from_u64(seed ^ HIVE_SALT);

        for (i, room) in map.rooms.iter().enumerate() {
            if Some(i) == start_room || !rng.gen_bool(HIVE_CHANCE) {
                continue;
            }

            let x_min = room.x as i32 + 1;
            let x_max = (room.x + room.width) as i32 - 2;
            let y_min = room.y as i32;
            let y_max = (room.y + room.height) as i32 - 1;
            let center_x = (room.x + room.width / 2.) as i32;

            // Floor spot nearest the middle (so the swarm fits), but not on top of the lamp
            let spot = (y_min..=y_max)
                .flat_map(|y| (x_min..=x_max).map(move |x| IVec2::new(x, y)))
                .filter(|t| is_floor_spot(map, t.x, t.y) && (t.x - center_x).abs() >= HIVE_LAMP_GAP)
                .min_by_key(|t| ((t.x - center_x).abs(), t.y));
            let Some(tile) = spot else {
                continue;
            };

            let in_room = |t: &IVec2| {
                t.x >= room.x as i32
                    && t.x < (room.x + room.width) as i32
                    && t.y >= room.y as i32
                    && t.y < (room.y + room.height) as i32
            };
            self.geralds.retain(|g| !in_room(&g.tile));
            self.amoogs.retain(|a| !in_room(&a.tile));
            self.hives.push(HiveSpawn { tile });
        }
    }
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
        let x = shaft.x + side * half; 

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
    // Solid wall behind it 
    let wall = (y - 1..=y + 1).all(|wy| solid(map, wall_x, wy));
    let ledge_above = solid(map, open_x, y + 1) && !solid(map, open_x, y + 2);
    let opposite_open = !solid(map, shaft.x - side * half, y + 1);
    let clear = (y - 1..=y)
        .all(|cy| (shaft.x - half..=shaft.x + half).all(|cx| !solid(map, cx, cy)));
    wall && ledge_above && opposite_open && clear
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
    fn wall_teeth_sit_under_a_ledge() {
        for seed in SEEDS {
            let (d, s) = spawns_for(seed);
            for t in s.teeth.iter().filter(|t| t.facing != TeethFacing::Down) {
                let (x, y) = (t.tile.x, t.tile.y);
                assert!(solid(&d.map, x, y + 1), "seed {seed}: wall teeth at {x},{y} has no ledge above");
                assert!(!solid(&d.map, x, y - 1), "seed {seed}: wall teeth at {x},{y} is on a floor");
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
    fn half_geralds_become_amoogs() {
        for seed in SEEDS {
            let (_, mut s) = spawns_for(seed);
            let before = s.geralds.clone();
            s.swap_half_geralds_for_amoogs();
            assert_eq!(s.geralds.len() + s.amoogs.len(), before.len());
            assert_eq!(s.geralds.len(), before.len().div_ceil(2));
            // Same spots as before, just split between the two
            assert!(s.geralds.iter().chain(&s.amoogs).all(|t| before.contains(t)));
        }
    }

}

   