use bevy::prelude::*;

use crate::{
    GameState, LEVEL_H, LEVEL_LEN, TILE_SIZE, WIN_H, WIN_W,
    loading::{LoadingAssets, despawn_with},
};

use crate::{
    LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_RIGHT, LEVEL_TOP,
    player::PLAYER_SIZE,
    proc_gen::{
        dungeon_generation::{DungeonLayout, generate_dungeon},
        enemy_placement::{EnemySpawns, place_enemies},
        room_generation::{RoomMap, TileType},
    },
};

#[derive(Component)]
pub struct Brick;

#[derive(Component)]
pub struct Background;

// Wall tile drawn for looks only: fully surrounded by other walls, so it has no Brick/collision
#[derive(Component)]
pub struct BuriedTile;

#[derive(Resource)]
pub struct BackgroundImage(Handle<Image>);
#[derive(Resource)]
pub struct BrickImage(Handle<Image>);

#[derive(Resource)]
pub struct ActiveLevel {
    pub map: Option<RoomMap>,
    pub spawn: Vec2,
    pub bounds_min: Vec2,
    pub bounds_max: Vec2,
    // Where to spawn enemies on a procedural map (empty for the intro level)
    pub enemies: EnemySpawns,
    pub lamps: Vec<Vec2>,
}

pub struct LevelPlugin;
impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_level)
            .add_systems(OnEnter(GameState::Playing), setup_level)
            .add_systems(
                OnExit(GameState::Playing),
                (
                    despawn_with::<Brick>,
                    despawn_with::<BuriedTile>,
                    despawn_with::<Background>,
                ),
            );
    }
}

fn load_level(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let bg_texture_handle = asset_server.load("backgrounds/noisey_background1.png");

    loading_assets.0.push(bg_texture_handle.clone().untyped());
    commands.insert_resource(BackgroundImage(bg_texture_handle));

    let brick_image_handle: Handle<Image> = asset_server.load("tiles/rocky_stone_cold_16x16.png");
    loading_assets.0.push(brick_image_handle.clone().untyped());

    commands.insert_resource(BrickImage(brick_image_handle));
}

