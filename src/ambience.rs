use bevy::prelude::*;
use rand::Rng;

const MIN_INTERVAL: f32 = 5.0;
const MAX_INTERVAL: f32 = 10.0;

const MIN_PLAYBACK_SPEED: f32 = 0.25;
const MAX_PLAYBACK_SPEED: f32 = 0.75;

pub struct AmbiencePlugin;

#[derive(Resource)]
struct AmbienceSounds {
    sounds: Vec<Handle<AudioSource>>,
    timer: Timer,
}

impl Plugin for AmbiencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ambience)
            .add_systems(
                Update,
                play_ambience.run_if(in_state(crate::GameState::Playing)),
            );
    }
}

fn setup_ambience(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let sounds = vec![
        asset_server.load("sounds/ambience/Recording2.ogg"),
        asset_server.load("sounds/ambience/Recording3.ogg"),
        asset_server.load("sounds/ambience/Recording4.ogg"),
        asset_server.load("sounds/ambience/Recording5.ogg"),
        asset_server.load("sounds/ambience/Recording6.ogg"),
        asset_server.load("sounds/ambience/Recording7.ogg"),
        asset_server.load("sounds/ambience/Recording8.ogg"),
        asset_server.load("sounds/ambience/Recording9.ogg"),
        asset_server.load("sounds/ambience/Recording10.ogg"),
        asset_server.load("sounds/ambience/Recording11.ogg"),
    ];

    let initial_interval = rand::thread_rng()
    .gen_range(MIN_INTERVAL..=MAX_INTERVAL);

    commands.insert_resource(AmbienceSounds {
        sounds,
        timer: Timer::from_seconds(initial_interval, TimerMode::Once),
    });
}

fn play_ambience(
    time: Res<Time>,
    mut ambience: ResMut<AmbienceSounds>,
    mut commands: Commands,
) {
    ambience.timer.tick(time.delta());

    if ambience.timer.just_finished() {
        if !ambience.sounds.is_empty() {
            let index: usize = rand::thread_rng()
                .gen_range(0..ambience.sounds.len());

            let speed = rand::thread_rng()
                .gen_range(0.5_f32..=1.0_f32);

            commands.spawn((
                AudioPlayer(ambience.sounds[index].clone()),
                PlaybackSettings::DESPAWN.with_speed(speed),
            ));
        }

        let next_interval = rand::thread_rng()
            .gen_range(MIN_INTERVAL..=MAX_INTERVAL);

        ambience.timer = Timer::from_seconds(
            next_interval,
            TimerMode::Once,
        );
    }
}