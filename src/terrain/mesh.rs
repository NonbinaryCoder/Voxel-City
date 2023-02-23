use std::ops::{Index, IndexMut};

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, PrimitiveTopology},
    utils::HashMap,
};
use bitflags::bitflags;

use self::mesh_builder::{MeshBuilder, SubtileFace};

use super::{
    chunk::{Chunk, ChunkPos, LocalPos, TileSlot, CHUNK_WIDTH},
    tile::Tile,
    Terrain,
};

pub mod mesh_builder;

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_material_system)
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
    mut mesh_ids: Local<HashMap<ChunkPos, (Entity, Handle<Mesh>)>>,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<TerrainMaterialHandles>,
) {
    let terrain = &mut *terrain;
    for chunk_pos in terrain.changed.drain() {
        if let Some(chunk) = terrain.chunks.get(&chunk_pos) {
            let mesh_handle = &mesh_ids
                .entry(chunk_pos)
                .or_insert_with(|| {
                    init_chunk_mesh(&mut commands, &mut meshes, &materials, chunk_pos)
                })
                .1;
            let mut mesh = MeshBuilder::edit(meshes.get_mut(mesh_handle).unwrap());
            add_inner_tiles(chunk, &mut mesh);
        } else if let Some((entity, _)) = mesh_ids.remove(&chunk_pos) {
            commands.entity(entity).despawn();
        }
    }
}

fn add_inner_tiles(chunk: &Chunk, mesh: &mut MeshBuilder) {
    for pos in LocalPos::INNER_POSITIONS {
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
        .spawn(MaterialMeshBundle {
            mesh: mesh.clone(),
            material: materials.opaque.clone(),
            transform: Transform::from_translation(pos.as_vec3() * CHUNK_WIDTH as f32),
            ..default()
        })
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

    /// Distance from corner to center of this tile
    pub const fn direction(self) -> Vec3 {
        match self.bits {
            0b000 => Vec3::new(-1.0, -1.0, -1.0),
            0b001 => Vec3::new(-1.0, -1.0, 1.0),
            0b010 => Vec3::new(-1.0, 1.0, -1.0),
            0b011 => Vec3::new(-1.0, 1.0, 1.0),
            0b100 => Vec3::new(1.0, -1.0, -1.0),
            0b101 => Vec3::new(1.0, -1.0, 1.0),
            0b110 => Vec3::new(1.0, 1.0, -1.0),
            0b111 => Vec3::new(1.0, 1.0, 1.0),
            _ => unreachable!(),
        }
    }

    pub fn tile_at_face(self, face: SubtileFace) -> Self {
        self ^ face.subtile_axis()
    }
}
