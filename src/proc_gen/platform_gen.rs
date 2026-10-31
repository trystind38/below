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

            let platform = Platform { x, y, width };

            // Place solid platform tiles into the map
            for tile_x in x..x + width {
                let index = y as usize * map.width + tile_x as usize;
                map.tiles[index] = TileType::Wall;
            }

            platforms.push(platform);

            // Alternate sides and go up 4 tiles
            left_right = !left_right;
            y += 4;
        }    
    }

    platforms
}