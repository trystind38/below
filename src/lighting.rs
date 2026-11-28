use bevy::{
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dPlugin},
    window::PrimaryWindow,
};

use crate::{
    GameState, TILE_SIZE,
    burst_meter::POWER_RGB,
    combat::Bullet,
    loading::{LoadingAssets, despawn_with},
    orb::Orb,
    player::{Player, PlayerSystems},
    shining_road::{ROAD_LENGTH, ROAD_WIDTH, RoadBeam},
    stats::{Enemy, Health, HitCooldown, Invulnerable},
    flytrap::Fireball,
};

const SHADER_PATH: &str = "shaders/darkness.wgsl";
const MAX_LIGHTS: usize = 32;
// Below the light burst (z = 50) so the burst still draws on top of the dark
const DARKNESS_Z: f32 = 40.;
const DARKNESS: f32 = 0.68;
const DARKNESS_RGB: Vec3 = Vec3::ZERO;
const VIEW_MARGIN: f32 = 1.2;

const PLAYER_LIGHT_RADIUS: f32 = TILE_SIZE * 6.;
const LIGHT_EASE: f32 = 4.;
const PLAYER_FLICKER: f32 = 0.02;

const BULLET_LIGHT: f32 = 0.7;
const BULLET_LIGHT_RADIUS: f32 = 40.;
const FIREBALL_LIGHT: f32 = 0.9;
const FIREBALL_LIGHT_RADIUS: f32 = 72.;
const FIREBALL_FLICKER: f32 = 0.08;
const FIREBALL_FALLBACK: Vec3 = Vec3::new(1., 0.45, 0.1);
const ROAD_LIGHT: f32 = 0.9;
const ORB_LIGHT: f32 = 0.8;
const ORB_LIGHT_RADIUS: f32 = 64.;
const ORB_FLICKER: f32 = 0.05;
const ORB_RGB: Vec3 = Vec3::new(1., 0.95, 0.8);

const BURN_DAMAGE: f32 = 1.;
const BURN_INTERVAL: f32 = 0.5;
const BURN_FLASH: f32 = 0.5;
const BURN_FLASH_TIME: f32 = 0.12;

#[derive(Resource, Default)]
pub struct PlayerLight {
    pub radius: f32,
}

#[derive(Resource)]
struct FireballGlow {
    image: Handle<Image>,
    color: Option<Vec3>,
}

#[derive(Component)]
struct Darkness;

#[derive(Component)]
struct Burn {
    timer: f32,
    flash: f32,
}

#[derive(ShaderType, Clone, Copy, Default, Debug)]
struct ShaderLight {
    a: Vec2,
    b: Vec2,
    color: Vec4,
    radius: f32,
}

#[derive(ShaderType, Clone, Default, Debug)]
struct DarknessUniform {
    view_min: Vec2,
    view_size: Vec2,
    viewport: Vec2,
    count: u32,
    ambient: Vec4,
    lights: [ShaderLight; MAX_LIGHTS],
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
struct DarknessMaterial {
    #[uniform(0)]
    lighting: DarknessUniform,
}

impl Material2d for DarknessMaterial {
    fn fragment_shader() -> ShaderRef {
        SHADER_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

pub struct LightingPlugin;
impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<DarknessMaterial>::default())
            .init_resource::<PlayerLight>()
            .add_systems(Startup, load_fireball_glow)
            .add_systems(OnEnter(GameState::Playing), spawn_darkness)
            .add_systems(
                Update,
                (sample_fireball_color, update_player_light, burn_enemies)
                    .chain()
                    .after(PlayerSystems)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                (fit_darkness_to_camera, update_lights, show_burn_flash)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), despawn_with::<Darkness>);
    }
}

fn load_fireball_glow(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading_assets: ResMut<LoadingAssets>,
) {
    let image: Handle<Image> = asset_server.load("enemy/fireball.png");
    loading_assets.0.push(image.clone().untyped());
    commands.insert_resource(FireballGlow { image, color: None });
}

