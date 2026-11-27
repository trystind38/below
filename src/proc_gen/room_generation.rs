use crate::{
    GameState,
    loading::{LoadingAssets, despawn_with},
    level::ActiveLevel,
};
use bevy::{prelude::*, window::PrimaryWindow};
use rand::{Rng, SeedableRng, rngs::StdRng};

struct SeededRng(u64);

impl SeededRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D049BB133111EB);
        value ^ (value >> 31)
    }

    fn gen_range_inclusive(&mut self, start: usize, end: usize) -> usize {
        let range = (end - start + 1) as u64;
        let threshold = range.wrapping_neg() % range;
        loop {
            let value = self.next_u64();
            if value >= threshold {
                return start + (value % range) as usize;
            }
        }
    }
}

#[derive(Resource)]
pub struct RoomWallImage(pub Handle<Image>);

#[derive(Clone, Copy)]
pub struct DungeonSpecs {
    pub seed: u64,
    pub map_width: usize,
    pub map_height: usize,
    pub room_count: usize,
}

pub const LEVEL_ONE: DungeonSpecs = DungeonSpecs {
    seed: 1,
    map_width: 192,
    map_height: 144,
    room_count: 12,
};

pub const LEVEL_TWO: DungeonSpecs = DungeonSpecs {
    seed: 2,
    map_width: 240,
    map_height: 172,
    room_count: 18,
};

#[derive(Resource)]
pub struct ProcGenSeed(pub u64);

impl ProcGenSeed {
    fn random() -> Self {
        Self(rand::random())
    }

    pub fn specs(&self) -> DungeonSpecs {
        DungeonSpecs {
            seed: self.0,
            ..LEVEL_ONE
        }
    }
}

#[derive(Component)]
pub struct RoomPreview;

pub struct Room {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy)]
pub enum TileType {
    Wall,
    Empty,
}

pub struct RoomMap {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Room>,
    pub width: usize,
    pub height: usize,
}

impl Room {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn intersects(&self, other: &Room) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

fn load_room(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let room_wall_image_handle: Handle<Image> =
        asset_server.load("tiles/rocky_stone_cold_16x16.png");
    loading_assets
        .0
        .push(room_wall_image_handle.clone().untyped());
    commands.insert_resource(RoomWallImage(room_wall_image_handle));
}


pub fn map_rooms(settings: DungeonSpecs) -> RoomMap {
    const MAX_ATTEMPTS: usize = 1_000;
    const MIN_WIDTH: usize = 22;
    const MAX_WIDTH: usize = 30;
    const MIN_HEIGHT: usize = 12;
    const MAX_HEIGHT: usize = 18;
    const GAP: f64 = 1.0;

    let mut map = RoomMap {
        tiles: vec![TileType::Wall; settings.map_width * settings.map_height],
        rooms: Vec::new(),
        width: settings.map_width,
        height: settings.map_height,
    };

    let mut rng = StdRng::seed_from_u64(settings.seed);

    for _ in 0..MAX_ATTEMPTS {
        if map.rooms.len() >= settings.room_count {
            break;
        }

        let width = rng.gen_range(MIN_WIDTH..=MAX_WIDTH);
        let height = rng.gen_range(MIN_HEIGHT..=MAX_HEIGHT);
        let x = rng.gen_range(1..=map.width - width - 1);
        let y = rng.gen_range(1..=map.height - height - 1);

        let new_room = Room::new(x as f64, y as f64, width as f64, height as f64);

        let padded_room = Room::new(
            new_room.x - GAP,
            new_room.y - GAP,
            new_room.width + 2.0 * GAP,
            new_room.height + 2.0 * GAP,
        );

        if map.rooms.iter().any(|room| room.intersects(&padded_room)) {
            continue;
        }

        for tile_y in y..y + height {
            for tile_x in x..x + width {
                let index = tile_y * map.width + tile_x;
                map.tiles[index] = TileType::Empty;
            }
        }
        map.rooms.push(new_room);
    }
    map
}

pub fn map_preview(map: &RoomMap, commands: &mut Commands, wall_image: &Handle<Image>) {
    const TILE_SIZE: f32 = 32.0;

    let mut room_color = Color::srgb(40.0 / 255.0, 52.0 / 255.0, 64.0 / 255.0);
    let mut yellow = true;

    for y in 0..map.height {
        for x in 0..map.width {
            let index = y * map.width + x;

            if yellow {
                room_color = Color::srgb(1.0, 1.0, 1.0);
            } //checkpoint
            if x > 20 && 30 > x {
                yellow = false;
            } else {
                yellow = false;
            }

            let sprite = match &map.tiles[index] {
                TileType::Wall => Sprite {
                    image: wall_image.clone(),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                TileType::Empty => Sprite {
                    color: room_color,
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
            };

            commands.spawn((
                sprite,
                Transform::from_xyz(
                    (x as f32 + 0.5) * TILE_SIZE,
                    (y as f32 + 0.5) * TILE_SIZE,
                    0.0,
                ),
                RoomPreview,
            ));
            room_color = Color::srgb(40.0 / 255.0, 52.0 / 255.0, 64.0 / 255.0);
        }
    }
}

// Existing Room Generation button: show the rooms before carving hallways.
fn setup_room_preview(
    mut commands: Commands,
    seed: Res<ProcGenSeed>,
    wall_image: Res<RoomWallImage>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let map = map_rooms(seed.specs());
    map_preview(&map, &mut commands, &wall_image.0);
    frame_camera(&map, window.into_inner(), camera);
}

// Preview Level 1 and Preview Level 2: show the finished playable tile map.
fn setup_level_preview(
    mut commands: Commands,
    state: Res<State<GameState>>,
    wall_image: Res<RoomWallImage>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let settings = match state.get() {
        GameState::PreviewLevelOne => LEVEL_ONE,
        GameState::PreviewLevelTwo => LEVEL_TWO,
        _ => return,
    };

    let active = ActiveLevel::procedural(settings);
    if let Some(map) = active.map.as_ref() {
        map_preview(map, &mut commands, &wall_image.0);
        frame_camera(map, window.into_inner(), camera);
    }
}

pub fn frame_camera(
    map: &RoomMap,
    window: &Window,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let world_width = map.width as f32 * 32.0;
    let world_height = map.height as f32 * 32.0;

    let (mut transform, mut projection) = camera.into_inner();

    transform.translation.x = world_width / 2.0;
    transform.translation.y = world_height / 2.0;

    if let Projection::Orthographic(orthographic) = &mut *projection {
        orthographic.scale = (world_width / window.width().max(1.0))
            .max(world_height / window.height().max(1.0))
            * 1.1;
    }
}

pub struct RoomGenerationPlugin;

impl Plugin for RoomGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProcGenSeed::random())
            .add_systems(Startup, load_room)
            .add_systems(
                OnTransition {
                    exited: GameState::MainMenu,
                    entered: GameState::ProcGenMenu,
                },
                |mut commands: Commands| {
                    commands.insert_resource(ProcGenSeed::random());
                },
            )
            .add_systems(OnEnter(GameState::RoomGeneration), setup_room_preview)
            .add_systems(OnEnter(GameState::PreviewLevelOne), setup_level_preview)
            .add_systems(OnEnter(GameState::PreviewLevelTwo), setup_level_preview)
            .add_systems(
                OnExit(GameState::RoomGeneration),
                despawn_with::<RoomPreview>,
            )
            .add_systems(
                OnExit(GameState::PreviewLevelOne),
                despawn_with::<RoomPreview>,
            )
            .add_systems(
                OnExit(GameState::PreviewLevelTwo),
                despawn_with::<RoomPreview>,
            );
    }
}
