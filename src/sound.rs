use bevy::prelude::*;

use crate::{
    GameState,
    loading::{LoadingAssets, despawn_with},
    player::Player,
};

const ICON_SIZE: u32 = 16;
const ICON_FRAMES: u32 = 3;
const SPRITE_SCALE: f32 = 2.;

const POPUP_TIME: f32 = 0.5;
const POPUP_OFFSET: Vec2 = Vec2::new(0., 56.); // above the player's head

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoundKind {
    Step,
    Land,
    Shot,
}

impl SoundKind {
    // Hearing radius in world units, for enemy AI to use later
    pub fn loudness(self) -> f32 {
        match self {
            SoundKind::Step => 150.,
            SoundKind::Land => 300.,
            SoundKind::Shot => 600.,
        }
    }

    // Speaker frame: 1, 2 or 3 sound arcs
    fn icon_index(self) -> usize {
        match self {
            SoundKind::Step => 0,
            SoundKind::Land => 1,
            SoundKind::Shot => 2,
        }
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub struct SoundEvent {
    pub pos: Vec2,
    pub kind: SoundKind,
}

impl SoundEvent {
    pub fn new(pos: Vec2, kind: SoundKind) -> Self {
        Self { pos, kind }
    }
}

#[derive(Component)]
struct SoundPopup {
    kind: SoundKind,
    timer: Timer,
}

// Debug toggle (H) for the speaker popups. Sound events are sent either way
#[derive(Resource)]
struct ShowSoundPopups(bool);

#[derive(Resource)]
struct SpeakerSheet {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

pub struct SoundPlugin;
impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SoundEvent>()
            .insert_resource(ShowSoundPopups(true))
            .add_systems(Startup, load_speaker)
            .add_systems(
                Update,
                (
                    toggle_popups,
                    spawn_popups.run_if(|show: Res<ShowSoundPopups>| show.0),
                    animate_popups,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), despawn_with::<SoundPopup>);
    }
}

fn load_speaker(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let image: Handle<Image> = asset_server.load("ui/speaker.png");
    loading_assets.0.push(image.clone().untyped());

    let layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(ICON_SIZE),
        ICON_FRAMES,
        1,
        None,
        None,
    ));

    commands.insert_resource(SpeakerSheet { image, layout });
}

fn toggle_popups(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut show: ResMut<ShowSoundPopups>,
    popups: Query<Entity, With<SoundPopup>>,
) {
    if !keys.just_pressed(KeyCode::KeyH) {
        return;
    }
    show.0 = !show.0;
    if !show.0 {
        for e in &popups {
            commands.entity(e).despawn();
        }
    }
}

fn spawn_popups(
    mut commands: Commands,
    sheet: Res<SpeakerSheet>,
    mut sounds: MessageReader<SoundEvent>,
    popup: Option<Single<(&mut SoundPopup, &mut Sprite)>>,
) {
    let Some(kind) = sounds
        .read()
        .map(|s| s.kind)
        .max_by(|a, b| a.loudness().total_cmp(&b.loudness()))
    else {
        return;
    };

    // Only one icon: replace it unless a louder sound is still showing
    if let Some(popup) = popup {
        let (mut popup, mut sprite) = popup.into_inner();
        if kind.loudness() < popup.kind.loudness() {
            return;
        }
        popup.kind = kind;
        popup.timer.reset();
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = kind.icon_index();
        }
        return;
    }

    commands.spawn((
        Sprite::from_atlas_image(
            sheet.image.clone(),
            TextureAtlas {
                layout: sheet.layout.clone(),
                index: kind.icon_index(),
            },
        ),
        Transform::from_scale(Vec3::splat(SPRITE_SCALE)),
        SoundPopup {
            kind,
            timer: Timer::from_seconds(POPUP_TIME, TimerMode::Once),
        },
    ));
}

fn animate_popups(
    mut commands: Commands,
    time: Res<Time>,
    player: Single<&Transform, (With<Player>, Without<SoundPopup>)>,
    mut popups: Query<(Entity, &mut SoundPopup, &mut Transform, &mut Sprite)>,
) {
    let head = player.translation.truncate() + POPUP_OFFSET;
    for (entity, mut popup, mut tf, mut sprite) in &mut popups {
        popup.timer.tick(time.delta());
        if popup.timer.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation = head.extend(5.);
        sprite.color.set_alpha(1. - popup.timer.fraction());
    }
}
