use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::menu;

use super::{tile::Tile, GlobalPos, Terrain};

pub struct InspectPlugin;

impl Plugin for InspectPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(inspect_tile_system);
    }
}

fn inspect_tile_system(
    mut egui_context: ResMut<EguiContext>,
    mut terrain: ResMut<Terrain>,
    mut pos: Local<GlobalPos>,
) {
    egui::Window::new("Tile Inspector")
        .open(&mut true)
        .default_width(200.0)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.add(&mut *pos);
            ui.separator();
            ui.add(terrain.widget_edit_tile(*pos));
        });
}

impl egui::Widget for &mut GlobalPos {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut xyz = self.xyz();
        let mut response = ui.add(menu::edit_slice(&mut xyz));
        if response.changed() {
            *self = GlobalPos::from_xyz(xyz);
        }

        response |= ui.monospace(format!(
            "Chunk: [{:2}, {:2}, {:2}]",
            self.chunk.x, self.chunk.y, self.chunk.z
        ));
        response |= ui.monospace(format!("Local: {:?}", self.local));
        response
    }
}

impl Terrain {
    pub fn widget_edit_tile(&mut self, pos: GlobalPos) -> impl egui::Widget + '_ {
        move |ui: &mut egui::Ui| {
            ui.vertical_centered_justified(|ui| {
                let mut slot = self.get(pos);
                let response = ui.add(Tile::widget_option(&mut slot));
                if response.changed() {
                    self.set_slot(pos, slot);
                }
                response
            })
            .inner
        }
    }
}
