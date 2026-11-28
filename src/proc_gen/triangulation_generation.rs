use crate::{
    GameState,
    loading::despawn_with,
    proc_gen::{
        geometry::{Edge, Point},
        room_generation::{ProcGenSeed, RoomWallImage, frame_camera, map_preview, map_rooms},
        triangulation_helpers::*,
    },
};
use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component)]
pub struct TriangulationPreview;

pub fn triangulate(points: &Vec<Point>) -> Vec<Edge> {
    let mut edges: Vec<QuarterEdge> = Vec::new();

    let _triangle = make_triangle(
        &mut edges,
        Point {
            x: -10000.0,
            y: 0.0,
        },
        Point { x: 10000.0, y: 0.0 },
        Point { x: 0.0, y: 10000.0 },
    );

    for p in points {
        let located = find_bounding_triangle(&edges, *p);
        let first_spoke = insert_point(&mut edges, located, *p);
        legalize(&mut edges, first_spoke, *p);
    }

    let mut result_edges = Vec::new();

    for i in 0..edges.len() {
        let start = match edges[i].data {
            Some(point) => point,
            None => continue,
        };
        let end = match dest(&edges, i) {
            Some(point) => point,
            None => continue,
        };

        let mut a = 0;
        let mut b = 0;
        let mut a_found = false;
        let mut b_found = false;
        for (j, p) in points.iter().enumerate() {
            if p.x == start.x && p.y == start.y {
                a = j;
                a_found = true;
            }
            if p.x == end.x && p.y == end.y {
                b = j;
                b_found = true;
            }
        }

        if a_found && b_found {
            result_edges.push(Edge::new(a, b));
        }
    }

    // Sort and deduplicate the edges to remove any duplicates
    result_edges.sort_unstable_by_key(|edge| (edge.a, edge.b));
    result_edges.dedup();

    result_edges
}

fn triangulation_preview(centers: &Vec<Point>, edges: &Vec<Edge>, commands: &mut Commands) {
    for edge in edges {
        let ax = centers[edge.a].x as f32 * 32.0;
        let ay = centers[edge.a].y as f32 * 32.0;
        let bx = centers[edge.b].x as f32 * 32.0;
        let by = centers[edge.b].y as f32 * 32.0;

        let dx = bx - ax;
        let dy = by - ay;
        let length = (dx * dx + dy * dy).sqrt();

        let mid_x = (ax + bx) / 2.0;
        let mid_y = (ay + by) / 2.0;
        let angle = dy.atan2(dx);

        commands.spawn((
            TriangulationPreview,
            Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(length, 3.0)),
                ..default()
            },
            Transform::from_xyz(mid_x, mid_y, 1.0).with_rotation(Quat::from_rotation_z(angle)),
        ));
    }
}

fn setup_triangulation_preview(
    mut commands: Commands,
    wall_image: Res<RoomWallImage>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    seed: Res<ProcGenSeed>,
) {
    //Getting room centers to then traingulate
    let map = map_rooms(seed.specs());
    map_preview(&map, &mut commands, &wall_image.0);

    //I want points not rooms, so i'm turning rooms into points
    let points: Vec<Point> = map
        .rooms
        .iter()
        .map(|room| {
            let (cx, cy) = room.center(); //getting room coordinates
            // yay points
            Point { x: cx, y: cy }
        })
        .collect();

    let edges = triangulate(&points); //now we have the adges from the traingulation

    triangulation_preview(&points, &edges, &mut commands); //map it out

    frame_camera(&map, window.into_inner(), camera); //camera
}

pub struct TriangulationPlugin;

impl Plugin for TriangulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Triangulation),
            setup_triangulation_preview,
        )
        .add_systems(
            OnExit(GameState::Triangulation),
            despawn_with::<TriangulationPreview>,
        );
    }
}
