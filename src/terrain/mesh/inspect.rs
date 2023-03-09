use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};
use bevy_egui::{egui, EguiContext};

use crate::terrain::{
    tile::{color::IndexedColor, Tile},
    GlobalPos, Terrain,
};

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
    mut cube_size: Local<isize>,
    mut tile_type: Local<Tile>,
) {
    egui::Window::new("Mesh Inspector")
        .open(&mut true)
        .default_width(200.0)
        .resizable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Terrain");

                ui.add(Tile::widget(&mut tile_type));

                ui.add_space(4.0);

                if ui.button("Clear").clicked() {
                    terrain.clear();
                };

                if ui.button("Tower").clicked() {
                    terrain.clear();
                    for y in -24..=24 {
                        terrain.set(GlobalPos::from_xyz_i32(IVec3::new(1, y, 1)), *tile_type);
                    }
                }

                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut *cube_size).clamp_range(1..=64));
                    ui.centered_and_justified(|ui| {
                        if ui.button("Cube").clicked() {
                            terrain.clear();
                            for x in -4..(*cube_size as i32 - 4) {
                                for y in -4..(*cube_size as i32 - 4) {
                                    for z in -4..(*cube_size as i32 - 4) {
                                        terrain.set(GlobalPos::from_xyz_i32([x, y, z]), *tile_type);
                                    }
                                }
                            }
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut *cube_size).clamp_range(1..=64));
                    ui.centered_and_justified(|ui| {
                        if ui.button("Cube Outline").clicked() {
                            terrain.clear();
                            let size = *cube_size as i32 - 4;
                            for x in -4..=size {
                                terrain.set(GlobalPos::from_xyz_i32([x, -4, -4]), *tile_type);
                                terrain.set(GlobalPos::from_xyz_i32([x, -4, size]), *tile_type);
                                terrain.set(GlobalPos::from_xyz_i32([x, size, -4]), *tile_type);
                                terrain.set(GlobalPos::from_xyz_i32([x, size, size]), *tile_type);
                            }
                            for y in -4..=size {
                                terrain.set(GlobalPos::from_xyz_i32([-4, y, -4]), *tile_type);
                                terrain.set(GlobalPos::from_xyz_i32([-4, y, size]), *tile_type);
                                terrain.set(GlobalPos::from_xyz_i32([size, y, -4]), *tile_type);
                                terrain.set(GlobalPos::from_xyz_i32([size, y, size]), *tile_type);
                            }
                            for z in -4..=size {
                                terrain.set(GlobalPos::from_xyz_i32([-4, -4, z]), *tile_type);
                                terrain.set(GlobalPos::from_xyz_i32([-4, size, z]), *tile_type);
                                terrain.set(GlobalPos::from_xyz_i32([size, -4, z]), *tile_type);
                                terrain.set(GlobalPos::from_xyz_i32([size, size, z]), *tile_type);
                            }
                        }
                    });
                });

                if ui.button("\"MESH\"").clicked() {
                    terrain.clear();
                    let mut draw_line = |mut bits: u32, y, z| {
                        while bits != 0 {
                            let x = bits.trailing_zeros();
                            bits ^= 1 << x;
                            terrain.set(GlobalPos::from_xyz_i32([x as i32, y, z]), *tile_type);
                        }
                    };
                    draw_line(0b10001000000000000, 0, 6);
                    draw_line(0b11011000000000000, 0, 5);
                    draw_line(0b10101011101110101, 0, 4);
                    draw_line(0b10001010001000101, 0, 3);
                    draw_line(0b10001011101110111, 0, 2);
                    draw_line(0b10001010000010101, 0, 1);
                    draw_line(0b10001011101110101, 0, 0);
                    draw_line(0b10001000000000000, 1, 6);
                    draw_line(0b11011000000000000, 1, 5);
                    draw_line(0b10101000000000000, 1, 4);
                    draw_line(0b10001000000000000, 1, 3);
                    draw_line(0b10001000000000000, 1, 2);
                    draw_line(0b10001000000000000, 1, 1);
                    draw_line(0b10001000000000000, 1, 0);
                }

                ui.add_space(4.0);

                if ui.button("Colors").clicked() {
                    terrain.clear();
                    for color in (0..).map_while(IndexedColor::from_index) {
                        let [x, y] = color.uv().map(|v| (v * 8.0 - 0.5) as i32);
                        terrain.set(GlobalPos::from_xyz_i32([x, 0, y]), Tile::Brick { color })
                    }
                }
            });

            ui.separator();

            ui.checkbox(&mut wireframe_config.global, "Wireframe");
        });
}
