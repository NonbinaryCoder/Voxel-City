use std::ops::{Index, IndexMut};

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        primitives::Aabb,
        render_resource::{AsBindGroup, PrimitiveTopology},
    },
};
use bitflags::bitflags;

use self::mesh_builder::{MeshBuilder, SubtileFace};

use super::{
    chunk::{Chunk, ChunkPos, LocalPos, TileSlot, CHUNK_WIDTH},
    tile::Tile,
    Terrain,
};

mod inspect;
pub mod mesh_builder;

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(inspect::InspectPlugin)
            .add_startup_system(init_material_system)
            .add_plugin(MaterialPlugin::<OpaqueTerrainMaterial>::default())
            .add_system(generate_meshes_system);
    }
}

fn init_material_system(
    mut commands: Commands,
    mut materials: ResMut<Assets<OpaqueTerrainMaterial>>,
) {
    commands.insert_resource(TerrainMaterialHandles {
        opaque: materials.add(OpaqueTerrainMaterial {}),
    });
}

#[derive(Debug, AsBindGroup, TypeUuid, Clone)]
#[uuid = "d8ec3dfe-1da4-418b-93dc-b99a0fe0ee1c"]
struct OpaqueTerrainMaterial {}

impl Material for OpaqueTerrainMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "terrain/opaque_shader.wgsl".into()
    }
}

#[derive(Debug, Clone, Resource)]
struct TerrainMaterialHandles {
    opaque: Handle<OpaqueTerrainMaterial>,
}

fn generate_meshes_system(
    mut commands: Commands,
    mut terrain: ResMut<Terrain>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TerrainMaterialHandles>,
    empty_chunk: Local<Chunk>,
) {
    let terrain = &mut *terrain;
    for chunk_pos in terrain.changed.drain() {
        if let Some(chunk) = terrain.chunks.get(&chunk_pos) {
            let (entity, mesh_handle) = &terrain.mesh_ids.entry(chunk_pos).or_insert_with(|| {
                init_chunk_mesh(&mut commands, &mut meshes, &materials, chunk_pos)
            });
            let mesh = &mut meshes.get_mut(mesh_handle).unwrap();
            let mut mesh_builder = MeshBuilder::edit(mesh);
            add_inner_tiles(chunk, &mut mesh_builder);
            add_x_face_tiles(
                chunk,
                terrain
                    .chunks
                    .get(&(chunk_pos + ChunkPos::X))
                    .unwrap_or(&empty_chunk),
                &mut mesh_builder,
            );
            add_y_face_tiles(
                chunk,
                terrain
                    .chunks
                    .get(&(chunk_pos + ChunkPos::Y))
                    .unwrap_or(&empty_chunk),
                &mut mesh_builder,
            );
            add_z_face_tiles(
                chunk,
                terrain
                    .chunks
                    .get(&(chunk_pos + ChunkPos::Z))
                    .unwrap_or(&empty_chunk),
                &mut mesh_builder,
            );
            commands
                .entity(*entity)
                .insert(mesh.compute_aabb().unwrap_or(Default::default()));
        } else if [
            [0, 0, 1],
            [0, 1, 0],
            [0, 1, 1],
            [1, 0, 0],
            [1, 0, 1],
            [1, 1, 0],
            [1, 1, 1],
        ]
        .into_iter()
        .any(|pos| terrain.chunks.contains_key(&pos.into()))
        {
            let (entity, mesh_handle) = &terrain.mesh_ids.entry(chunk_pos).or_insert_with(|| {
                init_chunk_mesh(&mut commands, &mut meshes, &materials, chunk_pos)
            });
            let mesh = &mut meshes.get_mut(mesh_handle).unwrap();
            let mut mesh_builder = MeshBuilder::edit(mesh);
            if let Some(next_chunk) = terrain.chunks.get(&(chunk_pos + ChunkPos::X)) {
                add_x_face_tiles(&empty_chunk, next_chunk, &mut mesh_builder);
            }
            if let Some(next_chunk) = terrain.chunks.get(&(chunk_pos + ChunkPos::Y)) {
                add_y_face_tiles(&empty_chunk, next_chunk, &mut mesh_builder);
            }
            if let Some(next_chunk) = terrain.chunks.get(&(chunk_pos + ChunkPos::Z)) {
                add_z_face_tiles(&empty_chunk, next_chunk, &mut mesh_builder);
            }
            commands
                .entity(*entity)
                .insert(mesh.compute_aabb().unwrap_or(Default::default()));
        } else if let Some((entity, _)) = terrain.mesh_ids.remove(&chunk_pos) {
            commands.entity(entity).despawn();
        }
    }
}

fn add_inner_tiles(chunk: &Chunk, mesh: &mut MeshBuilder) {
    for pos in LocalPos::inner_positions() {
        let tiles = CornerTiles([
            chunk[pos],
            chunk[pos.inc_z()],
            chunk[pos.inc_y()],
            chunk[pos.inc_z().inc_y()],
            chunk[pos.inc_x()],
            chunk[pos.inc_x().inc_z()],
            chunk[pos.inc_x().inc_y()],
            chunk[pos.inc_x().inc_z().inc_y()],
        ]);
        generate_corner_mesh(tiles, pos, mesh);
    }
}

