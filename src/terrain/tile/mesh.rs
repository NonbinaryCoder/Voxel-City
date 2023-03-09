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
            Some(Tile::Brick { color }) => {
                let uv = color.uv();
                for face in SubtileFace::faces() {
                    let adj_pos = subtile.tile_at_face(face);
                    let adj_tile = tiles[adj_pos];
                    if !Tile::is_solid(adj_tile) {
                        mesh.add(
                            Face::Wall(
                                WallFlags::CONNECTED_ALL
                                    .set_facing_positive(subtile, face.subtile_axis()),
                            ),
                            uv,
                            subtile,
                            face,
                        );
                    }
                }
            }
        }
    }
}
