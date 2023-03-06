use std::{
    fmt::{Debug, Display},
    ops::{BitAnd, Index},
};

use bevy::prelude::*;

use super::tile::Tile;

pub const CHUNK_WIDTH: u8 = 16;
pub const CHUNK_AREA: usize = (CHUNK_WIDTH as usize).pow(3);

pub type ChunkPos = IVec3;

pub type TileSlot = Option<Tile>;

#[derive(Debug)]
pub struct Chunk {
    data: [TileSlot; CHUNK_AREA],
    set_tiles: u16,
}

macro_rules! slot {
    ($chunk:ident, $pos:ident) => {
        &mut $chunk.data[$pos.0 as usize]
    };
}

impl Chunk {
    pub fn set(&mut self, pos: LocalPos, tile: Tile) -> Cleanup {
        let slot = slot!(self, pos);
        if slot.is_none() {
            self.set_tiles += 1;
        }
        *slot = Some(tile);
        Cleanup::None
    }

    pub fn remove(&mut self, pos: LocalPos) -> Cleanup {
        let slot = slot!(self, pos);
        if slot.is_some() {
            self.set_tiles -= 1;
        }
        *slot = None;
        Cleanup::remove_if_zero(self.set_tiles)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            data: [None; CHUNK_AREA],
            set_tiles: 0,
        }
    }
}

impl Index<LocalPos> for Chunk {
    type Output = TileSlot;

    fn index(&self, index: LocalPos) -> &Self::Output {
        &self.data[index.0 as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub enum Cleanup {
    None,
    RemoveChunk,
}

impl Cleanup {
    pub fn remove_if_zero(c: u16) -> Cleanup {
        match c {
            0 => Cleanup::RemoveChunk,
            _ => Cleanup::None,
        }
    }
}

/// A position within a chunk
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalPos(u16);

impl LocalPos {
    /// Bitmask that preserves x position of self
    pub const X_MASK: Self = Self(0xf00);
    /// Bitmask that preserves y position of self
    pub const Y_MASK: Self = Self(0x0f0);
    /// Bitmask that preserves z position of self
    pub const Z_MASK: Self = Self(0x00f);
    /// Bitmask that preserves x and y position of self
    pub const XY_MASK: Self = Self(0xff0);
    /// Bitmask that preserves x and z position of self
    pub const XZ_MASK: Self = Self(0xf0f);
    /// Bitmask that preserves y and z position of self
    pub const YZ_MASK: Self = Self(0x0ff);

    /// Constructs a new local pos
    ///
    /// # Panics
    ///
    /// Panics if any of `x`, `y`, or `z` is above 15
    pub fn new([x, y, z]: [u8; 3]) -> Option<Self> {
        (x < CHUNK_WIDTH && y < CHUNK_WIDTH && z < CHUNK_WIDTH)
            .then(|| Self::new_unchecked([x, y, z]))
    }

    /// Constructs a new local pos.  Incorrect usage of this function does not
    /// (currently) result in Undefined Behavior, but other code relies on proper
    /// layout of `LocalPos` and is likely to panic
    pub fn new_unchecked([x, y, z]: [u8; 3]) -> Self {
        let (x, y, z) = (x as u16, y as u16, z as u16);
        let (x, y, z) = (x << 8, y << 4, z);
        Self(x | y | z)
    }

    /// Tries to construct self from bit representation
    /// Layout (bits): 0000xxxxyyyyzzzz
    pub fn try_from_bits(bits: u16) -> Option<Self> {
        (bits & !0x0fff == 0).then_some(Self(bits))
    }

    /// Bit representation of self
    /// Layout (bits): 0000xxxxyyyyzzzz
    pub fn bits(self) -> u16 {
        self.0
    }

    pub fn x(self) -> u8 {
        ((self.0 & 0x0f00) >> 8) as u8
    }

    pub fn y(self) -> u8 {
        ((self.0 & 0x00f0) >> 4) as u8
    }

    pub fn z(self) -> u8 {
        (self.0 & 0x000f) as u8
    }

    pub fn xyz(self) -> [u8; 3] {
        [self.x(), self.y(), self.z()]
    }

    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x() as f32, self.y() as f32, self.z() as f32)
    }

    pub fn inc_x(self) -> Self {
        assert!(self.x() < 15);
        Self(self.0 + 0x0100)
    }

    pub fn inc_y(self) -> Self {
        assert!(self.y() < 15);
        Self(self.0 + 0x0010)
    }

    pub fn inc_z(self) -> Self {
        assert!(self.z() < 15);
        Self(self.0 + 0x0001)
    }

    pub const fn inner_positions() -> impl Iterator<Item = LocalPos> {
        #[derive(Debug)]
        pub struct Iter(u16);

        impl Iterator for Iter {
            type Item = LocalPos;

            fn next(&mut self) -> Option<Self::Item> {
                self.0 = self.0.wrapping_add(1);
                let mut skip_edge = |edge, add| {
                    if self.0 & edge == edge {
                        self.0 += add;
                    }
                };
                skip_edge(0x00f, 0x001);
                skip_edge(0x0f0, 0x010);
                skip_edge(0xf00, 0x100);
                LocalPos::try_from_bits(self.0)
            }
        }

        Iter(u16::MAX)
    }
}

impl Debug for LocalPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", self, self.bits())
    }
}

