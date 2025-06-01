use bevy::{platform::collections::HashMap, prelude::*};

use chunk_serde::CompressedChunkData;

use super::{Blocks, CHUNK_ARIA, CHUNK_SIZE, CHUNK_VOL, cellular_automata::AutomataChunk};

#[derive(Component, Deref, Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Reflect)]
#[component(immutable, on_insert = ChunkId::on_insert, on_remove = ChunkId::on_remove)]
#[require(Transform)]
pub struct ChunkId(IVec3);

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Chunk({},{},{})", self.x, self.y, self.z))
    }
}

impl ChunkId {
    pub fn new(x: i32, y: i32, z: i32) -> ChunkId {
        Self(IVec3::new(x, y, z))
    }

    fn on_insert(
        mut world: bevy::ecs::world::DeferredWorld,
        ctx: bevy::ecs::component::HookContext,
    ) {
        let id = *world
            .get::<ChunkId>(ctx.entity)
            .expect("This Just got inserted");
        if let Some(old) = world
            .resource_mut::<ChunkManager>()
            .insert_chunk(id, ctx.entity)
        {
            if old != ctx.entity {
                warn!(
                    "already used ChunkId({}) on {}: this is probably unitentonal despawing old entity",
                    id.0, ctx.entity
                );
                world.commands().entity(old).despawn();
            } else {
                warn!(
                    "inseted ChunkId({}) onto the same entity: this should not be done",
                    id.0
                )
            }
        }
    }

    fn on_remove(
        mut world: bevy::ecs::world::DeferredWorld,
        ctx: bevy::ecs::component::HookContext,
    ) {
        let id = *world
            .get::<ChunkId>(ctx.entity)
            .expect("This Just about to be removed");
        let mut map = world.resource_mut::<ChunkManager>();
        if let Some(old) = map.remove_chunk(&id) {
            if old != ctx.entity {
                error!(
                    "removed ChunkId from {} but {} has the same id\n*This is a Bug*\n
                Adding {} back to Manager",
                    ctx.entity, old, old
                );
                map.insert_chunk(id, old);
            }
        }
    }
}

#[derive(Default, Resource)]
pub(super) struct ChunkManager {
    map: HashMap<ChunkId, Entity>,
}

impl ChunkManager {
    fn insert_chunk(&mut self, id: ChunkId, entity: Entity) -> Option<Entity> {
        self.map.insert(id, entity)
    }
    fn remove_chunk(&mut self, id: &ChunkId) -> Option<Entity> {
        self.map.remove(id)
    }

    fn get_chunk(&self, id: &ChunkId) -> Option<Entity> {
        self.map.get(id).cloned()
    }

