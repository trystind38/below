use crate::{
    GameState,
    loading::despawn_with,
    proc_gen::{
        dungeon_generation::generate_dungeon,
        geometry::{Edge, Point},
        room_generation::{
            ProcGenSeed, RoomPreview, RoomWallImage, frame_camera, map_preview, map_rooms,
        },
        triangulation_generation::triangulate,
    },
};
use bevy::prelude::*;

const TILE_SIZE: f32 = 32.0;
const LINE_THICKNESS: f32 = 8.0;
const NODE_SIZE: f32 = 24.0;

// Union-find (disjoint set) used by Kruskal's to detect cycles
struct DisjointSet {
    parent: Vec<usize>,
}

impl DisjointSet {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
        }
    }

    // Follow parents up to the root, flattening the path as we go
    fn find(&mut self, i: usize) -> usize {
        let mut root = i;
        while self.parent[root] != root {
            root = self.parent[root];
        }
        let mut node = i;
        while self.parent[node] != root {
            let next = self.parent[node];
            self.parent[node] = root;
            node = next;
        }
        root
    }

    // Returns false if a and b were already in the same set
    fn union(&mut self, a: usize, b: usize) -> bool {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a == root_b {
            return false;
        }
        self.parent[root_b] = root_a;
        true
    }
}

fn distance(a: &Point, b: &Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

// Every point connected to every other point.
pub fn all_edges(points: &[Point]) -> Vec<Edge> {
    let mut edges = Vec::new();
    for a in 0..points.len() {
        for b in a + 1..points.len() {
            edges.push(Edge { a, b });
        }
    }
    edges
}

// Kruskal's algorithm: take edges shortest first, skip any that would make a cycle
pub fn minimum_spanning_tree(points: &[Point], edges: &[Edge]) -> Vec<Edge> {
    let mut sorted: Vec<&Edge> = edges.iter().collect();
    sorted.sort_by(|e1, e2| {
        distance(&points[e1.a], &points[e1.b]).total_cmp(&distance(&points[e2.a], &points[e2.b]))
    });

    let mut sets = DisjointSet::new(points.len());
    let mut tree = Vec::new();

    for edge in sorted {
        if sets.union(edge.a, edge.b) {
            tree.push(Edge {
                a: edge.a,
                b: edge.b,
            });
        }
        // A tree over n points always has n - 1 edges
        if tree.len() + 1 >= points.len() {
            break;
        }
    }

    tree
}

// Converts a map-tile position to a world position
fn to_world(point: &Point) -> Vec2 {
    Vec2::new(point.x as f32 * TILE_SIZE, point.y as f32 * TILE_SIZE)
}

fn setup_mst_preview(
    mut commands: Commands,
    wall_image: Res<RoomWallImage>,
    window: Single<&Window, With<bevy::window::PrimaryWindow>>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    seed: Res<ProcGenSeed>,
) {
    let dungeon = generate_dungeon(seed.specs());

    map_preview(&dungeon.map, &mut commands, &wall_image.0);

    let points = &dungeon.points;
    let tree = &dungeon.mst_edges;

    let line_color = Color::srgb(0.95, 0.75, 0.2);

    // Draw each edge as a thin rotated rectangle between the two room centers
    for edge in tree {
        let start = to_world(&points[edge.a]);
        let end = to_world(&points[edge.b]);
        let delta = end - start;
        let midpoint = (start + end) / 2.0;

        commands.spawn((
            Sprite {
                color: line_color,
                custom_size: Some(Vec2::new(delta.length(), LINE_THICKNESS)),
                ..default()
            },
            Transform::from_xyz(midpoint.x, midpoint.y, 1.0)
                .with_rotation(Quat::from_rotation_z(delta.y.atan2(delta.x))),
            RoomPreview,
        ));
    }

    // Draw a dot on each room center
    for point in points.iter() {
        let position = to_world(point);
        commands.spawn((
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::splat(NODE_SIZE)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, 2.0),
            RoomPreview,
        ));
    }

    frame_camera(&dungeon.map, window.into_inner(), camera);
}

pub struct MstPlugin;

impl Plugin for MstPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Mst), setup_mst_preview)
            .add_systems(OnExit(GameState::Mst), despawn_with::<RoomPreview>);
    }
}
