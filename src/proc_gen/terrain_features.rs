use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::{
    GameState, LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_LEN, TILE_SIZE,
    level::{BrickImage, spawn_tile},
    stats::Hitbox,
    proc_gen::room_generation::{RoomMap, TileType},
};

// ---------- demo level constants ----------
const SEED: u64 = 1;
const COLS: i32 = (LEVEL_LEN / TILE_SIZE) as i32; // 160
const START_COL: i32 = 34;
const GROUND_HEIGHT: i32 = 1; // floor is one tile tall

// ---------- generator tuning ----------
const HEADROOM: i32 = 2;         // rows the player needs above a piece (assumes player <= 2 tiles tall)
const MIN_RUN: i32 = 6;          // ignore floors shorter than this
const EDGE_MARGIN: usize = 2;    // keep run ends clear (doorways, ledges)
const MAX_OPEN_HEIGHT: i32 = 15; // above the tallest room (14), so only vertical shafts get skipped

// '#' = tile, '.' = empty. First line is the top row.
// lift = how many rows above the ground the piece's bottom row sits.
struct Piece {
    rows: &'static [&'static str],
    lift: i32,
}

const PIECES: [Piece; 4] = [
    Piece { rows: &[".##.", "####"], lift: 0 },               // small hill
    Piece { rows: &["..##..", ".####.", "######"], lift: 0 }, // big hill
    Piece { rows: &["#####"], lift: 3 },                      // floating platform
    Piece { rows: &["###"], lift: 0 },                        // low step (fits 3-tall hallways)
];

// A horizontal stretch of terrain the generator can decorate.
// All rows are relative to the region's own floor (row 0 = first free row).
pub struct Region {
    pub ground: Vec<i32>,  // first free row in each column
    pub ceiling: Vec<i32>, // first solid row above each column (i32::MAX if open)
    pub skip: Vec<bool>,   // columns to keep clear (doorways, shafts, spawn)
}

impl Region {
    pub fn flat(cols: usize, ground_height: i32) -> Self {
        Self {
            ground: vec![ground_height; cols],
            ceiling: vec![i32::MAX; cols],
            skip: vec![false; cols],
        }
    }

    pub fn clear_cols(mut self, range: std::ops::Range<usize>) -> Self {
        for c in range {
            self.skip[c] = true;
        }
        self
    }

    // Keep terrain features away from a world-space x position (enemy/player spawns).
    pub fn clear_world_x(&mut self, world_x: f32, origin_x: f32, radius_tiles: i32) {
        let col = ((world_x - origin_x) / TILE_SIZE).floor() as i32;
        for c in (col - radius_tiles)..=(col + radius_tiles) {
            if c >= 0 && (c as usize) < self.skip.len() {
                self.skip[c as usize] = true;
            }
        }
    }
}

// Pure function: no Bevy, returns (col, row) for every tile to place,
// relative to the region's bottom-left.
pub fn generate_features(seed: u64, region: &Region) -> Vec<IVec2> {
    let cols = region.ground.len() as i32;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut ground = region.ground.clone();
    let mut tiles = Vec::new();

    let mut col = 0;
    while col < cols {
        col += rng.gen_range(4..10); // flat ground before the next piece

        let piece = &PIECES[rng.gen_range(0..PIECES.len())];
        let width = piece.rows[0].len() as i32;

        // stop if it would run off the end of the region
        if col + width > cols {
            break;
        }

        // only place on flat, unblocked ground with enough headroom
        let base = ground[col as usize];
        let top = base + piece.lift + piece.rows.len() as i32;
        let fits = (col..col + width).all(|c| {
            let c = c as usize;
            ground[c] == base && !region.skip[c] && top + HEADROOM <= region.ceiling[c]
        });
        if !fits {
            continue;
        }

        // stamp the piece
        for (line, text) in piece.rows.iter().enumerate() {
            for (x, ch) in text.chars().enumerate() {
                if ch == '#' {
                    let row = base + piece.lift + (piece.rows.len() - 1 - line) as i32;
                    tiles.push(IVec2::new(col + x as i32, row));

                    // hills raise the ground; floating platforms don't
                    if piece.lift == 0 {
                        let c = (col + x as i32) as usize;
                        ground[c] = ground[c].max(row + 1);
                    }
                }
            }
        }

        col += width;
    }

    tiles
}