    fn save_chunk(
        &self,
        chunk: ChunkId,
        data: &Query<(&VoxelChunk, &AutomataChunk)>,
        path: &'static str,
    ) -> Result<(), ChunkManagerError> {
        let entity = self
            .get_chunk(&chunk)
            .ok_or(ChunkManagerError::NoEntity(chunk))?;
        let (blocks, automata) = data.get(entity)?;
        let blocks = blocks.compress();
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
enum ChunkManagerError {
    #[error("Failed to find Entity for {0}")]
    NoEntity(ChunkId),
    #[error("Failed to get data from Query: {0}")]
    NoData(#[from] bevy::ecs::query::QueryEntityError),
}

#[derive(Component, PartialEq, Eq, Debug)]
pub struct VoxelChunk {
    blocks: [Blocks; CHUNK_VOL],
}

impl VoxelChunk {
    #[inline(always)]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Blocks {
        if VoxelChunk::in_bounds(x, y, z) {
            self.blocks[VoxelChunk::index(x, y, z)]
        } else {
            #[cfg(debug_assertions)]
            panic!("Index({}, {}, {}) is out of bound", x, y, z);
            #[allow(unreachable_code)] // can when not in debug
            Blocks::Air
        }
    }

    #[inline(always)]
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, to: Blocks) {
        if VoxelChunk::in_bounds(x, y, z) {
            self.blocks[VoxelChunk::index(x, y, z)] = to;
        } else {
            #[cfg(debug_assertions)]
            panic!("Index({}, {}, {}) is out of bound", x, y, z);
        }
    }

    #[inline(always)]
    pub(super) fn index(x: i32, y: i32, z: i32) -> usize {
        x as usize + z as usize * CHUNK_SIZE + y as usize * CHUNK_ARIA
    }

    #[inline(always)]
    fn in_bounds(x: i32, y: i32, z: i32) -> bool {
        x < CHUNK_SIZE as i32
            && y < CHUNK_SIZE as i32
            && z < CHUNK_SIZE as i32
            && x >= 0
            && y >= 0
            && z >= 0
    }

    #[inline]
    fn blocks(&self) -> impl Iterator<Item = Blocks> {
        ChunkBlockIter::new(self)
    }

    pub fn compress(&self) -> CompressedChunkData<Blocks> {
        let mut solid = true;
        let mut longes_runs = [0; 10];
        let mut current_run = 1;
        let mut current_block = self.blocks[0];
        let mut total_runs = 1;
        for block in self.blocks().skip(1) {
            if current_block == block {
                current_run += 1;
                continue;
            }
            total_runs += 1;
            solid = false;
            let mut s = 0;
            let mut sr = u16::MAX;
            for (i, &run) in longes_runs.iter().enumerate() {
                if current_run > run && sr > run {
                    sr = run;
                    s = i;
                }
            }
            if sr != u16::MAX {
                longes_runs[s] = current_run;
            }
            current_run = 1;
            current_block = block;
        }
        if solid {
            return CompressedChunkData::Solid(current_block);
        } else
        // 9000 is the min number of runs required for this to be smaller then just saving every block
        if total_runs > 9001 {
            return CompressedChunkData::Raw(self.blocks.into());
        };
        // do run len encoding
        // maybe move this to top and make this encoding on first loop
        // then discard it if solid or raw is more efficent
        let mut runs = Vec::with_capacity(total_runs);
        current_block = self.blocks[0];
        current_run = 0;
        for block in self.blocks() {
            if current_block == block {
                current_run += 1;
            } else {
                runs.push((current_block, current_run));
                current_run = 1;
                current_block = block;
            }
        }
        runs.push((current_block, current_run));
        CompressedChunkData::RunLen(runs)
    }

    pub fn empty() -> Self {
        Self {
            blocks: [Blocks::Air; CHUNK_VOL],
        }
    }

    pub fn solid(fill: Blocks) -> Self {
        Self {
            blocks: [fill; CHUNK_VOL],
        }
    }

    pub fn decompress(data: &CompressedChunkData<Blocks>) -> VoxelChunk {
        match data {
            CompressedChunkData::Solid(block) => VoxelChunk::solid(*block),
            CompressedChunkData::RunLen(runs) => {
                let mut chunk = VoxelChunk::empty();
                let mut i = 0;
                for (block, run) in runs {
                    for _ in 0..*run {
                        chunk.blocks[i] = *block;
                        i += 1;
                    }
                }
                chunk
            }
            CompressedChunkData::Raw(items) => {
                let mut chunk = VoxelChunk::empty();
                chunk.blocks.copy_from_slice(&items[..CHUNK_VOL]);
                chunk
            }
            CompressedChunkData::Error(i) => {
                error!("Got Compression Error({i}); this is a bug dont decompress Errors");
                #[cfg(debug_assertions)]
                panic!(
                    "Got Compression Error({i});\n
                Can't Decompress and error;\n
                This will return VoxelChunk::empty() in release
                "
                );
                #[allow(unreachable_code)] // will be when not debug_assertions
                VoxelChunk::empty()
            }
        }
    }
}

struct ChunkBlockIter<'a>(crate::utils::BlockIter<30, 30, 30>, &'a VoxelChunk);

impl<'a> ChunkBlockIter<'a> {
    fn new(chunk: &'a VoxelChunk) -> Self {
        Self(crate::utils::BlockIter::new(), chunk)
    }
}

impl<'a> Iterator for ChunkBlockIter<'a> {
    type Item = Blocks;
    fn next(&mut self) -> Option<Self::Item> {
        let (x, y, z) = self.0.next()?;
        Some(self.1.get_block(x, y, z))
    }
}
