pub mod blocks;
pub mod cellular_automata;
pub mod map;
mod voxel_chunk;

use bevy::prelude::*;
use blocks::Blocks;
use cellular_automata::CellData;
use map::CHUNK_VOL;
use phoxels::{core::VoxelMaterial, prelude::PhoxelGenerator};
pub use voxel_chunk::chunk::{Chunk, ChunkId, NeighbourDirection, Neighbours};

// set to 16 for final test
const BX: i32 = 3;
// set to 16 for final test
const BZ: i32 = 3;
// set to 16 for final test
const BY: i32 = 1;

fn spawn_test(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    generator: Res<PhoxelGenerator<Blocks, ChunkId>>,
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
                commands.spawn((
                    ChunkId::new(x, y, z),
                    Chunk::<CellData>::empty(),
                    generator.clone(),
                    MeshMaterial3d(matterial_handle.clone()),
                ));
            }
        }
    }
    println!(
        "Generated: {} chunks\nEquivelent Voxels: {}",
        chunk_count, total_voxels
    );
}

#[cfg(test)]
mod tests;
