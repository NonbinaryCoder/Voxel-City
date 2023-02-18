use std::fmt::{Debug, Display};

use bevy::{prelude::*, utils::HashMap};

use self::{
    chunk::{Chunk, ChunkPos, Cleanup, LocalPos, TileSlot, CHUNK_WIDTH},
    tile::Tile,
};

mod chunk;
mod inspect;
mod tile;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(inspect::InspectPlugin)
            .init_resource::<Terrain>();
    }
}

#[derive(Debug, Default, Resource)]
pub struct Terrain {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl Terrain {
    pub fn get(&self, pos: GlobalPos) -> Option<Tile> {
        self.chunks
            .get(&pos.chunk)
            .and_then(|chunk| chunk[pos.local])
    }

    pub fn set(&mut self, pos: GlobalPos, tile: Tile) {
        let chunk = self.chunks.entry(pos.chunk).or_default();
        let cleanup = chunk.set(pos.local, tile);
        self.cleanup(pos, cleanup);
    }

    pub fn remove(&mut self, pos: GlobalPos) {
        if let Some(chunk) = self.chunks.get_mut(&pos.chunk) {
            let cleanup = chunk.remove(pos.local);
            self.cleanup(pos, cleanup);
        }
    }

    pub fn set_slot(&mut self, pos: GlobalPos, slot: TileSlot) {
        match slot {
            Some(tile) => self.set(pos, tile),
            None => self.remove(pos),
        }
    }

    pub fn cleanup(&mut self, pos: GlobalPos, cleanup: Cleanup) {
        match cleanup {
            Cleanup::None => {}
            Cleanup::RemoveChunk => drop(self.chunks.remove(&pos.chunk)),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalPos {
    chunk: ChunkPos,
    local: LocalPos,
}

impl GlobalPos {
    pub fn x(self) -> i64 {
        self.chunk.x as i64 * CHUNK_WIDTH as i64 + self.local.x() as i64
    }

    pub fn y(self) -> i64 {
        self.chunk.y as i64 * CHUNK_WIDTH as i64 + self.local.y() as i64
    }

    pub fn z(self) -> i64 {
        self.chunk.z as i64 * CHUNK_WIDTH as i64 + self.local.z() as i64
    }

    pub fn xyz(self) -> [i64; 3] {
        [self.x(), self.y(), self.z()]
    }

    pub fn from_xyz(xyz: [i64; 3]) -> GlobalPos {
        let xyz = xyz.map(|v| v.clamp(i32::MIN as i64, i32::MAX as i64) as i32);
        Self {
            chunk: xyz.map(|v| v.div_euclid(CHUNK_WIDTH as i32)).into(),
            local: LocalPos::new(xyz.map(|v| v.rem_euclid(CHUNK_WIDTH as i32) as u8)).unwrap(),
        }
    }

    pub fn from_xyz_i32(xyz: IVec3) -> GlobalPos {
        Self {
            chunk: xyz
                .to_array()
                .map(|v| v.div_euclid(CHUNK_WIDTH as i32))
                .into(),
            local: LocalPos::new(
                xyz.to_array()
                    .map(|v| v.rem_euclid(CHUNK_WIDTH as i32) as u8),
            )
            .unwrap(),
        }
    }
}

impl Debug for GlobalPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(GlobalPos))
            .field("chunk", &self.chunk.to_array())
            .field("local", &self.local.xyz())
            .field("actual", &self.xyz())
            .finish()
    }
}

impl Display for GlobalPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.xyz())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::{tile::Tile, GlobalPos, Terrain};

    prop_compose! {
        fn arb_global_pos()(xyz: [i32; 3]) -> GlobalPos {
            GlobalPos::from_xyz_i32(xyz.into())
        }
    }

    proptest! {
        #[test]
        fn add_remove_from_terrain(pos in arb_global_pos()) {
            let mut terrain = Terrain::default();

            terrain.set(pos, Tile::Brick);
            assert_eq!(terrain.get(pos), Some(Tile::Brick));
            assert_eq!(terrain.chunks.len(), 1);

            terrain.remove(pos);
            assert_eq!(terrain.get(pos), None);
            assert_eq!(terrain.chunks.len(), 0);
        }

        #[test]
        fn add_remove_many_from_terrain(
            poses in proptest::collection::vec(arb_global_pos(), 1..256)
        ) {
            let mut terrain = Terrain::default();

            for &pos in &poses {
                terrain.set(pos, Tile::Brick);
            }
            assert!(!terrain.chunks.is_empty());

            for &pos in &poses {
                terrain.remove(pos);
            }
            assert_eq!(terrain.chunks.len(), 0);
        }

        #[test]
        fn add_remove_many_from_terrain_small(
            poses in proptest::collection::vec([-1..=1, -1..=1, -1..=1], 1..32)
        ) {
            let poses = poses
                .into_iter()
                .map(|xyz| GlobalPos::from_xyz_i32(xyz.into()))
                .collect::<Vec<_>>();
            let mut terrain = Terrain::default();

            for &pos in &poses {
                terrain.set(pos, Tile::Brick);
            }
            assert!(!terrain.chunks.is_empty());

            for &pos in &poses {
                terrain.remove(pos);
            }
            assert_eq!(terrain.chunks.len(), 0);
        }

        #[test]
        fn global_pos_xyz(xyz: [i32; 3]) {
            let global_pos = GlobalPos::from_xyz_i32(xyz.into());
            assert_eq!(xyz.map(|v| v as i64), global_pos.xyz())
        }
    }
}