fn setup_level(
    mut commands: Commands,
    background_image: Res<BackgroundImage>,
    brick_image: Res<BrickImage>,
    active: Res<ActiveLevel>,
) {
    if let Some(map) = &active.map {
        let size = active.bounds_max - active.bounds_min;
        let center = (active.bounds_min + active.bounds_max) / 2.;

        commands.spawn((
            Sprite {
                color: Color::srgb(0.15, 0.20, 0.25),
                custom_size: Some(size),
                ..default()
            },
            Transform::from_xyz(center.x, center.y, -1.),
            Background,
        ));

        for y in 0..map.height {
            for x in 0..map.width {
                if !matches!(map.tiles[y * map.width + x], TileType::Wall) {
                    continue;
                }
                let translation = Vec3::new(
                    (x as f32 + 0.5) * TILE_SIZE,
                    (y as f32 + 0.5) * TILE_SIZE,
                    1.,
                );

                // Every collision system loops over all Bricks, and ~90% of wall tiles are buried in
                // solid rock where nothing can reach them. Draw those but leave them out of collision.
                if touches_open_space(map, x, y) {
                    spawn_tile(&mut commands, &brick_image, translation);
                } else {
                    commands.spawn((
                        Sprite {
                            image: brick_image.0.clone(),
                            custom_size: Some(Vec2::splat(TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(translation),
                        BuriedTile,
                    ));
                }
            }
        }

        return;
    }
    let mut y_offset = 0.;
    while y_offset < LEVEL_H {
        let mut x_offset = 0.;
        while x_offset < LEVEL_LEN {
            commands.spawn((
                Sprite::from_image(background_image.0.clone()),
                Transform::from_xyz(x_offset, y_offset, 0.),
                Background,
            ));

            x_offset += WIN_W;
        }

        y_offset += WIN_H;
    }

    // Spawn floor
    let mut i = 0;
    let mut t = Vec3::new(
        -WIN_W / 2. + TILE_SIZE / 2.,
        -WIN_H / 2. + TILE_SIZE / 2.,
        1.,
    );
    while (i as f32) * TILE_SIZE < LEVEL_LEN {
        commands
            .spawn((
                Sprite {
                    image: brick_image.0.clone(),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform {
                    translation: t,
                    ..default()
                },
                Brick,
            ))
            .insert(Brick);

        i += 1;
        t += Vec3::new(TILE_SIZE, 0., 0.);
    }

    // Spawn some obstacle tiles

    // Tiles are 32 units in world/game space, despite being 16 pixel units in resolution

    // Left tower to player 2 blocks tall part 1
    t = Vec3::new(-160., (-WIN_H / 2.) + TILE_SIZE + 16., 1.);
    spawn_tile(&mut commands, &brick_image, t);
    t = Vec3::new(-160., (-WIN_H / 2.) + TILE_SIZE + 48., 1.);
    spawn_tile(&mut commands, &brick_image, t);
    // Left tower part 2
    t += Vec3::new(32., -32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    t += Vec3::new(0., 32., 0.);
    spawn_tile(&mut commands, &brick_image, t);

    // uh quick reset for t
    t = Vec3::new(48., (-WIN_H / 2.) + TILE_SIZE + 16., 1.);

    // Right tower 2 blocks tall part 1
    t += Vec3::new(256., 0., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    t += Vec3::new(0., 32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    // Right tower part 2
    t += Vec3::new(32., -32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    t += Vec3::new(0., 32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    // Right tower part 3
    t += Vec3::new(32., -32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
    t += Vec3::new(0., 32., 0.);
    spawn_tile(&mut commands, &brick_image, t);
}

// True if any of the 8 tiles around (x, y) is open. Off the map counts as solid.
// Diagonals matter: an entity's box can clip a wall's corner from a diagonal tile.
fn touches_open_space(map: &RoomMap, x: usize, y: usize) -> bool {
    (-1..=1).any(|dy: i32| {
        (-1..=1).any(|dx: i32| {
            let (nx, ny) = (x as i32 + dx, y as i32 + dy);
            nx >= 0
                && ny >= 0
                && (nx as usize) < map.width
                && (ny as usize) < map.height
                && matches!(map.tiles[ny as usize * map.width + nx as usize], TileType::Empty)
        })
    })
}

// Adapted from CS1666_examples
pub fn spawn_tile(commands: &mut Commands, brick_image: &Res<BrickImage>, translation: Vec3) {
    commands
        .spawn((
            Sprite {
                image: brick_image.0.clone(),
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            Transform {
                translation: translation,
                ..default()
            },
            Brick,
        ))
        .insert(Brick);
}

impl Default for ActiveLevel {
    fn default() -> Self {
        Self {
            map: None,
            spawn: Vec2::new(0., LEVEL_BOTTOM + TILE_SIZE + PLAYER_SIZE.y / 2.),
            bounds_min: Vec2::new(LEVEL_LEFT, LEVEL_BOTTOM),
            bounds_max: Vec2::new(LEVEL_RIGHT, LEVEL_TOP),
            enemies: EnemySpawns::default(),
            lamps: vec![Vec2::new(4. * TILE_SIZE, LEVEL_BOTTOM + TILE_SIZE + 16.)],
        }
    }
}

impl ActiveLevel {
    pub fn procedural(seed: u64) -> Self {
        let DungeonLayout {
            mut map,
            vert_hallways,
            horz_hallways,
            ..
        } = generate_dungeon(seed);
        info!("Playable dungeon seed: {seed}");

        // Choose the room nearest the map's upper-left corner.
        let (start_room, room) = map
            .rooms
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                let a_distance = a.x.powi(2) + (map.height as f64 - a.y - a.height).powi(2);

                let b_distance = b.x.powi(2) + (map.height as f64 - b.y - b.height).powi(2);

                a_distance.total_cmp(&b_distance)
            })
            .expect("Dungeon generation produced no rooms");

        let spawn_x = room.x as usize + 3;
        let floor_y = room.y as usize;

        // Reserve a small landing pad with room for the 40×64 player.
        for x in spawn_x - 1..=spawn_x + 1 {
            map.tiles[(floor_y - 1) * map.width + x] = TileType::Wall;

            for y in floor_y..floor_y + 3 {
                map.tiles[y * map.width + x] = TileType::Empty;
            }
        }

        let bounds_max = Vec2::new(map.width as f32 * TILE_SIZE, map.height as f32 * TILE_SIZE);

        // After the landing pad so enemies see the final tiles; the start room stays empty
        let enemies = place_enemies(&map, &horz_hallways, &vert_hallways, Some(start_room), seed);
        info!(
            "Placed {} teeth and {} geralds",
            enemies.teeth.len(),
            enemies.geralds.len()
        );
        let lamps: Vec<Vec2> = map
            .rooms
            .iter()
            .map(|room| {
                let x = (room.x + room.width / 2.) as usize;
                let mut y = room.y as usize;
                // Terrain hills can raise the floor, so climb until the tile is open
                while matches!(map.tiles[y * map.width + x], TileType::Wall) {
                    y += 1;
                }
                Vec2::new((x as f32 + 0.5) * TILE_SIZE, (y as f32 + 0.5) * TILE_SIZE)
            })
            .collect();
        Self {
            spawn: Vec2::new(
                (spawn_x as f32 + 0.5) * TILE_SIZE,
                floor_y as f32 * TILE_SIZE + PLAYER_SIZE.y / 2.,
            ),
            bounds_min: Vec2::ZERO,
            bounds_max,
            map: Some(map),
            enemies,
            lamps,
        }
    }
}

pub fn intro_only(active: Res<ActiveLevel>) -> bool {
    active.map.is_none()
}
