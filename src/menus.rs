use bevy:: {
    prelude::*,
    ui_widgets::{Activate, Button},
};

use crate::{
    GameState,
    loading::despawn_with,
};

#[derive(Component)]
struct MenuUi;

#[derive(Component)]
struct SlideImage;

#[derive(Resource)]
struct AISlideshow {
    current_slide: usize,
    slides: Vec<String>,
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::MainMenu),
            setup_main_menu,
        )
        .add_systems(
            OnEnter(GameState::ProcGenMenu),
            setup_proc_gen_menu,
        )
        .add_systems(
            OnEnter(GameState::Playing),
            setup_basic_demo_controls,
        )
        .add_systems(
            OnEnter(GameState::RoomGeneration),
            setup_room_controls,
        )
        .add_systems(
            OnEnter(GameState::Triangulation),
            setup_room_controls,
        )
        .add_systems(
            OnEnter(GameState::Mst),
            setup_room_controls,
        )
        .add_systems(
            OnEnter(GameState::AISlideshow),
            (setup_slideshow_controls, spawn_slideshow,).chain(),
        )
        .add_systems(
            OnEnter(GameState::AStar),
            setup_basic_demo_controls,
        )
        .add_systems(
            OnExit(GameState::MainMenu),
            despawn_with::<MenuUi>,
        )
        .add_systems(
            OnExit(GameState::ProcGenMenu),
            despawn_with::<MenuUi>,
        )
        .add_systems(
            OnExit(GameState::Playing),
            despawn_with::<MenuUi>,
        )
        .add_systems(
            OnExit(GameState::RoomGeneration),
            despawn_with::<MenuUi>,
        )
        .add_systems(
            OnExit(GameState::Triangulation),
            despawn_with::<MenuUi>,
        )
        .add_systems(
            OnExit(GameState::Mst),
            despawn_with::<MenuUi>,
        )
        .add_systems(
            OnExit(GameState::AISlideshow),
            (despawn_with::<MenuUi>,
            despawn_with::<SlideImage>,).chain(),
        )
        .add_systems(
            OnExit(GameState::AStar),
            despawn_with::<MenuUi>,
        )
        .add_systems(Update, (escape_navigation, next_slide.run_if(in_state(GameState::AISlideshow))));
    }
}

// Main menu
fn setup_main_menu(mut commands: Commands) {
    spawn_menu(
        &mut commands,
        "Below",
        &[
            ("Basic Demo", GameState::Playing),
            ("Proc Gen", GameState::ProcGenMenu),
            ("Slideshow", GameState::AISlideshow),
            ("A Star", GameState::AStar),
        ],
    );
}

// Proc Gen menu
fn setup_proc_gen_menu(mut commands: Commands) {
    spawn_menu(
        &mut commands,
        "Procedural Generation",
        &[
            ("Room Generation", GameState::RoomGeneration),
            ("Delaunay Triangulation", GameState::Triangulation),
            ("MST", GameState::Mst),
            ("Back", GameState::MainMenu),
        ],
    );
}

// Triangulation screen
fn setup_triangulation_placeholder(mut commands: Commands) {
    spawn_menu(
        &mut commands,
        "Delaunay Triangulation",
        &[
            ("Back", GameState::ProcGenMenu),
        ],
    );
}

fn spawn_menu(
    commands: &mut Commands,
    title: &str,
    options: &[(&str, GameState)],
) {
    // Implementation for spawning a menu goes here
    commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(18),
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.09, 0.12)),
            MenuUi,
        ))
        .with_children(|parent| {
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
                            width: px(320),
                            height: px(56),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.25, 0.31)),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(label),
                            TextFont {
                                font_size: FontSize::Px(22.0),
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

// Small Back button over the basic level
fn setup_basic_demo_controls(mut commands: Commands) {
    spawn_back_button(&mut commands, GameState::MainMenu);
}

// Small Back button over the generated room map 
fn setup_room_controls(mut commands: Commands) {
    spawn_back_button(&mut commands, GameState::ProcGenMenu);
}

// Small Back button over the AI slidehow
fn setup_slideshow_controls(mut commands: Commands) {
    spawn_back_button(&mut commands, GameState::MainMenu);
}

fn spawn_back_button(
    commands: &mut Commands,
    destination: GameState,
) {
    commands
        .spawn((
            Button,
            Node {
                position_type: PositionType::Absolute,
                top: px(16),
                left: px(16),
                width: px(150),
                height: px(44),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.18, 0.25, 0.31)),
            MenuUi,
        ))
        .with_children(|button| {
            button.spawn((
                Text::new("Back (Esc)"),
                TextFont {
                    font_size: FontSize::Px(20.0),
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

fn escape_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    let destination = match state.get() {
        GameState::Playing | GameState::ProcGenMenu | GameState:: AISlideshow => {
            GameState::MainMenu
        }

        GameState::RoomGeneration
        | GameState::Triangulation
        | GameState::Mst => GameState::ProcGenMenu,

        _ => return,
    };

    next_state.set(destination);
}

fn spawn_slideshow(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let first_slide = asset_server.load("ai_slideshow/AmoogDecisionTree.png");

    commands.spawn((
        ImageNode::new(first_slide),
        Node {
            width: percent(100),
            height: percent(100),
            ..default()
        },
        SlideImage,
    ));
}

fn next_slide(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut images: Query<&mut ImageNode, With<SlideImage>>,
    asset_server: Res<AssetServer>,
    mut current_slide: Local<usize>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        *current_slide = (*current_slide + 1) % 3;

        let slide_path = match *current_slide {
            0 => "ai_slideshow/AmoogDecisionTree.png",
            1 => "ai_slideshow/GeraldDecisionTree.png",
            2 => "ai_slideshow/HivemindDecisionTree.png",
            _ => unreachable!(),
        };

        if let Ok(mut image) = images.single_mut() {
            image.image = asset_server.load(slide_path);
        }
    }
}