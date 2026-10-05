use bevy::prelude::*;

use crate::{
    GameState, LEVEL_BOTTOM, LEVEL_LEFT, LEVEL_RIGHT, LEVEL_TOP, WIN_H, WIN_W,
};

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
        app.add_systems(Startup, setup_camera).add_systems(
            Update,
            follow_target.run_if(in_state(GameState::Playing)),
        );
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

fn follow_target(
    time: Res<Time>,
    target: Single<&Transform, (With<CameraTarget>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    let goal = Vec2::new(
        clamp_axis(target.translation.x, LEVEL_LEFT, LEVEL_RIGHT, WIN_W * CAMERA_SCALE),
        clamp_axis(target.translation.y, LEVEL_BOTTOM, LEVEL_TOP, WIN_H * CAMERA_SCALE),
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
