use std::iter;

use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use bitflags::bitflags;

use super::Subtile;

#[derive(Debug)]
pub struct MeshBuilder<'a> {
    position: &'a mut Vec<[f32; 3]>,
    normal: &'a mut Vec<[f32; 3]>,
    uv: &'a mut Vec<[f32; 2]>,
    offset: Vec3,
}

impl<'a> MeshBuilder<'a> {
    pub(super) fn edit(mesh: &'a mut Mesh) -> Self {
        let mut position = None;
        let mut normal = None;
        let mut uv = None;

        for (id, values) in mesh.attributes_mut() {
            if id == Mesh::ATTRIBUTE_POSITION.id {
                let VertexAttributeValues::Float32x3(p) = values else { panic!(
                    "position should be `Float32x3` but is `{}``",
                    values.enum_variant_name()
                ) };
                position = Some(p);
            } else if id == Mesh::ATTRIBUTE_NORMAL.id {
                let VertexAttributeValues::Float32x3(n) = values else { panic!(
                    "normal should be `Float32x3` but is `{}``",
                    values.enum_variant_name()
                ) };
                normal = Some(n);
            } else if id == Mesh::ATTRIBUTE_UV_0.id {
                let VertexAttributeValues::Float32x2(p) = values else { panic!(
                    "uv should be `Float32x2` but is `{}``",
                    values.enum_variant_name()
                ) };
                uv = Some(p);
            } else {
                panic!("Unexpected mesh attribute: {id:?}")
            }
        }

        let position = position.expect("Terrain mesh missing position");
        let normal = normal.expect("Terrain mesh missing normal");
        let uv = uv.expect("Terrain mesh missing uv");

        position.clear();
        normal.clear();
        uv.clear();
        Self {
            position,
            normal,
            uv,
            offset: Vec3::ZERO,
        }
    }

    pub(super) fn set_offset(&mut self, offset: Vec3) {
        self.offset = offset;
    }

    pub fn add<T: MeshObject>(&mut self, object: T, uv: [f32; 2], subtile: Subtile, pos: T::Pos) {
        object.add_to_mesh(self, uv, subtile, pos);
    }

    fn add_tri(&mut self, position: [Vec3; 3], normal: Vec3, uv: [f32; 2]) {
        self.position
            .extend(position.into_iter().map(|p| (p + self.offset).to_array()));
        self.normal.extend(iter::repeat(normal.to_array()).take(3));
        self.uv.extend([uv; 3]);
    }

    fn add_quad(&mut self, position: [Vec3; 4], normal: Vec3, uv: [f32; 2]) {
        self.add_tri([position[0], position[1], position[3]], normal, uv);
        self.add_tri([position[2], position[3], position[1]], normal, uv);
    }
}

#[derive(Debug, Default)]
pub enum Face {
    #[default]
    None,
    Wall(WallFlags),
}

impl MeshObject for Face {
    type Pos = SubtileFace;

