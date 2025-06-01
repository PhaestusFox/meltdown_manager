use bevy::{platform::collections::HashMap, prelude::*};
use cellular_automata::AutomataChunk;
use phoxels::{
    core::VoxelMaterial,
    prelude::{ChunkData, PhoxelGenerator, PhoxelsPlugin},
};
use voxel_chunk::{ChunkId, ChunkManager, VoxelChunk};

mod cellular_automata;

mod voxel_chunk;

pub const CHUNK_SIZE: usize = 30;
pub const CHUNK_ARIA: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOL: usize = CHUNK_ARIA * CHUNK_SIZE;

pub fn plugin(app: &mut App) {
    app.insert_resource(Time::<Fixed>::from_hz(10.))
        .init_resource::<ChunkManager>()
        .add_systems(Startup, spawn_test)
        .add_plugins(PhoxelsPlugin::<ChunkId>::default())
        .insert_resource(PhoxelGenerator::new(|id: ChunkId| {
            println!("gen data for {}", id);
            if id.y == 0 {
                ChunkData::solid(Blocks::Copper)
            } else {
                ChunkData::empty()
            }
        }));
}

// set to 16 for final test
const BX: i32 = 1;
// set to 16 for final test
const BZ: i32 = 1;
// set to 16 for final test
const BY: i32 = 1;

fn spawn_test(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    generator: Res<PhoxelGenerator<ChunkId>>,
) {
    let matterial = VoxelMaterial {
        atlas_shape: UVec2::splat(16),
        base_color_texture: Some(asset_server.load("solid_color.png")),
        ..Default::default()
    };
    let matterial_handle = asset_server.add(matterial);
    let mut chunk_count = 0;
    let mut total_voxels = 0;
    for x in -BX..=BX {
        for z in -BZ..=BZ {
            for y in -BY..=BY + 1 {
                chunk_count += 1;
                total_voxels += CHUNK_VOL;
                let mut entity = commands.spawn((
                    ChunkId::new(x, y, z),
                    cellular_automata::AutomataChunk::empty(),
                ));
                if x == 0 && z == 0 {
                    entity.insert((generator.clone(), MeshMaterial3d(matterial_handle.clone())));
                }
            }
        }
    }
    println!(
        "Generated: {} chunks\nEquivelent Voxels: {}",
        chunk_count, total_voxels
    );
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, strum_macros::EnumIter, strum_macros::FromRepr)]
#[repr(u8)]
enum Blocks {
    Air = 0,
    Copper,
    Iron,
    Steel,
    Uranium,
}

impl chunk_serde::Serialize for Blocks {
    fn into_vec(&self, vec: &mut Vec<u8>) -> usize {
        vec.push(*self as u8);
        1
    }
    fn from_slice(slice: &[u8]) -> (Self, usize) {
        #[cfg(debug_assertions)]
        return (Blocks::from_repr(slice[0]).unwrap(), 1);
        #[cfg(not(debug_assertions))]
        return (Blocks::from_repr(slice[0]).unwrap_or(Blocks::Air), 1);
    }
}

impl phoxels::prelude::Block for Blocks {
    fn id(&self) -> u8 {
        *self as u8 - 1
    }
    fn is_solid(&self) -> bool {
        true
    }
    fn is_transparent(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests;
