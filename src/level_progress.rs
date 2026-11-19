use bevy::prelude::*;

use crate::{
    GameState, LEVEL_BOTTOM, LEVEL_RIGHT, TILE_SIZE,
    boss_level::boss_level,
    level::ActiveLevel,
    loading::despawn_with,
    player::{PLAYER_SIZE, Player},
    proc_gen::room_generation::{Room, TileType},
};

// Seeds for the two generated levels. Change a number to get a different dungeon.
const LEVEL_ONE_SEED: u64 = 1;
const LEVEL_TWO_SEED: u64 = 2;

const DOOR_SIZE: Vec2 = Vec2::new(TILE_SIZE, TILE_SIZE * 2.);

// Every level, in the order they are played
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum LevelId {
    #[default]
    Demo,
    One,
    Two,
    Boss,
}

impl LevelId {
    pub const ALL: [LevelId; 4] = [LevelId::Demo, LevelId::One, LevelId::Two, LevelId::Boss];

    // Text for the main menu buttons
    pub fn name(self) -> &'static str {
        match self {
            LevelId::Demo => "Demo",
            LevelId::One => "Level 1",
            LevelId::Two => "Level 2",
            LevelId::Boss => "Boss",
        }
    }

    // The level the exit door leads to. None = the last level, so no door.
    pub fn next(self) -> Option<LevelId> {
        match self {
            LevelId::Demo => Some(LevelId::One),
            LevelId::One => Some(LevelId::Two),
            LevelId::Two => Some(LevelId::Boss),
            LevelId::Boss => None,
        }
    }

    fn build(self) -> ActiveLevel {
        match self {
            LevelId::Demo => ActiveLevel::default(),
            LevelId::One => ActiveLevel::procedural(LEVEL_ONE_SEED),
            LevelId::Two => ActiveLevel::procedural(LEVEL_TWO_SEED),
            LevelId::Boss => boss_level(),
        }
    }
}

// Which level is being played right now
#[derive(Resource, Default)]
pub struct CurrentLevel(pub LevelId);

// Loads a level and (re)enters Playing. Call this from the main menu buttons and from doors.
pub fn start_level(
    id: LevelId,
    active: &mut ActiveLevel,
    current: &mut CurrentLevel,
    next_state: &mut NextState<GameState>,
) {
    *active = id.build();
    current.0 = id;
    next_state.set(GameState::Playing);
}

#[derive(Component)]
struct ExitDoor;

pub struct LevelProgressPlugin;
impl Plugin for LevelProgressPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentLevel>()
            .add_systems(OnEnter(GameState::Playing), spawn_exit_door)
            .add_systems(Update, enter_next_level.run_if(in_state(GameState::Playing)))
            .add_systems(OnExit(GameState::Playing), despawn_with::<ExitDoor>);
    }
}

// Where the exit door goes in a level: the intro level's far right end, or for a generated
// map the room farthest from the spawn, near its right end
fn exit_door_position(level: &ActiveLevel) -> Option<Vec2> {
    let Some(map) = level.map.as_ref() else {
        return Some(Vec2::new(
            LEVEL_RIGHT - 2. * TILE_SIZE,
            LEVEL_BOTTOM + TILE_SIZE + DOOR_SIZE.y / 2.,
        ));
    };

    let from = level.spawn / TILE_SIZE;
    let dist = |r: &Room| {
        let (cx, cy) = r.center();
        Vec2::new(cx as f32, cy as f32).distance_squared(from)
    };
    let room = map.rooms.iter().max_by(|a, b| dist(a).total_cmp(&dist(b)))?;

    let x = (room.x + room.width - 3.) as usize;
    let mut y = room.y as usize;
    // Terrain hills can raise the floor, so climb until the tile is open
    while y < map.height - 1 && matches!(map.tiles[y * map.width + x], TileType::Wall) {
        y += 1;
    }
    Some(Vec2::new(
        (x as f32 + 0.5) * TILE_SIZE,
        y as f32 * TILE_SIZE + DOOR_SIZE.y / 2.,
    ))
}

fn spawn_exit_door(mut commands: Commands, current: Res<CurrentLevel>, active: Res<ActiveLevel>) {
    let Some(next) = current.0.next() else {
        return; // the last level has no exit
    };
    let Some(pos) = exit_door_position(&active) else {
        return;
    };

    // the door to the boss is purple, the others are gold
    let color = if next == LevelId::Boss {
        Color::srgb(0.35, 0.1, 0.45)
    } else {
        Color::srgb(0.75, 0.6, 0.2)
    };
    commands.spawn((
        Sprite::from_color(color, DOOR_SIZE),
        Transform::from_translation(pos.extend(0.5)),
        ExitDoor,
    ));
}

// Touch the door to go to the next level
fn enter_next_level(
    mut active: ResMut<ActiveLevel>,
    mut current: ResMut<CurrentLevel>,
    mut next_state: ResMut<NextState<GameState>>,
    player: Single<&Transform, With<Player>>,
    doors: Query<&Transform, (With<ExitDoor>, Without<Player>)>,
) {
    let Some(next) = current.0.next() else {
        return;
    };
    for door in &doors {
        let d = (player.translation - door.translation).truncate().abs();
        let reach = (PLAYER_SIZE + DOOR_SIZE) / 2.;
        if d.x < reach.x && d.y < reach.y {
            start_level(next, &mut active, &mut current, &mut next_state);
            return;
        }
    }
}