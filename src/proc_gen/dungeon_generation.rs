use rand::{rngs::StdRng, Rng, SeedableRng};
use crate::proc_gen::{
    geometry::{Edge, Point},
    room_generation::{RoomMap, map_rooms},
    triangulation_generation::triangulate,
    mst::minimum_spanning_tree,
    hallway_gen::{carve_hallways, VertHallway},
};


pub struct DungeonLayout {
    pub map: RoomMap,
    pub points: Vec<Point>,
    pub delauney_edges: Vec<Edge>,
    pub mst_edges: Vec<Edge>,
    pub selected_edges: Vec<Edge>,
    pub vert_hallways: Vec<VertHallway>,
}

pub fn generate_dungeon(seed: u64) -> DungeonLayout {
    // Generate the rooms
    let mut map = map_rooms(seed);

    // Get the center all rooms
    let points: Vec<Point> = map.rooms.iter().map(|room| {
        let (cx, cy) = room.center();
        Point { x: cx, y: cy }
    }).collect();

    // Delaunay triangulation
    let delauney_edges = triangulate(&points);
    // MST from Delaunay edges
    let mst_edges = minimum_spanning_tree(&points, &delauney_edges);

    // Select edges for hallways

    let mut selected_edges = mst_edges.clone();

    // Seeded so the same seed always gives the same loops
    let mut rng = StdRng::seed_from_u64(seed);
    for edge in &delauney_edges {
        let in_mst = mst_edges.iter().any(|e| e.a == edge.a && e.b == edge.b);
        if !in_mst && rng.gen_bool(0.2) {
            selected_edges.push(*edge);
        }
    }

    
    // Carve hallways
    let vert_hallways = carve_hallways(&mut map, &selected_edges);

    DungeonLayout {
        map,
        points,
        delauney_edges,
        mst_edges,
        selected_edges,
        vert_hallways,
    }
}