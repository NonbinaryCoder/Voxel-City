use crate::terrain::mesh::{
    mesh_builder::{Face, MeshBuilder, SubtileFace, WallFlags},
    CornerTiles, Subtile,
};

use super::Tile;

impl Tile {
    pub fn generate_mesh(tiles: &CornerTiles, subtile: Subtile, mesh: &mut MeshBuilder) {
        let tile = tiles[subtile];
        match tile {
            None => {}
            Some(Tile::Brick) => {
                for face in SubtileFace::faces() {
                    let adj_pos = subtile.tile_at_face(face);
                    let adj_tile = tiles[adj_pos];
                    if adj_tile != tile {
                        mesh.add(
                            Face::Wall(
                                WallFlags::CONNECTED_ALL
                                    .set_facing_positive(subtile, face.subtile_axis()),
                            ),
                            subtile,
                            face,
                        );
                    }
                }
            }
        }
    }
}
