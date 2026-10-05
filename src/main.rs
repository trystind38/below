use bevy::{prelude::*, window::PresentMode};

mod camera;
mod level;
mod loading;
mod player;
mod stats;

const TITLE: &str = "Below - Level Demo";
const WIN_W: f32 = 1280.;
const WIN_H: f32 = 720.;

const TILE_SIZE: f32 = 64.;

const LEVEL_LEN: f32 = WIN_W * 4.;
const LEVEL_H: f32 = WIN_H * 2.;

// World-space edges of the level. The level starts at the bottom-left corner of the initial screen.
const LEVEL_LEFT: f32 = -WIN_W / 2.;
const LEVEL_RIGHT: f32 = LEVEL_LEFT + LEVEL_LEN;
const LEVEL_BOTTOM: f32 = -WIN_H / 2.;
const LEVEL_TOP: f32 = LEVEL_BOTTOM + LEVEL_H;

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
            }).set(ImagePlugin::default_nearest())
        )
        .insert_resource(ClearColor(Color::Srgba(Srgba::gray(0.25))))
        // Set initial state
        .init_state::<GameState>()
        // Add general systems
        .add_systems(OnEnter(GameState::Loading), log_state_change)
        .add_systems(OnEnter(GameState::Playing), log_state_change)
        // Add all subsystems
        .add_plugins((
            camera::CameraPlugin,
            level::LevelPlugin,
            loading::LoadingPlugin,
            player::PlayerPlugin,
        ))
        // Run the game
        .run();
}

fn log_state_change(state: Res<State<GameState>>) {
    info!("Just moved to {:?}!", state.get());
}