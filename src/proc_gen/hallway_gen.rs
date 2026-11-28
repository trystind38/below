use crate::proc_gen::{
    geometry::Edge,
    room_generation::{Room, RoomMap, TileType},
};

// Number of open rows from the hallway floor to the ceiling
pub const HORIZONTAL_HEIGHT: i32 = 4;
// Jump height
const MAX_HALLWAY_JUMP_TILES: i32 = 3;

// Radius of a vertical shaft: 
// two tiles on each side plus the center
const VERTICAL_WIDTH: i32 = 2;

// Tiles x_start..=x_end; y_top is the highest carved row (the ceiling sits right above it)
pub struct HorzHallway {
    pub x_start: i32,
    pub x_end: i32,
    pub y_top: i32,
}

pub struct VertHallway {
    pub x: i32,
    pub y_start: i32,
    pub y_end: i32,
    pub width: i32,
}

// Convert a room center into tile coordinates.
fn room_center(room: &Room) -> (i32, i32) {
    let (x, y) = room.center();
    (x.floor() as i32, y.floor() as i32)
}

fn choose_shaft_x(left: i32, right: i32, used_a: &[i32], used_b: &[i32]) -> i32 {
    let middle = (left + right) / 2;
    let spacing = VERTICAL_WIDTH * 2 + 2;
    let mut best_x = middle;
    let mut best_gap = 0;

    // Start at the middle and try columns farther out on either side.
    for offset in 0..=(right - left) / 2 + 1 {
        for x in [middle - offset, middle + offset] {
            if x - VERTICAL_WIDTH < left || x + VERTICAL_WIDTH > right {
                continue;
            }

            let mut gap = spacing;
            for &used_x in used_a.iter().chain(used_b) {
                gap = gap.min((x - used_x).abs());
            }

            if gap == spacing {
                return x;
            }

            if gap > best_gap {
                best_gap = gap;
                best_x = x;
            }
        }
    }

    best_x
}

// Change a wall tile into an empty tile.
fn carve_tile(map: &mut RoomMap, x: i32, y: i32) {
    // Keep the outer map boundary intact.
    if x <= 0 || y <= 0 || x >= map.width as i32 - 1 || y >= map.height as i32 - 1 {
        return;
    }

    let index = y as usize * map.width + x as usize;
    map.tiles[index] = TileType::Empty;
}

fn carve_horizontal(
    map: &mut RoomMap,
    x1: i32,
    x2: i32,
    floor_y: i32,
    horz_hallways: &mut Vec<HorzHallway>,
) {
    let x_start = x1.min(x2);
    let x_end = x1.max(x2);
    let y_top = floor_y + HORIZONTAL_HEIGHT - 1;

    for x in x_start..=x_end {
        for y in floor_y..=y_top {
            carve_tile(map, x, y);
        }
    }

    horz_hallways.push(HorzHallway {
        x_start,
        x_end,
        y_top,
    });
}

// Carve a vertical hallway between two Y coordinates.
fn carve_vertical(
    map: &mut RoomMap,
    y1: i32,
    y2: i32,
    x: i32,
    vert_hallways: &mut Vec<VertHallway>,
) {
    let hallway = VertHallway {
        x,
        y_start: y1.min(y2),
        y_end: y1.max(y2),
        width: VERTICAL_WIDTH * 2 + 1,
    };
    vert_hallways.push(hallway);
    for y in y1.min(y2)..=y1.max(y2) {
        for offset in -VERTICAL_WIDTH..=VERTICAL_WIDTH {
            carve_tile(map, x + offset, y);
        }
    }
}

// Carve hallways along the selected graph edges.
pub fn carve_hallways(map: &mut RoomMap, edges: &[Edge]) -> (Vec<VertHallway>, Vec<HorzHallway>) {
    let mut vert_hallways = Vec::new();
    let mut horz_hallways = Vec::new();

    // Each room keeps a list of the X positions of its openings
    let mut room_shafts: Vec<Vec<i32>> = vec![Vec::new(); map.rooms.len()];

    for edge in edges {
        let (Some(a), Some(b)) = (map.rooms.get(edge.a), map.rooms.get(edge.b)) else {
            continue;
        };

        let (_, ay) = room_center(a);
        let (_, by) = room_center(b);

        // Calculate the overlapping tile ranges.
        let overlap_x_start = (a.x as i32).max(b.x as i32);
        let overlap_x_end = ((a.x + a.width) as i32 - 1).min((b.x + b.width) as i32 - 1);

        if overlap_x_start <= overlap_x_end {
                let shaft_x = choose_shaft_x(
                overlap_x_start,
                overlap_x_end,
                &room_shafts[edge.a],
                &room_shafts[edge.b],
            );

            let lower_floor = a.y.min(b.y) as i32;
            let upper_floor = a.y.max(b.y) as i32;

            carve_vertical(
                map,
                lower_floor,
                upper_floor + 2,
                shaft_x,
                &mut vert_hallways,
            );
            
            room_shafts[edge.a].push(shaft_x);
            room_shafts[edge.b].push(shaft_x);
        } else {
            // Find the lower room and the side that faces the upper room.
            let (lower, upper, upper_room_id) = if a.y <= b.y {
                (a, b, edge.b)
            } else {
                (b, a, edge.a)
            };
            let lower_y = lower.y as i32;
            let upper_y = upper.y as i32;

            let (lower_edge, upper_edge) = if lower.x < upper.x {
                (
                    (lower.x + lower.width) as i32 - 1,
                    upper.x as i32,
                )
            } else {
                (
                    lower.x as i32,
                    (upper.x + upper.width) as i32 - 1,
                )
            };

            if upper_y - lower_y <= MAX_HALLWAY_JUMP_TILES {
                // Jump from inside the lower room into a straight hallway
                carve_horizontal(map, lower_edge, upper_edge, upper_y, &mut horz_hallways);
            } else {
                                let room_left = upper.x as i32;
                let room_right = (upper.x + upper.width) as i32 - 1;

                let shaft_x = choose_shaft_x(
                    room_left,
                    room_right,
                    &room_shafts[upper_room_id],
                    &[],
                );

                carve_horizontal(map, lower_edge, shaft_x, lower_y, &mut horz_hallways);
                carve_vertical(map, lower_y, upper_y + 2, shaft_x, &mut vert_hallways);

                room_shafts[upper_room_id].push(shaft_x);
            }
        }
    }
    (vert_hallways, horz_hallways)
}