fn spawn_darkness(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<DarknessMaterial>>,
    mut light: ResMut<PlayerLight>,
) {
    light.radius = PLAYER_LIGHT_RADIUS;
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(1., 1.))),
        MeshMaterial2d(materials.add(DarknessMaterial {
            lighting: DarknessUniform::default(),
        })),
        Transform::from_xyz(0., 0., DARKNESS_Z),
        Darkness,
    ));
}

fn sample_fireball_color(images: Res<Assets<Image>>, mut glow: ResMut<FireballGlow>) {
    if glow.color.is_some() {
        return;
    }
    let Some(image) = images.get(&glow.image) else {
        return;
    };

    let mut sum = Vec3::ZERO;
    let mut weight = 0.;
    for y in 0..image.height() {
        for x in 0..image.width() {
            let Ok(color) = image.get_color_at(x, y) else {
                continue;
            };
            let c = color.to_linear();
            let rgb = Vec3::new(c.red, c.green, c.blue);
            let w = c.alpha * rgb.max_element();
            sum += rgb * w;
            weight += w;
        }
    }

    let average = if weight > 0. { sum / weight } else { FIREBALL_FALLBACK };
    glow.color = Some(average / average.max_element().max(0.001));
}

fn update_player_light(
    time: Res<Time>,
    player: Single<&Health, With<Player>>,
    mut light: ResMut<PlayerLight>,
) {
    let fraction = (player.current / player.max).clamp(0., 1.);
    let target = PLAYER_LIGHT_RADIUS * fraction.sqrt();
    let step = 1. - (-LIGHT_EASE * time.delta_secs()).exp();
    light.radius += (target - light.radius) * step;
    if (target - light.radius).abs() < 0.5 {
        light.radius = target;
    }
}

fn burn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    light: Res<PlayerLight>,
    player: Single<&Transform, With<Player>>,
    mut enemies: Query<
        (Entity, &Transform, &mut Health, Option<&mut Burn>),
        (With<Enemy>, Without<Invulnerable>, Without<Player>),
    >,
) {
    let dt = time.delta_secs();
    let center = player.translation.truncate();

    for (entity, tf, mut health, burn) in &mut enemies {
        let lit = health.current > 0. && tf.translation.truncate().distance(center) <= light.radius;
        match burn {
            Some(mut burn) if lit => {
                burn.timer -= dt;
                if burn.timer <= 0. {
                    burn.timer += BURN_INTERVAL;
                    burn.flash = BURN_FLASH_TIME;
                    health.current = (health.current - BURN_DAMAGE).max(0.);
                }
            }
            Some(mut burn) => burn.timer = BURN_INTERVAL,
            None if lit => {
                commands.entity(entity).try_insert(Burn {
                    timer: BURN_INTERVAL,
                    flash: 0.,
                });
            }
            None => {}
        }
    }
}

fn fit_darkness_to_camera(
    camera: Single<(&Transform, &Projection), (With<Camera2d>, Without<Darkness>)>,
    mut darkness: Single<&mut Transform, With<Darkness>>,
) {
    let (cam_tf, projection) = camera.into_inner();
    let Projection::Orthographic(ortho) = projection else {
        return;
    };
    let center = cam_tf.translation.truncate() + ortho.area.center();
    darkness.translation = center.extend(DARKNESS_Z);
    darkness.scale = (ortho.area.size() * VIEW_MARGIN).extend(1.);
}

