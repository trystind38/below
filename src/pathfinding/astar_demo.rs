use bevy::{
    color::palettes::css::*,
    prelude::*,
    ui_widgets:: {
        Activate,
        Button,
    },
    window::PrimaryWindow,
};

use rand::Rng;
use std::collections::HashMap;

#[derive(Component)]
pub struct AStarDemoUI;

use crate:: {
    GameState,
    loading::{ LoadingAssets, despawn_with },
    pathfinding::astar::*,
};

#[derive(Resource)]
pub struct EmptyGridImage(pub Handle<Image>);

#[derive(Component, Clone, Copy, Debug)]
pub struct GridCell {
    position: IVec2,
    walkable: bool,
}

impl GridCell {
    fn new(position: IVec2, walkable: bool) -> GridCell {
        GridCell {
            position: position,
            walkable: walkable,
        }
    }
}



const GRID_SIZE: i32 = 9;
const CELL_SIZE: i32 = 32;
const MAX_BARRIERS: i32 = 18;

const PADDING: i32 = 64;

// Load grid asset "EmptyGridImage"
fn load_astar_demo_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let empty_grid_handle: Handle<Image> = asset_server.load("tiles/nice_grid.png");
    loading_assets.0.push(empty_grid_handle.clone().untyped());
    commands.insert_resource(EmptyGridImage(empty_grid_handle));
}

// Frame camera and draw the empty grid
fn setup_astar_demo(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    time: Res<Time>,
    empty_grid_image: Res<EmptyGridImage>,
) {
    frame_camera(window.into_inner(), camera);
    spawn_menu(
        &mut commands,
        "A* Demo",
        &[
            ("Regenerate", GameState::AStar),
            ("Back", GameState::MainMenu),
        ]
    );
    let grid = draw_grid(&mut commands, &empty_grid_image);
    //println!("{:#?}", grid);

    // Uh
    //println!("A Star Solve!!!!");
    astar_solve(&mut commands, &grid, empty_grid_image, time);
}

fn frame_camera(
    window: &Window,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let world_width = (GRID_SIZE * CELL_SIZE) as f32;
    let world_height = (GRID_SIZE * CELL_SIZE + PADDING) as f32;

    let (mut transform, mut projection) = camera.into_inner();

    transform.translation.x = world_width / 2.0;
    transform.translation.y = world_height / 2.0;

    if let Projection::Orthographic(orthographic) = &mut *projection {
        orthographic.scale = (
            world_width / window.width().max(1.0)
        )
            .max(world_height / window.height().max(1.0))
            * 1.1;
    }
}

fn spawn_menu(
    commands: &mut Commands,
    title: &str,
    options: &[(&str, GameState)],
) {
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                row_gap: px(6),
                column_gap: px(6),
                ..default()
            },
            AStarDemoUI
        ))
        .with_children( |parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            for &(label, destination) in options {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: px(200),
                            height: px(42),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.20, 0.35, 0.20)),
                    ))
                    .with_children( |button| {
                        button.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: FontSize::Px(18.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    })
                    .observe(
                        move |_: On<Activate>,
                            mut next_state: ResMut<NextState<GameState>>| {
                                next_state.set(destination);
                            },
                    );
            }
        });
}

fn draw_grid(
    commands: &mut Commands,
    empty_grid_image: &Res<EmptyGridImage>,
) -> Vec<GridCell> {
    let mut rng = rand::thread_rng();

    let mut barriers: Vec<(i32, i32)> = vec![];
    for _i in 0..MAX_BARRIERS {
        barriers.push((rng.gen_range(0..GRID_SIZE), rng.gen_range(0..GRID_SIZE)));
    }

    //println!("{:?}", barriers);

    let mut grid: Vec<GridCell> = vec![];

    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            // Place cells in a spot that will look nice in the screen
            let x_position = (x as f32 + 0.5) * CELL_SIZE as f32;
            let y_position = (y as f32 + 0.5) * CELL_SIZE as f32;

            // Construct a GridCell to add to the graph we later give to a star.
            let new_cell = GridCell::new(
                IVec2::new(x,y),
                !barriers.contains(&(x,y))
                || (x,y) == (0,0)
                || (x,y) == (GRID_SIZE - 1, GRID_SIZE - 1), // Top right corner
            );

            grid.push(new_cell);

            commands.spawn((
                Sprite {
                    image: empty_grid_image.0.clone(),
                    color: if x == 0 && y == 0 {
                        Color::Srgba(BLUE)
                    } else if x == GRID_SIZE - 1 && y == GRID_SIZE - 1{
                        Color::Srgba(GOLD)
                    } else if barriers.contains(&(x, y)) {
                        Color::Srgba(BLACK)
                    } else {
                        Color::Srgba(GRAY)
                    },
                    custom_size: Some(Vec2::splat(CELL_SIZE as f32)),
                    ..default()
                },
                Transform::from_xyz(
                    x_position,
                    y_position,
                    0.0,
                ),
                new_cell
            ));
        }
    }

    grid
}

