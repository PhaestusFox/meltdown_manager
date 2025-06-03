use bevy::{
    app::{App, Startup},
    asset::AssetApp,
    math::UVec3,
};
use noise::{MultiFractal, NoiseFn};
use phoxels::{PhoxelsPlugin, core::PhoxelGenerator};
use strum::EnumCount;

use crate::voxels::{
    Blocks, cellular_automata, spawn_test,
    voxel_chunk::{
        chunk::{ChunkId, ChunkManager},
        prefab::ChunkPrefabLoader,
    },
};

pub type ChunkData = phoxels::prelude::ChunkData<Blocks>;

pub const CHUNK_SIZE: usize = 30;
pub const CHUNK_ARIA: usize = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOL: usize = CHUNK_ARIA * CHUNK_SIZE;

pub fn map_plugin(app: &mut App) {
    let noise = MapNoise::new();
    app.init_asset_loader::<ChunkPrefabLoader>()
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