    fn add_to_mesh(self, mesh: &mut MeshBuilder, uv: [f32; 2], subtile: Subtile, pos: Self::Pos) {
        let Directions {
            normal,
            tangent,
            bitangent,
        } = pos.directions(subtile);
        let side = |a, b| a * 0.5 + b * 0.5;
        mesh.add_quad(
            [
                side(tangent, bitangent),
                side(Vec3::ZERO, bitangent),
                side(Vec3::ZERO, Vec3::ZERO),
                side(tangent, Vec3::ZERO),
            ],
            normal,
            uv,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubtileFace {
    X,
    Y,
    Z,
}

impl SubtileFace {
    pub fn faces() -> impl Iterator<Item = SubtileFace> {
        struct Iter(Option<SubtileFace>);

        impl Iterator for Iter {
            type Item = SubtileFace;

            fn next(&mut self) -> Option<Self::Item> {
                let ret = self.0;
                self.0 = match self.0 {
                    Some(SubtileFace::X) => Some(SubtileFace::Y),
                    Some(SubtileFace::Y) => Some(SubtileFace::Z),
                    Some(SubtileFace::Z) => None,
                    None => None,
                };
                ret
            }
        }

        Iter(Some(SubtileFace::X))
    }

    pub fn subtile_axis(self) -> Subtile {
        match self {
            SubtileFace::X => Subtile::X,
            SubtileFace::Y => Subtile::Y,
            SubtileFace::Z => Subtile::Z,
        }
    }

    pub fn directions(self, subtile: Subtile) -> Directions {
        macro_rules! dir {
            ($normal:ident, $tangent:ident, $bitangent:ident) => {
                Directions {
                    normal: Vec3::$normal,
                    tangent: Vec3::$tangent,
                    bitangent: Vec3::$bitangent,
                }
            };
        }
        // The compiler should figure out this prevents the `unreachable!()` calls
        // from being reached
        assert!(subtile.bits() <= 0b111);
        match self {
            SubtileFace::X => match subtile.bits() {
                0b000 => dir!(X, NEG_Y, NEG_Z),
                0b001 => dir!(X, Z, NEG_Y),
                0b010 => dir!(X, NEG_Z, Y),
                0b011 => dir!(X, Y, Z),
                0b100 => dir!(NEG_X, NEG_Z, NEG_Y),
                0b101 => dir!(NEG_X, NEG_Y, Z),
                0b110 => dir!(NEG_X, Y, NEG_Z),
                0b111 => dir!(NEG_X, Z, Y),
                _ => unreachable!(),
            },
            SubtileFace::Y => match subtile.bits() {
                0b000 => dir!(Y, NEG_Z, NEG_X),
                0b001 => dir!(Y, NEG_X, Z),
                0b010 => dir!(NEG_Y, NEG_X, NEG_Z),
                0b011 => dir!(NEG_Y, Z, NEG_X),
                0b100 => dir!(Y, X, NEG_Z),
                0b101 => dir!(Y, Z, X),
                0b110 => dir!(NEG_Y, NEG_Z, X),
                0b111 => dir!(NEG_Y, X, Z),
                _ => unreachable!(),
            },
            SubtileFace::Z => match subtile.bits() {
                0b000 => dir!(Z, NEG_X, NEG_Y),
                0b001 => dir!(NEG_Z, NEG_Y, NEG_X),
                0b010 => dir!(Z, Y, NEG_X),
                0b011 => dir!(NEG_Z, NEG_X, Y),
                0b100 => dir!(Z, NEG_Y, X),
                0b101 => dir!(NEG_Z, X, NEG_Y),
                0b110 => dir!(Z, X, Y),
                0b111 => dir!(NEG_Z, Y, X),
                _ => dir!(ZERO, ZERO, ZERO),
            },
        }
    }
}

pub struct Directions {
    normal: Vec3,
    tangent: Vec3,
    bitangent: Vec3,
}

bitflags! {
    pub struct WallFlags: u8 {
        const CONNECTED_N = 0b00001;
        const CONNECTED_E = 0b00010;
        const CONNECTED_S = 0b00100;
        const CONNECTED_W = 0b01000;
        const FACING_POSITIVE = 0b10000;

        const CONNECTED_ALL = 0b01111;
    }
}

impl Default for WallFlags {
    fn default() -> Self {
        Self::CONNECTED_ALL
    }
}

impl WallFlags {
    pub fn set_facing_positive(self, subtile: Subtile, axis: Subtile) -> Self {
        if subtile.contains(axis) {
            self | Self::FACING_POSITIVE
        } else {
            self
        }
    }
}

#[derive(Debug, Default)]
pub enum Edge {
    #[default]
    None,
    Round,
}

pub trait MeshObject {
    type Pos;

    fn add_to_mesh(self, mesh: &mut MeshBuilder, uv: [f32; 2], subtile: Subtile, pos: Self::Pos);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubtileEdge {
    XY,
    XZ,
    ZY,
}
