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

use crate::voxels::{
    cellular_automata::{CellFlags, Cells, FixedNum, NextStep},
    map::ChunkData,
};

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
        atlas_shape: UVec4::new(16, 16, 0, 0),
        base_color_texture: Some(asset_server.load("solid_color.png")),
        ..Default::default()
    };
    let matterial_handle = asset_server.add(matterial);
    let mut chunk_count = 0;
    let mut total_voxels = 0;

    // return;
    commands
        .spawn((
            Name::new("Chunks"),
            Transform::IDENTITY,
            Visibility::Visible,
        ))
        .with_children(|root| {
            for x in -BX..=BX {
                for z in -BZ..=BZ {
                    for y in -BY..=BY {
                        chunk_count += 1;
                        total_voxels += CHUNK_VOL;
                        root.spawn((
                            ChunkId::new(x, y, z),
                            Chunk::<CellData>::empty(),
                            // Mesh3d(Default::default()),
                            generator.clone(),
                            MeshMaterial3d(matterial_handle.clone()),
                        ));
                    }
                }
            }
        });
    println!(
        "Generated: {} chunks\nEquivelent Voxels: {}",
        chunk_count, total_voxels
    );
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests;

fn remove_evaluation(
    mut mesher: ResMut<phoxels::ChunkMesher>,
    mut chunks: Query<(Entity, &mut NextStep, &mut ChunkData)>,
) {
    let mut removed = 0;
    for (entity, mut next, mut data) in &mut chunks {
        let mut update = false;
        for (x, y, z) in crate::utils::BlockIter::<30, 30, 30>::new() {
            #[cfg(debug_assertions)]
            let chunk = next
                .borrow_mut()
                .expect("NextStep has not run yet\n Should only run in Step::Done");
            #[cfg(not(debug_assertions))]
            let Some(chunk) = next.borrow_mut() else {
                error!("Tried to remove gas from a chunk that has not run yet");
                continue;
            };
            let cell = chunk.get_by_index_mut(Cells::index(x, y, z));
            if cell.block == Blocks::Void || cell.block == Blocks::Air {
                continue;
            }
            if cell.flags.contains(CellFlags::IS_GAS) {
                data.set_block(x as u32, y as u32, z as u32, Blocks::Air);
                cell.block = Blocks::Air;
                cell.energy = FixedNum::ZERO;
                update = true;
                removed += 1;
            };
        }
        if update {
            mesher.add_to_queue(entity);
        }
    }
    if removed > 0 {
        println!("Removed {} blocks as gas", removed);
    }
}
