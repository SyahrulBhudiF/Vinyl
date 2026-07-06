use bevy::prelude::*;

/// Marker for the VN 2D camera.
#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub struct VnCamera;

pub(crate) fn spawn_camera(mut commands: Commands, cameras: Query<Entity, With<VnCamera>>) {
    if cameras.is_empty() {
        commands.spawn((VnCamera, Camera2d));
    }
}
