pub mod block;
pub mod cellular_automata;
pub mod map;
mod voxel_chunk;

use bevy::prelude::*;
use block::BlockType;
use phoxels::{core::VoxelMaterial, prelude::PhoxelGenerator};
pub use voxel_chunk::*;

use crate::voxels::{
    cellular_automata::{CellFlags, Cells, FixedNum, NextStep},
    map::ChunkData,
};

pub use map::{CHUNK_SIZE, CHUNK_VOL};

#[cfg(target_arch = "wasm32")]
const MAP_SIZE: UVec3 = UVec3::new(5, 1, 5);

#[cfg(not(target_arch = "wasm32"))]
const MAP_SIZE: UVec3 = UVec3::new(7, 2, 7);

#[derive(Resource)]
pub struct VoxleMaterialHandle(Handle<VoxelMaterial>);

impl FromWorld for VoxleMaterialHandle {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let material = VoxelMaterial {
            atlas_shape: UVec4::new(16, 16, 0, 0),
            base_color_texture: Some(asset_server.load("solid_color.png")),
            ..Default::default()
        };
        let handle = asset_server.add(material);
        VoxleMaterialHandle(handle)
    }
}

impl VoxleMaterialHandle {
    pub fn get(&self) -> Handle<VoxelMaterial> {
        self.0.clone()
    }
}

fn spawn_test(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    generator: Res<PhoxelGenerator<BlockType, ChunkId>>,
    matterial_handle: Res<VoxleMaterialHandle>,
) {
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
            let y = MAP_SIZE.y as i32;
            for x in 0..MAP_SIZE.x {
                for z in 0..MAP_SIZE.z {
                    for y in -y..1 {
                        chunk_count += 1;
                        total_voxels += CHUNK_VOL;
                        root.spawn((
                            ChunkId::new(x as i32, y, z as i32),
                            // Chunk::<CellData>::empty(),
                            // Mesh3d(Default::default()),
                            generator.clone(),
                            MeshMaterial3d(matterial_handle.get()),
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

fn remove_evaluation(mut chunks: Query<&mut Cells>) {
    let mut removed = 0;
    for mut chunk in &mut chunks {
        for (x, y, z) in crate::utils::BlockIter::new() {
            let mut cell = chunk.get_by_index_mut(Cells::index(x, y, z));
            if cell.get_block_type() == BlockType::Void || cell.get_block_type() == BlockType::Air {
                continue;
            }
            if cell.is_gas() {
                cell.set_block_type(BlockType::Air);
                cell.energy = FixedNum::ZERO;
                removed += 1;
            };
        }
    }
    if removed > 0 {
        println!("Removed {} blocks as gas", removed);
    }
}
