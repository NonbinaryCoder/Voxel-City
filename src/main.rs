use bevy::prelude::*;

mod camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(camera::CameraPlugin)
        .run();
}
