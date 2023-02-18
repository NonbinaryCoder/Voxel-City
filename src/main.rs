use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod camera;
mod menu;
mod terrain;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(terrain::TerrainPlugin)
        .run();
}