fn astar_solve(
    commands: &mut Commands,
    grid: &Vec<GridCell>,
    empty_grid_image: Res<EmptyGridImage>,
    time: Res<Time>,
) {
    let mut node_graph: HashMap<IVec2, Neighbors> = HashMap::new();

    // Construct neighbor HashMap for NodeGraph to hand off to astar
    for y in 0..GRID_SIZE {
        for x in 0..GRID_SIZE {
            // x and y will never be > GRID_SIZE
            let vec_index: usize = (x + (y * GRID_SIZE)) as usize;
            let current_cell = grid[vec_index];

            // Cells that aren't walkable don't matter
            if !current_cell.walkable {
                continue;
            }

            // Create empty list for the current_cell's neighbors.
            let mut neighbors: Vec<(IVec2, i32)> = vec![];

            // If the neighbor exists, add it to neighbors with a cost of 1 because this is a grid

            // Left neighbor
            if x > 0 {
                let left_neighbor: GridCell = grid[vec_index - 1];
                if left_neighbor.walkable {
                    neighbors.push((left_neighbor.position, 1));
                }
            }

            // Right neighbor
            if x < GRID_SIZE - 1 {
                let right_neighbor: GridCell = grid[vec_index + 1];
                if right_neighbor.walkable {
                    neighbors.push((right_neighbor.position, 1));
                }
            }

            // Above neighbor
            if y > 0 {
                let above_neighbor: GridCell = grid[vec_index - GRID_SIZE as usize];
                if above_neighbor.walkable {
                    neighbors.push((above_neighbor.position, 1));
                }
            }

            // Below neighbor
            if !(y == GRID_SIZE - 1) {
                let below_neighbor: GridCell = grid[vec_index + GRID_SIZE as usize]; // TODO: Handle edge case of last corner cell?
                if below_neighbor.walkable {
                    neighbors.push((below_neighbor.position, 1));
                }
            }

            node_graph.insert(current_cell.position, neighbors);
        }
    }

    let start: IVec2 = IVec2::new(0,0);
    let destination: IVec2 = IVec2::new(GRID_SIZE - 1, GRID_SIZE - 1);

    let path: Option<Vec<IVec2>>= astar_path(
        start,
        destination,
        &NodeGraph {
            neighbors: node_graph
        }
    );

    let path = match path {
        Some(path) => path,
        None => vec![],
    };

    // if path.is_empty() {
    //     println!("Path not found!");
    //     return;
    // }

    // Draw solved path
    for i in 0..path.len() {
        if path[i].x == 0 && path[i].y == 0
        || path[i].x == GRID_SIZE - 1 && path[i].y == GRID_SIZE - 1 {
            continue;
        }

        commands.spawn((
                Sprite {
                    image: empty_grid_image.0.clone(),
                    color: Color::Srgba(GREEN),
                    custom_size: Some(Vec2::splat(CELL_SIZE as f32)),
                    ..default()
                },
                Transform::from_xyz(
                    (path[i].x as f32 + 0.5) * CELL_SIZE as f32,
                    (path[i].y as f32 + 0.5) * CELL_SIZE as f32,
                    1.0,
                ),
                GridCell { position: IVec2::new(-1,-1), walkable: false }
        ));
    }
}

pub struct AStarDemoPlugin;

impl Plugin for AStarDemoPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Startup,
                load_astar_demo_assets,
            )
            .add_systems(
                OnEnter(GameState::AStar),
                setup_astar_demo
            )
            .add_systems(
                OnTransition {
                    exited: GameState::AStar,
                    entered: GameState::MainMenu
                },
                despawn_with::<GridCell>,
            )
            .add_systems(
                OnTransition {
                    exited: GameState::AStar,
                    entered: GameState::MainMenu,
                },
                despawn_with::<AStarDemoUI>,
            )
            .add_systems(
                OnTransition {
                    exited: GameState::AStar,
                    entered: GameState::AStar,
                },
                despawn_with::<GridCell>,
            );
    }
}
