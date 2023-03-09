use bevy_egui::egui::{self, Widget};

use crate::terrain::chunk::TileSlot;

use super::Tile;

impl Tile {
    pub fn widget(&mut self) -> impl Widget + '_ {
        move |ui: &mut egui::Ui| {
            ui.vertical_centered_justified(|ui| {
                let brick = ui.selectable_label(self.is_brick(), "Brick");
                if brick.clicked() && !self.is_brick() {
                    *self = Tile::BRICK;
                }
                let mut response = brick;

                match self {
                    Tile::Brick { color } => response |= ui.add(color),
                }

                response
            })
            .inner
        }
    }

    pub fn widget_option(tile_slot: &mut TileSlot) -> impl Widget + '_ {
        move |ui: &mut egui::Ui| {
            ui.vertical_centered_justified(|ui| {
                let mut response = ui.selectable_value(tile_slot, None, "Air");
                if let Some(tile) = tile_slot {
                    response |= ui.add(tile.widget());
                } else {
                    // Must match `Tile::widget` or button becomes deselected
                    // after it is pressed
                    ui.vertical_centered_justified(|ui| {
                        let brick = ui.selectable_label(false, "Brick");
                        if brick.clicked() {
                            *tile_slot = Some(Tile::BRICK);
                            response.mark_changed();
                        }
                        response |= brick;
                    })
                    .inner
                }
                response
            })
            .inner
        }
    }
}
