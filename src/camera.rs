use bevy::prelude::*;
use bevy_flycam::{FlyCam, MovementSettings};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_flycam::NoCameraPlayerPlugin)
            .insert_resource(MovementSettings {
                sensitivity: 0.00004,
                ..default()
            })
            .add_startup_system(spawn_camera_system);
    }
}

fn spawn_camera_system(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::Z * 12.0),
            ..default()
        },
        FlyCam,
    ));
}
