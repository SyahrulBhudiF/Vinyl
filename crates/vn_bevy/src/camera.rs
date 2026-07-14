use bevy::camera::ScalingMode as CameraScalingMode;
use bevy::prelude::*;

/// Marker for the VN 2D camera.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct VnCamera;

pub(crate) fn spawn_camera(mut commands: Commands, cameras: Query<Entity, With<VnCamera>>) {
    if cameras.is_empty() {
        commands.spawn((
            VnCamera,
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: CameraScalingMode::AutoMin {
                    min_width: 1280.0,
                    min_height: 720.0,
                },
                ..OrthographicProjection::default_2d()
            }),
        ));
    }
}
