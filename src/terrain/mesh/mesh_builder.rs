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

    pub fn add<T: MeshObject>(&mut self, object: T, subtile: Subtile, pos: T::Pos) {
        object.add_to_mesh(self, subtile, pos);
    }

    fn add_tri(&mut self, position: [Vec3; 3], normal: Vec3, uv: Vec2) {
        self.position
            .extend(position.into_iter().map(|p| (p + self.offset).to_array()));
        self.normal.extend(iter::repeat(normal.to_array()).take(3));
        self.uv.extend(iter::repeat(uv.to_array()).take(3));
    }

    fn add_quad(&mut self, position: [Vec3; 4], normal: Vec3, uv: Vec2) {
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

    fn add_to_mesh(self, mesh: &mut MeshBuilder, subtile: Subtile, pos: Self::Pos) {
        let Directions { out, north, east } = pos.directions(subtile);
        let side = |a, b| out * 0.5 + a * 0.5 + b * 0.5;
        mesh.add_quad(
            [
                side(north, east),
                side(-north, east),
                side(-north, -east),
                side(north, -east),
            ],
            out,
            Vec2::splat(0.5),
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

    pub const fn directions(self, subtile: Subtile) -> Directions {
        const fn axis(do_neg: bool, pos: Vec3, neg: Vec3) -> Vec3 {
            if do_neg {
                pos
            } else {
                neg
            }
        }

        let x = axis(subtile.contains(Subtile::X), Vec3::X, Vec3::NEG_X);
        let y = axis(subtile.contains(Subtile::Y), Vec3::Y, Vec3::NEG_Y);
        let z = axis(subtile.contains(Subtile::Z), Vec3::Z, Vec3::NEG_Z);

        match self {
            SubtileFace::X => Directions {
                out: x,
                north: y,
                east: z,
            },
            SubtileFace::Y => Directions {
                out: y,
                north: x,
                east: z,
            },
            SubtileFace::Z => Directions {
                out: z,
                north: y,
                east: x,
            },
        }
    }
}

pub struct Directions {
    out: Vec3,
    north: Vec3,
    east: Vec3,
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

    fn add_to_mesh(self, mesh: &mut MeshBuilder, subtile: Subtile, pos: Self::Pos);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubtileEdge {
    XY,
    XZ,
    ZY,
}
