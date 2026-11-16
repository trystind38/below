use bevy::prelude::*;

use crate::{GameState, WIN_H, WIN_W, level::ActiveLevel};

// How quickly thee camera catches up to its target. Higher is snappier.
const CAMERA_DECAY: f32 = 7.;
const CAMERA_SCALE: f32 = 5. / 6.;

/// Add this to the entity the camera should follow (e.g. the player).
/// If no entity has it, the camera stays where it is.
#[derive(Component)]
pub struct CameraTarget;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(OnEnter(GameState::Playing), reset_gameplay_camera)
            .add_systems(Update, follow_target.run_if(in_state(GameState::Playing)));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scale: CAMERA_SCALE,
            ..OrthographicProjection::default_2d()
        }),
    ));
}

fn reset_gameplay_camera(
    active: Res<ActiveLevel>,
    camera: Single<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let (mut transform, mut projection) = camera.into_inner();

    if active.map.is_some() {
        transform.translation.x = clamp_axis(
            active.spawn.x,
            active.bounds_min.x,
            active.bounds_max.x,
            WIN_W * CAMERA_SCALE,
        );

        transform.translation.y = clamp_axis(
            active.spawn.y,
            active.bounds_min.y,
            active.bounds_max.y,
            WIN_H * CAMERA_SCALE,
        );
    } else {
        transform.translation.x = 0.;
        transform.translation.y = 0.;
    }

    if let Projection::Orthographic(orthographic) = &mut *projection {
        orthographic.scale = CAMERA_SCALE;
    }
}

fn follow_target(
    active: Res<ActiveLevel>,
    time: Res<Time>,
    target: Single<&Transform, (With<CameraTarget>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let goal = Vec2::new(
        clamp_axis(
            target.translation.x,
            active.bounds_min.x,
            active.bounds_max.x,
            WIN_W * CAMERA_SCALE,
        ),
        clamp_axis(
            target.translation.y,
            active.bounds_min.y,
            active.bounds_max.y,
            WIN_H * CAMERA_SCALE,
        ),
    );

    let mut pos = camera.translation.truncate();
    pos.smooth_nudge(&goal, CAMERA_DECAY, time.delta_secs());
    camera.translation = pos.extend(camera.translation.z);
}

/// Clamp the camera's center so the view never shows past the level edges.
/// If the level is smaller than the view along this axis, center on the level.
fn clamp_axis(target: f32, min: f32, max: f32, view: f32) -> f32 {
    if max - min <= view {
        (min + max) / 2.
    } else {
        target.clamp(min + view / 2., max - view / 2.)
    }
}