// Finds every flat floor run in a tile grid (y increases upward, out of bounds = solid).
// Returns the first free cell of each run plus a Region describing it.
pub fn regions_from_grid(
    width: i32,
    height: i32,
    is_solid: impl Fn(i32, i32) -> bool,
) -> Vec<(IVec2, Region)> {
    let solid = |x: i32, y: i32| x < 0 || y < 0 || x >= width || y >= height || is_solid(x, y);
    let mut out = Vec::new();

    for y in 1..height {
        let mut x = 0;
        while x < width {
            // floor cell = empty with solid directly below
            if solid(x, y) || !solid(x, y - 1) {
                x += 1;
                continue;
            }
            let start = x;
            while x < width && !solid(x, y) && solid(x, y - 1) {
                x += 1;
            }
            let len = x - start;
            if len < MIN_RUN {
                continue;
            }

            let mut region = Region::flat(len as usize, 0);
            for i in 0..len {
                let cx = start + i;
                let mut top = y + 1;
                while !solid(cx, top) {
                    top += 1;
                }
                region.ceiling[i as usize] = top - y;
                if top - y > MAX_OPEN_HEIGHT {
                    region.skip[i as usize] = true;
                }
            }
            for i in 0..EDGE_MARGIN.min(len as usize) {
                region.skip[i] = true;
                region.skip[len as usize - 1 - i] = true;
            }
            out.push((IVec2::new(start, y), region));
        }
    }
    out
}

// ---------- dungeon: write features into the RoomMap as wall tiles ----------

pub fn decorate_map(map: &mut RoomMap, seed: u64) {
    let (w, h) = (map.width as i32, map.height as i32);
    let mut new_tiles: Vec<IVec2> = Vec::new();

    let regions = regions_from_grid(w, h, |x, y| {
        matches!(map.tiles[y as usize * map.width + x as usize], TileType::Wall)
    });

    for (cell, region) in regions {
        // different seed per region so rooms don't all get the same hills
        let s = seed ^ (((cell.x as u64) << 32) | cell.y as u64).wrapping_mul(0x9E3779B97F4A7C15);
        new_tiles.extend(generate_features(s, &region).into_iter().map(|t| t + cell));
    }

    // write after scanning so one region's pieces can't change what another sees
    for t in new_tiles {
        map.tiles[t.y as usize * map.width + t.x as usize] = TileType::Wall;
    }
}

// ---------- demo level: spawn tile entities ----------

fn tile_pos(origin: Vec2, col: i32, row: i32) -> Vec3 {
    Vec3::new(
        origin.x + TILE_SIZE / 2. + col as f32 * TILE_SIZE,
        origin.y + TILE_SIZE / 2. + row as f32 * TILE_SIZE,
        1.,
    )
}

pub fn spawn_region(commands: &mut Commands, brick: &Res<BrickImage>, origin: Vec2, tiles: &[IVec2]) {
    for t in tiles {
        spawn_tile(commands, brick, tile_pos(origin, t.x, t.y));
    }
}

// Must run AFTER enemies/player are spawned so their positions can be kept clear.
fn spawn_features(
    mut commands: Commands,
    brick_image: Res<BrickImage>,
    actors: Query<&Transform, With<Hitbox>>,
) {
    let origin = Vec2::new(LEVEL_LEFT, LEVEL_BOTTOM);
    let mut region = Region::flat(COLS as usize, GROUND_HEIGHT).clear_cols(0..START_COL as usize);
    for tf in &actors {
        region.clear_world_x(tf.translation.x, origin.x, 3);
    }
    let tiles = generate_features(SEED, &region);
    spawn_region(&mut commands, &brick_image, origin, &tiles);
}

pub struct TerrainFeaturesPlugin;
impl Plugin for TerrainFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_features);
    }
}