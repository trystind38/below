use bevy::{prelude::*, window::PresentMode};

mod level;
mod loading;

const TITLE: &str = "Below - Level Demo";
const WIN_W: f32 = 1280.;
const WIN_H: f32 = 720.;

const TILE_SIZE: f32 = 64.;

const LEVEL_LEN: f32 = 1280.;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Loading,
    Playing,
}

fn main() {
    App::new()
        // Setup Bevy and game window
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from(TITLE),
                resolution: (WIN_W as u32, WIN_H as u32).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::Srgba(Srgba::gray(0.25))))
        // Set initial state
        .init_state::<GameState>()
        // Add general systems
        .add_systems(Startup, setup_camera)
        .add_systems(OnEnter(GameState::Loading), log_state_change)
        .add_systems(OnEnter(GameState::Playing), log_state_change)
        // Add all subsystems
        .add_plugins((
            level::LevelPlugin,
            loading::LoadingPlugin,
        ))
        // Run the game
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn log_state_change(state: Res<State<GameState>>) {
    info!("Just moved to {:?}!", state.get());
}