fn add_x_face_tiles(chunk: &Chunk, next_chunk: &Chunk, mesh: &mut MeshBuilder) {
    add_face_tiles(
        chunk,
        next_chunk,
        mesh,
        {
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
                    LocalPos::try_from_bits(self.0)
                }
            }

            Iter(0x0f00 - 1)
        },
        |chunk, next_chunk, pos| {
            let next_pos = LocalPos::try_from_bits(pos.bits() & 0x00ff).unwrap();
            CornerTiles([
                chunk[pos],
                chunk[pos.inc_z()],
                chunk[pos.inc_y()],
                chunk[pos.inc_z().inc_y()],
                next_chunk[next_pos],
                next_chunk[next_pos.inc_z()],
                next_chunk[next_pos.inc_y()],
                next_chunk[next_pos.inc_z().inc_y()],
            ])
        },
    )
}

fn add_y_face_tiles(chunk: &Chunk, next_chunk: &Chunk, mesh: &mut MeshBuilder) {
    add_face_tiles(
        chunk,
        next_chunk,
        mesh,
        {
            #[derive(Debug)]
            pub struct Iter(u16);

            impl Iterator for Iter {
                type Item = LocalPos;

                fn next(&mut self) -> Option<Self::Item> {
                    self.0 = self.0.wrapping_add(1);
                    if self.0 & 0x00f == 0x00f {
                        self.0 += 0x001;
                    }
                    if self.0 & 0x0f0 != 0 {
                        self.0 &= 0xf0f;
                        self.0 += 0x100;
                    }
                    if self.0 & 0xf00 == 0xf00 {
                        self.0 += 0x100;
                    }
                    LocalPos::try_from_bits(self.0 | 0x0f0)
                }
            }

            Iter(u16::MAX)
        },
        |chunk, next_chunk, pos| {
            let next_pos = LocalPos::try_from_bits(pos.bits() & 0x0f0f).unwrap();
            CornerTiles([
                chunk[pos],
                chunk[pos.inc_z()],
                next_chunk[next_pos],
                next_chunk[next_pos.inc_z()],
                chunk[pos.inc_x()],
                chunk[pos.inc_x().inc_z()],
                next_chunk[next_pos.inc_x()],
                next_chunk[next_pos.inc_x().inc_z()],
            ])
        },
    )
}

fn add_z_face_tiles(chunk: &Chunk, next_chunk: &Chunk, mesh: &mut MeshBuilder) {
    add_face_tiles(
        chunk,
        next_chunk,
        mesh,
        {
            #[derive(Debug)]
            pub struct Iter(u16);

            impl Iterator for Iter {
                type Item = LocalPos;

                fn next(&mut self) -> Option<Self::Item> {
                    self.0 = self.0.wrapping_add(0x10);
                    let mut skip_edge = |edge, add| {
                        if self.0 & edge == edge {
                            self.0 += add;
                        }
                    };
                    skip_edge(0x0f0, 0x010);
                    skip_edge(0xf00, 0x100);
                    LocalPos::try_from_bits(self.0 | 0x00f)
                }
            }

            Iter(0xffff)
        },
        |chunk, next_chunk, pos| {
            let next_pos = LocalPos::try_from_bits(pos.bits() & 0x0ff0).unwrap();
            CornerTiles([
                chunk[pos],
                next_chunk[next_pos],
                chunk[pos.inc_y()],
                next_chunk[next_pos.inc_y()],
                chunk[pos.inc_x()],
                next_chunk[next_pos.inc_x()],
                chunk[pos.inc_x().inc_y()],
                next_chunk[next_pos.inc_x().inc_y()],
            ])
        },
    )
}

fn add_face_tiles(
    chunk: &Chunk,
    next_chunk: &Chunk,
    mesh: &mut MeshBuilder,
    iter: impl Iterator<Item = LocalPos>,
    extract_tiles: impl Fn(&Chunk, &Chunk, LocalPos) -> CornerTiles,
) {
    for pos in iter {
        generate_corner_mesh(extract_tiles(chunk, next_chunk, pos), pos, mesh);
    }
}

fn init_chunk_mesh(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &TerrainMaterialHandles,
    pos: ChunkPos,
) -> (Entity, Handle<Mesh>) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
    let mesh = meshes.add(mesh);

    let entity = commands
        .spawn((
            MaterialMeshBundle {
                mesh: mesh.clone(),
                material: materials.opaque.clone(),
                transform: Transform::from_translation(pos.as_vec3() * CHUNK_WIDTH as f32),
                ..default()
            },
            Aabb::default(),
        ))
        .id();
    (entity, mesh)
}

fn generate_corner_mesh(tiles: CornerTiles, pos: LocalPos, mesh: &mut MeshBuilder) {
    mesh.set_offset(pos.to_vec3());
    for subtile in Subtile::subtiles() {
        Tile::generate_mesh(&tiles, subtile, mesh);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CornerTiles([TileSlot; 8]);

impl Index<Subtile> for CornerTiles {
    type Output = TileSlot;

    fn index(&self, index: Subtile) -> &Self::Output {
        &self.0[index.bits() as usize]
    }
}

impl IndexMut<Subtile> for CornerTiles {
    fn index_mut(&mut self, index: Subtile) -> &mut Self::Output {
        &mut self.0[index.bits() as usize]
    }
}

bitflags! {
    pub struct Subtile: u8 {
        const X = 0b100;
        const Y = 0b010;
        const Z = 0b001;
    }
}

impl Subtile {
    pub fn subtiles() -> impl Iterator<Item = Subtile> {
        struct Iter(u8);

        impl Iterator for Iter {
            type Item = Subtile;

            fn next(&mut self) -> Option<Self::Item> {
                self.0 = self.0.wrapping_add(1);
                Subtile::from_bits(self.0)
            }
        }

        Iter(u8::MAX)
    }

    pub fn tile_at_face(self, face: SubtileFace) -> Self {
        self ^ face.subtile_axis()
    }
}
