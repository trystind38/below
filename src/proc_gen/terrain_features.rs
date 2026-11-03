use bevy::prelude::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
 
use crate::{
    GameState, LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_LEN, TILE_SIZE,
    level::{BrickImage, spawn_tile},
};
 
const SEED: u64 = 7;
const COLS: i32 = (LEVEL_LEN / TILE_SIZE) as i32; // 160
const START_COL: i32 = 34;
const GROUND_HEIGHT: i32 = 1; // floor is one tile tall
 
// '#' = tile, '.' = empty. First line is the top row.
// lift = how many rows above the ground the piece's bottom row sits.
struct Piece {
    rows: &'static [&'static str],
    lift: i32,
}
 
const PIECES: [Piece; 3] = [
    Piece { rows: &[".##.", "####"], lift: 0 },               // small hill
    Piece { rows: &["..##..", ".####.", "######"], lift: 0 }, // big hill
    Piece { rows: &["#####"], lift: 3 },                      // floating platform
];
 
fn tile_pos(col: i32, row: i32) -> Vec3 {
    Vec3::new(
        LEVEL_LEFT + TILE_SIZE / 2. + col as f32 * TILE_SIZE,
        LEVEL_BOTTOM + TILE_SIZE / 2. + row as f32 * TILE_SIZE,
        1.,
    )
}
 
// Pure function: no Bevy, just returns (col, row) for every tile to spawn.
fn generate_features(seed: u64) -> Vec<IVec2> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut ground = vec![GROUND_HEIGHT; COLS as usize]; // first free row in each column
    let mut tiles = Vec::new();
 
    let mut col = START_COL;
    while col < COLS {
        col += rng.gen_range(4..10); // flat ground before the next piece
 
        let piece = &PIECES[rng.gen_range(0..PIECES.len())];
        let width = piece.rows[0].len() as i32;
 
        // stop if it would run off the end of the level
        if col + width > COLS {
            break;
        }
 
        // only place it on flat ground
        let base = ground[col as usize];
        let flat = (col..col + width).all(|c| ground[c as usize] == base);
        if !flat {
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
 
fn spawn_features(mut commands: Commands, brick_image: Res<BrickImage>) {
    for t in generate_features(SEED) {
        spawn_tile(&mut commands, &brick_image, tile_pos(t.x, t.y));
    }
}
 
pub struct TerrainFeaturesPlugin;
impl Plugin for TerrainFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_features);
    }
}