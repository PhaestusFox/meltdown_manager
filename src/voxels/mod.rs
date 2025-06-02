use bevy::prelude::*;
use cellular_automata::CellData;
use chunk_serde::BinSerializer;
use noise::{MultiFractal, NoiseFn};
use phoxels::{
    core::{BlockId, VoxelMaterial},
    prelude::{PhoxelGenerator, PhoxelsPlugin},
};
use strum::EnumCount;
use voxel_chunk::ChunkManager;
pub use voxel_chunk::{Chunk, ChunkId};

use crate::voxels::cellular_automata::{BlockProperties, FixedNum};
pub mod cellular_automata;
pub type ChunkData = phoxels::prelude::ChunkData<Blocks>;
mod voxel_chunk;

pub const CHUNK_SIZE: usize = 30;
pub const CHUNK_ARIA: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOL: usize = CHUNK_ARIA * CHUNK_SIZE;

pub fn plugin(app: &mut App) {
    let noise = MapNoise::new();
    app.init_asset_loader::<voxel_chunk::ChunkPrefabLoader>()
        .insert_resource(Time::<Fixed>::from_hz(10.))
        .init_resource::<ChunkManager>()
        .add_systems(Startup, spawn_test)
        .add_plugins(PhoxelsPlugin::<Blocks, ChunkId>::default())
        .insert_resource(PhoxelGenerator::new(move |id: ChunkId| {
            let noise = noise.clone();
            let mut chunk = ChunkData::new(UVec3::splat(CHUNK_SIZE as u32));
            for x in 0..CHUNK_SIZE as i32 {
                for z in 0..CHUNK_SIZE as i32 {
                    let gx = id.x * CHUNK_SIZE as i32 + x;
                    let gz = id.z * CHUNK_SIZE as i32 + z;
                    let h = noise.get_ground(gx, gz);
                    let start_y = id.y * CHUNK_SIZE as i32;
                    if start_y > h {
                        continue;
                    }
                    let num_blocks = Blocks::COUNT as f64 - 1.;
                    for y in 0..(h - start_y).min(30) {
                        let r = ((noise.sample(gx, y + start_y, gz) * 10.) % num_blocks) as u8;
                        let block = Blocks::from_repr(r + 1).unwrap_or_default();
                        chunk.set_block(x as u32, y as u32, z as u32, block);
                    }
                }
            }
            chunk
        }));
    app.add_plugins(cellular_automata::plugin);
}

#[derive(Clone)]
struct MapNoise {
    noise: std::sync::Arc<noise::Fbm<noise::Simplex>>,
    ground: i32,
}

impl MapNoise {
    fn new() -> MapNoise {
        let mut noise = noise::Fbm::new(0);
        noise.frequency = 0.01;
        noise = noise.set_persistence(0.2);
        MapNoise {
            noise: std::sync::Arc::new(noise),
            ground: 32,
        }
    }

    fn sample(&self, x: i32, y: i32, z: i32) -> f64 {
        let mut h = self.noise.get([x as f64, y as f64, z as f64]);
        h += 1.;
        h /= 2.;
        h
    }

    fn get_ground(&self, x: i32, z: i32) -> i32 {
        let h = self.sample(x, 0, z) * self.ground as f64;
        h as i32
    }
}

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

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    strum_macros::EnumIter,
    strum_macros::FromRepr,
    Default,
    strum_macros::EnumCount,
)]
#[repr(u8)]
pub enum Blocks {
    #[default]
    Void = 0,
    Copper,
    Iron,
    Steel,
    Uranium,
}

impl Blocks {
    const fn block_properties(&self) -> BlockProperties {
        match self {
            Blocks::Void => BlockProperties::VOID,
            Blocks::Copper => BlockProperties {
                ..BlockProperties::DEFAULT
            },
            Blocks::Iron => BlockProperties::DEFAULT,
            Blocks::Steel => BlockProperties::DEFAULT,
            Blocks::Uranium => BlockProperties::URANIUM,
        }
    }
}

impl From<BlockId> for Blocks {
    fn from(id: BlockId) -> Self {
        Blocks::from_repr(id.0).unwrap_or(Blocks::Void)
    }
}

impl chunk_serde::Serialize for Blocks {
    fn insert(&self, vec: &mut BinSerializer) -> Result<usize> {
        vec.push(*self as u8);
        Ok(1)
    }
    fn extract(slice: &[u8]) -> Result<(Self, usize)> {
        #[cfg(debug_assertions)]
        return Ok((Blocks::from_repr(slice[0]).unwrap(), 1));
        #[cfg(not(debug_assertions))]
        return (Blocks::from_repr(slice[0]).unwrap_or(Blocks::Void), 1);
    }
}

impl phoxels::prelude::Block for Blocks {
    fn id(&self) -> u8 {
        *self as u8
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
