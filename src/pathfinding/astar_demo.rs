use bevy::{prelude::*, window::PrimaryWindow};
use crate:: {
    GameState,
    loading::{ LoadingAssets, despawn_with },
    //pathfinding::astar::*,
};

#[derive(Resource)]
pub struct EmptyGridImage(pub Handle<Image>);

#[derive(Component)]
pub struct Grid;



const GRID_WIDTH: i32 = 5;
const GRID_HEIGHT: i32 = 5;
const GRID_SIZE: i32 = 32;

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
    commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    empty_grid_image: Res<EmptyGridImage>,
) {
    frame_camera(window.into_inner(), camera);
    let grid = draw_grid(commands, empty_grid_image);
    println!("{:#?}", grid);
}

fn frame_camera(
    window: &Window,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let world_width = (GRID_WIDTH * GRID_SIZE) as f32;
    let world_height = (GRID_HEIGHT * GRID_SIZE) as f32;

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

fn draw_grid(
    mut commands: Commands,
    empty_grid_image: Res<EmptyGridImage>,
) -> Vec<IVec2> {
    let mut grid: Vec<IVec2> = vec![];

    let grid_sprite = Sprite {
        image: empty_grid_image.0.clone(),
        custom_size: Some(Vec2::splat(GRID_SIZE as f32)),
        ..default()
    };

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let x_position = (x as f32 + 0.5) * GRID_SIZE as f32;
            let y_position = (y as f32 + 0.5) * GRID_SIZE as f32;

            commands.spawn((
                grid_sprite.clone(),
                Transform::from_xyz(
                    x_position,
                    y_position,
                    0.0,
                ),
                Grid,
            ));

            grid.push(IVec2::new(x, y));
        }
    }

    grid
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
                OnExit(GameState::AStar),
                despawn_with::<Grid>,
            );
    }
}
