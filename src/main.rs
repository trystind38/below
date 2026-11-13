use bevy::{prelude::*, window::PresentMode};

mod pathfinding {
    pub mod astar;
}
mod burst_meter;
mod camera;
mod combat;
mod forcefield;
mod shining_road;
mod gun;
mod level;
mod loading;
mod player;
mod stats;
mod health_bar;
mod gerald;
mod teeth;
mod hivemind;
mod hivemind_controller;
mod tank_light;
mod menus;
mod sound;
mod light_burst;
mod proc_gen {
    pub mod geometry;
    pub mod mst;
    pub mod room_generation;
    pub mod triangulation_helpers;
    pub mod triangulation_generation;
    pub mod dungeon_generation;
    pub mod hallway_gen;
    pub mod platform_gen;
    pub mod terrain_features;
}

const TITLE: &str = "Below - Level Demo";
const WIN_W: f32 = 1280.;
const WIN_H: f32 = 720.;

const TILE_SIZE: f32 = 32.;

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
    MainMenu,
    Playing,
    RoomGeneration,
    ProcGenMenu,
    Triangulation,
    Mst,
    AISlideshow,
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
            health_bar::HealthBarPlugin,
            player::PlayerPlugin,
            gerald::GeraldPlugin,
            hivemind::HivemindPlugin,
            hivemind_controller::HivemindControllerPlugin,
            gun::GunPlugin,
            combat::CombatPlugin,
            menus::MenuPlugin,
            sound::SoundPlugin,
            proc_gen::room_generation::RoomGenerationPlugin,
            proc_gen::triangulation_generation::TriangulationPlugin,
            proc_gen::mst::MstPlugin,
        ))
        .add_plugins((
            tank_light::TankLightPlugin,
            teeth::TeethPlugin,
            burst_meter::BurstMeterPlugin,
            light_burst::LightBurstPlugin,
            forcefield::ForceFieldPlugin,
            shining_road::ShiningRoadPlugin,
            proc_gen::terrain_features::TerrainFeaturesPlugin,
        ))
        // Run the game
        .run();
}

fn log_state_change(state: Res<State<GameState>>) {
    info!("Just moved to {:?}!", state.get());
}