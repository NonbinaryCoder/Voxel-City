use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera_system);
    }
}

fn spawn_camera_system(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::splat(-8.0)).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
