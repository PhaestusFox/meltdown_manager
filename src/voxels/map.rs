use bevy::prelude::*;
use noise::{MultiFractal, NoiseFn};
use phoxels::{PhoxelsPlugin, core::PhoxelGenerator};
use strum::EnumCount;

use crate::{
    GameState,
    utils::BlockIter,
    voxels::{
        BlockType, VoxleMaterialHandle,
        cellular_automata::{self, Cells},
        spawn_test,
        voxel_chunk::{ChunkId, chunk::ChunkManager, prefab::ChunkPrefabLoader},
    },
};

pub type ChunkData = phoxels::prelude::ChunkData<BlockType>;

pub const CHUNK_SIZE: i32 = 10;
pub const CHUNK_AREA: i32 = CHUNK_SIZE * CHUNK_SIZE;
pub const CHUNK_VOL: usize = (CHUNK_AREA * CHUNK_SIZE) as usize;

pub fn map_plugin(app: &mut App) {
    let noise = MapNoise::new();
    app.init_asset_loader::<ChunkPrefabLoader>()
        .init_resource::<ChunkManager>()
        .add_systems(OnEnter(GameState::Game), spawn_test)
        .add_plugins(PhoxelsPlugin::<BlockType, ChunkId>::default());

    app.insert_resource(PhoxelGenerator::new(move |id: ChunkId| {
        let noise = noise.clone();
        let mut chunk = ChunkData::new(UVec3::splat(CHUNK_SIZE as u32));
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let gx = id.x * CHUNK_SIZE + x;
                let gz = id.z * CHUNK_SIZE + z;
                let h = noise.get_ground(gx, gz);
                let start_y = id.y * CHUNK_SIZE;
                let num_blocks = BlockType::COUNT as f64 - 1.;
                // if start_y > h {
                //     for y in 0..CHUNK_SIZE {
                //         chunk.set_block(x as u32, y as u32, z as u32, Blocks::Air);
                //     }
                // } else if h - start_y < CHUNK_SIZE {
                //     // for y in (h - start_y)..CHUNK_SIZE {
                //     //     chunk.set_block(x as u32, y as u32, z as u32, Blocks::Air);
                //     // }
                //     // for y in 0..(h - start_y).min(30) {
                //     //     let r = ((noise.sample(gx, y + start_y, gz) * num_blocks * 10.)e
                //     //         % num_blocks) as u8;
                //     //     let block = Blocks::from_repr(r).unwrap_or_default();
                //     //     debug_assert!(block != Blocks::Void);
                //     //     chunk.set_block(x as u32, y as u32, z as u32, block);
                //     // }
                // } else {
                // }
                for y in 0..CHUNK_SIZE {
                    let b = if start_y + y >= h {
                        BlockType::Air
                    } else {
                        let r = ((noise.sample(gx, y + start_y, gz) * num_blocks * 3.) % num_blocks)
                            as u8;
                        BlockType::from_repr(r).unwrap_or_default()
                    };
                    debug_assert!(b != BlockType::Void);
                    chunk.set_block(x as u32, y as u32, z as u32, b);
                }
            }
        }
        chunk
    }));

    app.add_plugins(cellular_automata::plugin);
    app.init_resource::<super::VoxleMaterialHandle>();
    app.world_mut()
        .register_component_hooks::<ChunkData>()
        .on_add(|mut world, ctx| {
            let blocks = world
                .get::<ChunkData>(ctx.entity)
                .expect("Just inserted ChunkData")
                .iter()
                .cloned()
                .collect::<Vec<_>>();
            let Some(mut chunk) = world.get_mut::<Cells>(ctx.entity) else {
                let mut chunk = Cells::empty();
                for (i, block) in blocks.into_iter().enumerate() {
                    chunk.get_by_index_mut(i).set_block_type(block);
                }
                world.commands().entity(ctx.entity).insert(chunk);
                return;
            };
            for (i, block) in blocks.into_iter().enumerate() {
                chunk.get_by_index_mut(i).set_block_type(block);
            }
        });
    app.add_systems(Update, add_mesh_data_to_loaded_chunks);
    app.add_systems(
        PostUpdate,
        super::remove_evaluation.run_if(crate::voxels::cellular_automata::can_modify_next_step),
    );
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

fn add_mesh_data_to_loaded_chunks(
    chunks: Query<(Entity, &Cells), Without<ChunkData>>,
    material: Res<VoxleMaterialHandle>,
    mut meshes: ResMut<phoxels::ChunkMesher>,
    mut commands: Commands,
) {
    for (entity, chunk) in &chunks {
        let mut data = ChunkData::new(UVec3::splat(CHUNK_SIZE as u32));
        for (x, y, z) in BlockIter::new() {
            let block = chunk.get_cell(x, y, z).get_block_type();
            data.set_block(x as u32, y as u32, z as u32, block);
        }
        commands
            .entity(entity)
            .insert((data, MeshMaterial3d(material.get())));
        meshes.add_to_queue(entity);
    }
}
