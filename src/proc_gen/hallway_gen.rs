use crate::proc_gen::{
    geometry::Edge,
    room_generation::{Room, RoomMap, TileType},
};

// Hallway gene constants; horizontal = 3 tiles, vertical = 5 tiles
const HORIZONTAL_WIDTH: i32 = 4;
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

// Change a wall tile into an empty tile.
fn carve_tile(map: &mut RoomMap, x: i32, y: i32) {
    // Keep the outer map boundary intact.
    if x <= 0 || y <= 0 || x >= map.width as i32 - 1 || y >= map.height as i32 - 1 {
        return;
    }

    let index = y as usize * map.width + x as usize;
    map.tiles[index] = TileType::Empty;
}

// Carve a horizontal hallway between two X coordinates.
fn carve_horizontal(
    map: &mut RoomMap,
    x1: i32,
    x2: i32,
    y: i32,
    horz_hallways: &mut Vec<HorzHallway>,
) {
    horz_hallways.push(HorzHallway {
        x_start: x1.min(x2),
        x_end: x1.max(x2),
        y_top: y + HORIZONTAL_WIDTH - 2,
    });
    for x in x1.min(x2)..=x1.max(x2) {
        for offset in -1..HORIZONTAL_WIDTH - 1 {
            carve_tile(map, x, y + offset);
        }
    }
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

    for edge in edges {
        let (Some(a), Some(b)) = (map.rooms.get(edge.a), map.rooms.get(edge.b)) else {
            continue;
        };

        let (ax, ay) = room_center(a);
        let (bx, by) = room_center(b);

        // Calculate the overlapping tile ranges.
        let overlap_x_start = (a.x as i32).max(b.x as i32);
        let overlap_x_end = ((a.x + a.width) as i32 - 1).min((b.x + b.width) as i32 - 1);

        let overlap_y_start = (a.y as i32).max(b.y as i32);
        let overlap_y_end = ((a.y + a.height) as i32 - 1).min((b.y + b.height) as i32 - 1);

        if overlap_x_start <= overlap_x_end {
            // Rooms overlap along X: vertical hallway.
            let x = (overlap_x_start + overlap_x_end) / 2;
            carve_vertical(map, ay, by, x, &mut vert_hallways);
        } else if overlap_y_start <= overlap_y_end {
            // Rooms overlap along Y: horizontal hallway.
            let y = (overlap_y_start + overlap_y_end) / 2;
            carve_horizontal(map, ax, bx, y, &mut horz_hallways);
        } else {
            // Neither axis overlaps: L-shaped hallway.
            carve_horizontal(map, ax, bx, ay, &mut horz_hallways);
            carve_vertical(map, ay, by, bx, &mut vert_hallways);
        }
    }
    (vert_hallways, horz_hallways)
}