fn update_lights(
    time: Res<Time>,
    light: Res<PlayerLight>,
    glow: Res<FireballGlow>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Transform, &Projection), With<Camera2d>>,
    darkness: Single<&MeshMaterial2d<DarknessMaterial>, With<Darkness>>,
    mut materials: ResMut<Assets<DarknessMaterial>>,
    player: Single<&Transform, With<Player>>,
    beams: Query<&Transform, With<RoadBeam>>,
    fireballs: Query<(Entity, &Transform), With<Fireball>>,
    bullets: Query<(&Transform, &Sprite), With<Bullet>>,
    orbs: Query<(Entity, &Transform), With<Orb>>,
) {
    let (cam_tf, projection) = camera.into_inner();
    let Projection::Orthographic(ortho) = projection else {
        return;
    };
    let view_min = cam_tf.translation.truncate() + ortho.area.min;
    let view_max = cam_tf.translation.truncate() + ortho.area.max;
    let now = time.elapsed_secs();

    let mut uniform = DarknessUniform {
        view_min,
        view_size: view_max - view_min,
        viewport: Vec2::new(window.physical_width() as f32, window.physical_height() as f32),
        ambient: DARKNESS_RGB.extend(DARKNESS),
        ..default()
    };
    let mut add = |a: Vec2, b: Vec2, color: Vec3, intensity: f32, radius: f32| {
        let on_screen =
            a.min(b).cmplt(view_max + radius).all() && a.max(b).cmpgt(view_min - radius).all();
        let slot = uniform.count as usize;
        if radius > 0. && on_screen && slot < MAX_LIGHTS {
            uniform.lights[slot] = ShaderLight {
                a,
                b,
                color: color.extend(intensity),
                radius,
            };
            uniform.count += 1;
        }
    };

    let power = linear(Color::srgb(POWER_RGB[0], POWER_RGB[1], POWER_RGB[2]));
    let center = player.translation.truncate();
    add(center, center, power, 1., light.radius * flicker(now, 0., PLAYER_FLICKER));

    for tf in &beams {
        let half = Vec2::X * ROAD_LENGTH / 2.;
        let mid = tf.translation.truncate();
        add(mid - half, mid + half, power, ROAD_LIGHT * tf.scale.y, ROAD_WIDTH);
    }

    let fire = glow.color.unwrap_or(FIREBALL_FALLBACK);
    for (entity, tf) in &fireballs {
        let pos = tf.translation.truncate();
        let f = flicker(now, (entity.to_bits() % 997) as f32, FIREBALL_FLICKER);
        add(pos, pos, fire, FIREBALL_LIGHT * f, FIREBALL_LIGHT_RADIUS * f);
    }

    for (tf, sprite) in &bullets {
        let pos = tf.translation.truncate();
        add(pos, pos, linear(sprite.color), BULLET_LIGHT, BULLET_LIGHT_RADIUS);
    }

    for (entity, tf) in &orbs {
        let pos = tf.translation.truncate();
        let f = flicker(now, (entity.to_bits() % 997) as f32, ORB_FLICKER);
        add(pos, pos, ORB_RGB, ORB_LIGHT * f, ORB_LIGHT_RADIUS * f);
    }

    if let Some(mut material) = materials.get_mut(&darkness.0) {
        material.lighting = uniform;
    }
}

fn show_burn_flash(
    time: Res<Time>,
    mut burning: Query<(&mut Burn, &mut Sprite), With<HitCooldown>>,
) {
    let dt = time.delta_secs();
    for (mut burn, mut sprite) in &mut burning {
        if burn.flash <= 0. {
            continue;
        }
        let boost = 1. + BURN_FLASH * burn.flash / BURN_FLASH_TIME;
        burn.flash = (burn.flash - dt).max(0.);
        let c = sprite.color.to_linear();
        sprite.color =
            LinearRgba::new(c.red * boost, c.green * boost, c.blue * boost, c.alpha).into();
    }
}

fn flicker(now: f32, phase: f32, amount: f32) -> f32 {
    1. + amount * (0.6 * (now * 7.1 + phase).sin() + 0.4 * (now * 12.7 + phase * 1.7).sin())
}

fn linear(color: Color) -> Vec3 {
    let c = color.to_linear();
    Vec3::new(c.red, c.green, c.blue)
}