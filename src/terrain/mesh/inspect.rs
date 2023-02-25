use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext};

use crate::terrain::{tile::Tile, GlobalPos, Terrain};

pub struct InspectPlugin;

impl Plugin for InspectPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WireframePlugin)
            .add_system(inspect_mesh_system);
    }
}

fn inspect_mesh_system(
    mut egui_context: ResMut<EguiContext>,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut terrain: ResMut<Terrain>,
) {
    egui::Window::new("Mesh Inspector")
        .open(&mut true)
        .default_width(200.0)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Terrain");

                if ui.button("Clear").clicked() {
                    terrain.clear();
                };

                if ui.button("Tower").clicked() {
                    terrain.clear();
                    for y in -24..=24 {
                        terrain.set(GlobalPos::from_xyz_i32(IVec3::new(1, y, 1)), Tile::Brick);
                    }
                }
            });

            ui.separator();

            ui.checkbox(&mut wireframe_config.global, "Wireframe");
        });
}
