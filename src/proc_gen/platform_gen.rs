use rand::{Rng, SeedableRng, rngs::StdRng};

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

pub fn generate_platforms(map: &mut RoomMap, hallways: &[VertHallway], seed: u64) -> Vec<Platform> {
    let mut platforms = Vec::new();
    let mut rng = StdRng::seed_from_u64(seed);

        // Group shafts that share carved tiles, including partial overlaps.
    let mut grouped = vec![false; hallways.len()];

    for first in 0..hallways.len() {
        if grouped[first] {
            continue;
        }

        let mut group = vec![first];
        grouped[first] = true;

        let mut i = 0;
        while i < group.len() {
            let a = &hallways[group[i]];

            for (j, b) in hallways.iter().enumerate() {
                if grouped[j] {
                    continue;
                }

                let overlaps_x = (a.x - b.x).abs() <= a.width / 2 + b.width / 2;
                let overlaps_y = a.y_start <= b.y_end && b.y_start <= a.y_end;

                if overlaps_x && overlaps_y {
                    grouped[j] = true;
                    group.push(j);
                }
            }

            i += 1;
        }

        let bottom = group.iter().map(|&i| hallways[i].y_start).min().unwrap();
        let top = group.iter().map(|&i| hallways[i].y_end).max().unwrap();

        for (row, y) in ((bottom + 3)..(top - 2)).step_by(4).enumerate() {
            // Find the horizontal span of each shaft that crosses this row.
            let mut spans = Vec::new();
            for &i in &group {
                let shaft = &hallways[i];
                if shaft.y_start <= y && y <= shaft.y_end {
                    let left = shaft.x - shaft.width / 2;
                    spans.push((left, left + shaft.width - 1));
                }
            }
            spans.sort_unstable();

            // Overlapping spans are one physical passage at this height.
            let mut passages: Vec<(i32, i32)> = Vec::new();
            for (left, right) in spans {
                if let Some(last) = passages.last_mut() {
                    if left <= last.1 {
                        last.1 = last.1.max(right);
                        continue;
                    }
                }
                passages.push((left, right));
            }

            for (left, right) in passages {
                let width = rng.gen_range(2..=3);
                let x = if row % 2 == 0 {
                    left
                } else {
                    right - width + 1
                };
            // Ensure the platform is inside map bounds
            let in_bounds =
                x >= 1 && x + width <= map.width as i32 - 1 && y >= 1 && y < map.height as i32 - 1;

            let blocks_room = map.rooms.iter().any(|room| {
            let room_left = room.x as i32;
            let room_right = (room.x + room.width) as i32;
            let room_floor = room.y as i32;
            let room_ceiling = (room.y + room.height) as i32;

            let inside_room = x < room_right
                && x + width > room_left
                && y >= room_floor
                && y < room_ceiling;

            if !inside_room {
                return false;
            }
        
            // Allow ledges beneath a shaft that exits through this room's ceiling.
            let under_shaft = group.iter().any(|&i| {
                let shaft = &hallways[i];
                let left = shaft.x - shaft.width / 2;
                let right = left + shaft.width;
            
                shaft.y_start <= y
                    && shaft.y_end >= room_ceiling
                    && x >= left
                    && x + width <= right
            });
        
            !under_shaft
        });
        
        if in_bounds && !blocks_room {
                let platform = Platform { x, y, width };

                for tile_x in x..x + width {
                    let index = y as usize * map.width + tile_x as usize;
                    map.tiles[index] = TileType::Wall;
                }

                platforms.push(platform);
            }

            } 
        } 
    } 

    platforms
}     