impl Display for LocalPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{:2}, {:2}, {:2}]", self.x(), self.y(), self.z())
    }
}

impl BitAnd for LocalPos {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::terrain::tile::Tile;

    use super::{Chunk, Cleanup, LocalPos, CHUNK_WIDTH};

    prop_compose! {
        fn arb_local_pos()(xyz in [..CHUNK_WIDTH; 3]) -> LocalPos {
            LocalPos::new(xyz).unwrap()
        }
    }

    proptest! {
        #[test]
        fn add_remove_from_chunk(pos in arb_local_pos()) {
            let mut chunk = Chunk::default();

            let cleanup = chunk.set(pos, Tile::Brick);
            assert_eq!(cleanup, Cleanup::None);

            let cleanup = chunk.remove(pos);
            assert_eq!(cleanup, Cleanup::RemoveChunk);
        }

        #[test]
        fn add_remove_many_from_chunk(poses in proptest::collection::vec(arb_local_pos(), 1..64)) {
            let mut chunk = Chunk::default();

            for &pos in &poses {
                let cleanup = chunk.set(pos, Tile::Brick);
                assert_eq!(cleanup, Cleanup::None);
            }

            for &pos in &poses[..(poses.len() - 1)] {
                let _ = chunk.remove(pos);
            }
            let cleanup = chunk.remove(*poses.last().unwrap());
            assert_eq!(cleanup, Cleanup::RemoveChunk);
        }

        #[test]
        fn local_pos_new_doesnt_panic(xyz in [..CHUNK_WIDTH; 3]) {
            LocalPos::new(xyz).unwrap();
        }

        #[test]
        #[should_panic]
        fn local_pos_new_panics(xyz in [CHUNK_WIDTH.., CHUNK_WIDTH.., CHUNK_WIDTH..]) {
            LocalPos::new(xyz).unwrap();
        }

        #[test]
        fn local_pos_xyz(xyz in [..CHUNK_WIDTH; 3]) {
            assert_eq!(xyz, LocalPos::new(xyz).unwrap().xyz());
        }
    }

    #[test]
    fn inner_pos_is_inner() {
        for inner_pos in LocalPos::inner_positions() {
            assert!(inner_pos.x() < 16);
            assert!(inner_pos.y() < 16);
            assert!(inner_pos.z() < 16);
        }
    }

    #[test]
    fn inner_pos_correct_count() {
        assert_eq!(LocalPos::inner_positions().count(), 15_usize.pow(3))
    }
}
