use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::proc_gen::{
    hallway_gen::VertHallway,
    room_generation::{RoomMap, TileType},
};

pub struct Platform {
    // first tile of the platform (left)
    pub x: i32,     
    pub y: i32,      
    pub width: i32, 
}

pub fn generate_platforms(
    map: &mut RoomMap,
    hallways: &[VertHallway],
    seed: u64,
) -> Vec<Platform> {
    let mut platforms = Vec::new();
    let mut rng = StdRng::seed_from_u64(seed);
    
    for hallway in hallways {
        let mut y = hallway.y_start + 2;
        let mut left_right = false;
        // Generate platforms all along the vertical hallway
        while y < hallway.y_end - 2 {
            // Random platform width
            let width = rng.gen_range(2..=3);

            // Calculate left boundary of hallway
            let hallway_left = hallway.x - hallway.width / 2;

            // Alternate between left and right sides
            let x = if left_right {
                hallway_left + hallway.width - width
            } else {
                hallway_left
            };

            // Ensure the platform is inside map bounds
            let in_bounds =
                x >= 1
                && x + width <= map.width as i32 - 1
                && y >= 1
                && y < map.height as i32 - 1;

            // Don't place platforms inside rooms or against their bounds
            let overlaps_room = map.rooms.iter().any(|room| {
                let left = room.x as i32;
                let right = (room.x + room.width) as i32;
                let bottom = room.y as i32 - 1;
                let top = (room.y + room.height) as i32 + 1;
            
                x < right && x + width > left
                    && y >= bottom && y < top
            });

            if in_bounds && !overlaps_room {
                let platform = Platform { x, y, width };
            
                for tile_x in x..x + width {
                    let index = y as usize * map.width + tile_x as usize;
                    map.tiles[index] = TileType::Wall;
                }
            
                platforms.push(platform);
            }
            
            // Alternate sides and go up 4 tiles
            left_right = !left_right;
            y += 4;
        }   
    }

    platforms
}