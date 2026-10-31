use bevy::{prelude::*, window::PrimaryWindow};
use rand::{rngs::StdRng, Rng, SeedableRng};
use crate::{
    GameState,
    loading::{LoadingAssets, despawn_with},
};

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

// Shared by all proc gen previews so each stage shows the same dungeon
#[derive(Resource)]
pub struct ProcGenSeed(pub u64);

impl ProcGenSeed {
    fn random() -> Self {
        let seed = rand::random::<u64>();
        info!("Proc gen seed: {seed}");
        Self(seed)
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
        Self { x, y, width, height }
    }

    pub fn intersects(&self, other: &Room) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
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
    let room_wall_image_handle: Handle<Image> = asset_server.load("tiles/rocky_stone_cold_16x16.png");
    loading_assets.0.push(room_wall_image_handle.clone().untyped());
    commands.insert_resource(RoomWallImage(room_wall_image_handle));
}

pub fn map_rooms(seed: u64) -> RoomMap {

    const MAP_WIDTH: usize = 160;
    const MAP_HEIGHT: usize = 120;
    const MAX_ROOMS: usize = 12;
    // Prevent it from looping forever or getting stuck
    const MAX_ATTEMPTS: usize = 1_000;
    const MIN_SIZE: usize = 12;
    const MAX_SIZE: usize = 20;
    const GAP: f64 = 1.0;

    let mut map = RoomMap {
        tiles: vec![TileType::Wall; MAP_WIDTH * MAP_HEIGHT],
        rooms: Vec::new(),
        width: MAP_WIDTH,
        height: MAP_HEIGHT,
    };

    let mut rng = StdRng::seed_from_u64(seed);

    for _ in 0..MAX_ATTEMPTS {
        if map.rooms.len() >= MAX_ROOMS {
            break;
        }

        let width = rng.gen_range(MIN_SIZE..=MAX_SIZE);
        let height = rng.gen_range(MIN_SIZE..=MAX_SIZE);
        let x = rng.gen_range(1..=MAP_WIDTH - width - 1);
        let y = rng.gen_range(1..=MAP_HEIGHT - height - 1);

        let new_room = Room::new(
            x as f64, 
            y as f64, 
            width as f64, 
            height as f64
        );
        
        let padded_room = Room::new(
            new_room.x - GAP,
            new_room.y - GAP,
            new_room.width + 2.0 * GAP,
            new_room.height + 2.0 * GAP,
        );

        if map.rooms.iter().any(|room| room.intersects(&padded_room)) {
            continue;
        }

        // Carve out the room in the map
        for tile_y in y..y + height {
            for tile_x in x..x + width {
                let index = tile_y * MAP_WIDTH + tile_x;
                map.tiles[index] = TileType::Empty;
            }
        }
        map.rooms.push(new_room);
    }

    map
}

pub fn map_preview(
    map: &RoomMap,
    commands: &mut Commands,
    wall_image: &Handle<Image>,
) {
    const TILE_SIZE: f32 = 32.0;

    let mut room_color = Color::srgb(40.0/255.0, 52.0/255.0, 64.0/255.0,);
    let mut yellow = true;

    for y in 0..map.height {
        for x in 0..map.width {
            let index = y * map.width + x;

            if yellow { room_color = Color::srgb(1.0, 1.0, 1.0);} //checkpoint
            if x > 20 && 30 > x {
                yellow = false;
            }else{
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
            room_color = Color::srgb(40.0/255.0, 52.0/255.0, 64.0/255.0,);
        }
    }
}

fn setup_room_preview(
    mut commands: Commands,
    wall_image: Res<RoomWallImage>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
    seed: Res<ProcGenSeed>,
) {
    let map = map_rooms(seed.0);
    map_preview(&map, &mut commands, &wall_image.0);
    frame_camera(&map, window.into_inner(), camera);
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
        orthographic.scale = (
            world_width / window.width().max(1.0)
        )
            .max(world_height / window.height().max(1.0))
            * 1.1;
    }
}

pub struct RoomGenerationPlugin;

impl Plugin for RoomGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProcGenSeed::random())
            .add_systems(Startup, load_room)
            // New dungeon each time the demo is opened from the main menu,
            // but not when backing out of a preview to the proc gen menu
            .add_systems(
                OnTransition {
                    exited: GameState::MainMenu,
                    entered: GameState::ProcGenMenu,
                },
                |mut commands: Commands| commands.insert_resource(ProcGenSeed::random()),
            )
            .add_systems(
                OnEnter(GameState::RoomGeneration),
                setup_room_preview,
            )
            .add_systems(
                OnExit(GameState::RoomGeneration),
                despawn_with::<RoomPreview>,
            );
    }
}