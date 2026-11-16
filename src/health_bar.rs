use bevy::prelude::*;

use crate::{
    GameState,
    loading::despawn_with,
    player::Player,
    stats::{Health, PLAYER_HEALTH},
};

const SEGMENT_COUNT: usize = 3;
const SEGMENT_SIZE: Vec2 = Vec2::new(40., 16.); // size of each segment
const SEGMENT_GAP: f32 = 6.; // gap between segments
const BAR_MARGIN: f32 = 20.; // margin from the edge of the screen
const FULL_COLOR: Color = Color::srgb(1., 0.3, 0.6); // pink
const EMPTY_COLOR: Color = Color::srgb(0.25, 0.15, 0.15); //empty health is dark red

#[derive(Component)] // tag for each segment of the health bar
struct HealthSegment(usize); // index of this segment in the health bar

#[derive(Component)]
struct HealthBar;

//the player starts off with full health, so we can spawn the full health bar at the start of the game
//when the player takes damage, we can update the health bar to reflect the new health value.
pub struct HealthBarPlugin;
impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_health_bar)
            .add_systems(
                Update,
                update_health_bar.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), despawn_with::<HealthBar>);
    }
}
//spawn the health bar when the player spawns, and despawn it when the player dies
fn spawn_health_bar(mut commands: Commands) {
    info!("spawning health bar");
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(BAR_MARGIN),
                top: Val::Px(BAR_MARGIN),
                column_gap: Val::Px(SEGMENT_GAP),
                ..default()
            },
            HealthBar,
        ))
        .with_children(|bar| {
            for i in 0..SEGMENT_COUNT {
                // outer box: the dark "empty" background of one segment
                bar.spawn((
                    Node {
                        width: Val::Px(SEGMENT_SIZE.x),
                        height: Val::Px(SEGMENT_SIZE.y),
                        ..default()
                    },
                    BackgroundColor(EMPTY_COLOR),
                ))
                // inner box: the pink fill that shrinks as health drops
                .with_children(|segment| {
                    segment.spawn((
                        Node {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                            ..default()
                        },
                        BackgroundColor(FULL_COLOR),
                        HealthSegment(i),
                    ));
                });
            }
        });
}

fn update_health_bar(
    player: Single<&Health, With<Player>>,
    mut fills: Query<(&HealthSegment, &mut Node)>,
) {
    let per_segment = PLAYER_HEALTH / SEGMENT_COUNT as f32;
    for (segment, mut node) in &mut fills {
        let fill = ((player.current - segment.0 as f32 * per_segment) / per_segment).clamp(0., 1.);
        node.width = Val::Percent(fill * 100.);
    }
}
