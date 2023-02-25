use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext};

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
) {
    egui::Window::new("Mesh Inspector")
        .open(&mut true)
        .default_width(200.0)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.checkbox(&mut wireframe_config.global, "Wireframe");
        });
}
