use derive_more::IsVariant;

use self::color::IndexedColor;

pub mod color;
mod inspect;
mod mesh;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IsVariant)]
pub enum Tile {
    Brick { color: IndexedColor },
}

impl Default for Tile {
    fn default() -> Self {
        Self::BRICK
    }
}

impl Tile {
    pub const BRICK: Self = Self::Brick {
        color: IndexedColor::DEFAULT,
    };

    /// Solid tiles completely hide faces of adjacent tiles that face them,
    /// meaning rendering those faces can be skipped
    pub fn is_solid(tile: Option<Tile>) -> bool {
        tile.is_some()
    }
}
