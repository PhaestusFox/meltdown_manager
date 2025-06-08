use bevy::prelude::*;
use chunk_serde::CompressedChunkData;

use crate::voxels::{
    block::BlockType,
    cellular_automata::{CellData, CellId, Cells, NextStep, TargetTick, VoxelStep, VoxelTick},
    map::{CHUNK_AREA, CHUNK_SIZE, CHUNK_VOL, ChunkData},
    voxel_chunk::ChunkId,
};

#[derive(Default, Resource)]
pub struct ChunkManager {
    map: indexmap::IndexMap<ChunkId, Entity>,
    // the lowest x,y,z of any chunk
    lowest: ChunkId,
    // the highest x,y,z of any chunk
    higes: ChunkId,
}

impl ChunkManager {
    pub fn insert_chunk(&mut self, id: ChunkId, entity: Entity) -> Option<Entity> {
        self.lowest = self.lowest.min(id);
        self.higes = self.higes.max(id);
        self.map.insert(id, entity)
    }
    pub fn remove_chunk(&mut self, id: &ChunkId) -> Option<Entity> {
        self.map.swap_remove(id)
    }

    pub fn get_chunk(&self, id: &ChunkId) -> Option<Entity> {
        self.map.get(id).cloned()
    }

    pub fn save_chunk(
        &self,
        chunk: ChunkId,
        data: &Query<&Cells>,
    ) -> Result<Vec<u8>, ChunkManagerError> {
        let Some(entity) = self.get_chunk(&chunk) else {
            return Err(ChunkManagerError::NoEntity(chunk));
        };
        let cells = data.get(entity)?;
        let mut serde = chunk_serde::BinSerializer::new();
        serde
            .insert(b"PhoxC")
            .map_err(ChunkManagerError::SerdeError)?;
        let compressed = cells.compress();
        serde
            .insert(&compressed)
            .map_err(ChunkManagerError::SerdeError)?;
        Ok(serde.finalize())
    }

    pub fn save_world(
        &self,
        data: &Query<&Cells>,
        tick: u64,
    ) -> Result<Vec<u8>, ChunkManagerError> {
        let mut serde = chunk_serde::BinSerializer::new();
        serde
            .insert(b"PhoxW")
            .map_err(ChunkManagerError::SerdeError)?;
        serde.insert(&tick).map_err(ChunkManagerError::SerdeError)?;
        let len = self.len() as u64;
        serde.insert(&len).map_err(ChunkManagerError::SerdeError)?;
        for (id, entity) in self.map.iter() {
            let cells = data.get(*entity)?;
            let compressed = cells.compress();
            serde.insert(id).map_err(ChunkManagerError::SerdeError)?;
            serde
                .insert(&compressed)
                .map_err(ChunkManagerError::SerdeError)?;
        }
        Ok(serde.finalize())
    }

    pub fn load_chunk(
        &self,
        id: ChunkId,
        data: &[u8],
        commands: &mut Commands,
    ) -> Result<(), ChunkManagerError> {
        let mut serde = chunk_serde::BinDeSerializer::new(data);
        let magic = serde
            .extract::<[u8; 5]>()
            .map_err(ChunkManagerError::SerdeError)?;
        if magic != *b"PhoxC" {
            return Err(ChunkManagerError::SerdeError(
                bevy::ecs::error::BevyError::from("Attempted to load world as chunk data"),
            ));
        }
        let compressed = serde
            .extract::<CompressedChunkData<CellData>>()
            .map_err(ChunkManagerError::SerdeError)?;
        let cells = Cells::decompress(&compressed);
        if let Some(entity) = self.get_chunk(&id) {
            commands.entity(entity).remove::<NextStep>().insert(cells);
        } else {
            commands.spawn((cells, id));
        }
        Ok(())
    }

    pub fn load_world(
        &self,
        data: &[u8],
        commands: &mut Commands,
    ) -> Result<(), ChunkManagerError> {
        let mut serde = chunk_serde::BinDeSerializer::new(data);
        let magic = serde
            .extract::<[u8; 5]>()
            .map_err(ChunkManagerError::SerdeError)
            .unwrap();
        if magic != *b"PhoxW" {
            return Err(ChunkManagerError::SerdeError(
                bevy::ecs::error::BevyError::from("Attempted to load chunk data as world data"),
            ));
        }
        let tick = serde
            .extract::<u64>()
            .map_err(ChunkManagerError::SerdeError)?;
        let len = serde
            .extract::<u64>()
            .map_err(ChunkManagerError::SerdeError)?;

        for _ in 0..len {
            let id = serde.extract::<ChunkId>().unwrap();
            let compressed = serde
                .extract::<CompressedChunkData<CellData>>()
                .map_err(ChunkManagerError::SerdeError)?;
            let cells = Cells::decompress(&compressed);

            if let Some(entity) = self.get_chunk(&id) {
                commands
                    .entity(entity)
                    .remove::<(NextStep, ChunkData)>()
                    .insert(cells);
            } else {
                commands.spawn((cells, id));
            }
        }
        commands.insert_resource(VoxelTick::new(tick));
        commands.insert_resource(TargetTick::new(tick));
        commands.insert_resource(VoxelStep::default());
        Ok(())
    }

    pub fn save_compressed_chunk(
        &self,
        chunk: ChunkId,
        data: &Query<&Chunk<CellData>>,
    ) -> Result<Vec<u8>, ChunkManagerError> {
        let Some(entity) = self.get_chunk(&chunk) else {
            return Err(ChunkManagerError::NoEntity(chunk));
        };
        let chunk_data = data.get(entity)?;
        let mut min = Chunk::<BlockType>::empty();
        for (i, b) in chunk_data.blocks.iter().enumerate() {
            min.set_by_index(i, b.get_block_type());
        }
        let compressed = min.compress();
        let mut serde = chunk_serde::BinSerializer::new();
        serde
            .insert(b"PhoxK")
            .map_err(ChunkManagerError::SerdeError)?;
        serde
            .insert(&compressed)
            .map_err(ChunkManagerError::SerdeError)?;
        Ok(serde.finalize())
    }

    pub fn save_compressed_world(
        &self,
        data: &Query<&Chunk<CellData>>,
        tick: u64,
    ) -> Result<Vec<u8>, ChunkManagerError> {
        let mut serde = chunk_serde::BinSerializer::new();
        serde
            .insert(b"PhoxM")
            .map_err(ChunkManagerError::SerdeError)?;
        let len = self.len() as u64;
        serde.insert(&tick).map_err(ChunkManagerError::SerdeError)?;
        serde.insert(&len).map_err(ChunkManagerError::SerdeError)?;
        let mut min = Chunk::<BlockType>::empty();
        for (id, entity) in self.map.iter() {
            let chunk_data = data.get(*entity)?;
            serde.insert(id).map_err(ChunkManagerError::SerdeError)?;
            for (i, b) in chunk_data.blocks.iter().enumerate() {
                min.set_by_index(i, b.get_block_type());
            }
            let compressed = min.compress();
            serde
                .insert(&compressed)
                .map_err(ChunkManagerError::SerdeError)?;
        }
        Ok(serde.finalize())
    }

    pub fn load_compressed_chunk(
        &self,
        id: ChunkId,
        data: &[u8],
        commands: &mut Commands,
    ) -> Result<(), ChunkManagerError> {
        let mut serde = chunk_serde::BinDeSerializer::new(data);
        let magic = serde
            .extract::<[u8; 5]>()
            .map_err(ChunkManagerError::SerdeError)?;
        if magic != *b"PhoxK" {
            return Err(ChunkManagerError::SerdeError(
                bevy::ecs::error::BevyError::from("Attempted to load world as chunk data"),
            ));
        }
        let compressed = serde
            .extract::<CompressedChunkData<BlockType>>()
            .map_err(ChunkManagerError::SerdeError)?;
        let chunk_data = Chunk::<BlockType>::decompress(&compressed);
        let mut chunk = Chunk::empty();
        for (i, b) in chunk_data.blocks.iter().enumerate() {
            let mut block = CellData::default();
            block.set_block_type(*b);
            chunk.set_by_index(i, block);
        }
        if let Some(entity) = self.get_chunk(&id) {
            commands.entity(entity).remove::<NextStep>().insert(chunk);
        } else {
            commands.spawn((chunk, id));
        }
        Ok(())
    }

    pub fn load_compressed_world(
        &self,
        data: &[u8],
        commands: &mut Commands,
    ) -> Result<(), ChunkManagerError> {
        let mut serde = chunk_serde::BinDeSerializer::new(data);
        let magic = serde
            .extract::<[u8; 5]>()
            .map_err(ChunkManagerError::SerdeError)
            .unwrap();
        if magic != *b"PhoxM" {
            panic!("Attempted to load chunk data as world data");
        }
        let tick = serde
            .extract::<u64>()
            .map_err(ChunkManagerError::SerdeError)
            .unwrap();
        let len = serde
            .extract::<u64>()
            .map_err(ChunkManagerError::SerdeError)
            .unwrap();

        for _ in 0..len {
            let id = serde
                .extract::<ChunkId>()
                .map_err(ChunkManagerError::SerdeError)?;
            let compressed = serde
                .extract::<CompressedChunkData<BlockType>>()
                .map_err(ChunkManagerError::SerdeError)?;
            let chunk_data = Chunk::<BlockType>::decompress(&compressed);
            let mut chunk = Chunk::empty();
            for (i, b) in chunk_data.blocks.iter().enumerate() {
                let mut block = CellData::default();
                block.set_block_type(*b);
                chunk.set_by_index(i, block);
            }
            if let Some(entity) = self.get_chunk(&id) {
                commands
                    .entity(entity)
                    .remove::<(NextStep, ChunkData)>()
                    .insert(chunk);
            } else {
                commands.spawn((chunk, id));
            }
        }
        commands.insert_resource(VoxelTick::new(tick));
        commands.insert_resource(TargetTick::new(tick));
        commands.insert_resource(VoxelStep::default());
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn update_chunk_order(&mut self) {
        let size = (self.higes - self.lowest).abs();
        self.map
            .sort_by_cached_key(|a, _| a.x + a.z * size.x + a.y * size.x * size.z);
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> {
        self.map.values().cloned()
    }

    pub fn get_chunk_and_local_block(&self, x: i32, y: i32, z: i32) -> Option<(Entity, CellId)> {
        let chunk_id = ChunkId::from_translation(Vec3::new(x as f32, y as f32, z as f32));
        let entity = self.get_chunk(&chunk_id)?;
        let cell_id = CellId::new(
            x - chunk_id.x * CHUNK_SIZE,
            y - chunk_id.y * CHUNK_SIZE,
            z - chunk_id.z * CHUNK_SIZE,
        );
        Some((entity, cell_id))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ChunkManagerError {
    #[error("Failed to find Entity for {0}")]
    NoEntity(ChunkId),
    #[error("Failed to get data from Query: {0}")]
    NoData(#[from] bevy::ecs::query::QueryEntityError),
    #[error("Failed to open file: {0}")]
    FileOpen(#[from] std::io::Error),
    #[error("Failed to Sererialise: {0}")]
    SerdeError(bevy::ecs::error::BevyError),
}

#[derive(Component, Debug)]
pub struct Chunk<T> {
    is_single_block: bool,
    blocks: Vec<T>,
}

impl<T> Clone for Chunk<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            is_single_block: self.is_single_block,
            blocks: self.blocks.clone(),
        }
    }
}

impl<T> Chunk<T> {
    pub fn swap(&mut self, a: &CellId, b: &CellId) {
        #[cfg(debug_assertions)]
        if !Self::in_bounds(a.x, a.y, a.z) || !Self::in_bounds(b.x, b.y, b.z) {
            panic!(
                "Tried to swap out of bounds cells: ({}, {}, {}) and ({}, {}, {})",
                a.x, a.y, a.z, b.x, b.y, b.z
            );
        }
        let index_a = Self::index(a.x, a.y, a.z);
        let index_b = Self::index(b.x, b.y, b.z);
        self.blocks.swap(index_a, index_b);
    }
}

impl<T: PartialEq + Eq> Eq for Chunk<T> {}

impl<T: PartialEq> PartialEq for Chunk<T> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..CHUNK_VOL {
            if self.blocks[i] != other.blocks[i] {
                return false;
            }
        }
        true
    }
}

impl<T> Chunk<T> {
    #[inline(always)]
    pub fn index(x: i32, y: i32, z: i32) -> usize {
        (x + z * CHUNK_SIZE + y * CHUNK_AREA) as usize
    }

    #[inline(always)]
    fn in_bounds(x: i32, y: i32, z: i32) -> bool {
        x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE && x >= 0 && y >= 0 && z >= 0
    }

    #[inline(always)]
    pub fn set_by_index(&mut self, index: usize, to: T) {
        debug_assert!(index < CHUNK_VOL);
        self.blocks[index] = to;
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.blocks.iter_mut()
    }

    pub fn is_solid(&self) -> bool {
        self.is_single_block
    }

    pub fn set_not_solid(&mut self) {
        self.is_single_block = false;
    }
}

impl<T: Copy + Default> Chunk<T> {
    #[inline(always)]
    pub fn get_cell(&self, x: i32, y: i32, z: i32) -> T {
        if Self::in_bounds(x, y, z) {
            self.blocks[Self::index(x, y, z)]
        } else {
            #[cfg(debug_assertions)]
            panic!("Index({}, {}, {}) is out of bound", x, y, z);
            #[allow(unreachable_code)] // can when not in debug
            T::default()
        }
    }

    pub fn empty() -> Self {
        Self {
            is_single_block: true,
            blocks: vec![T::default(); CHUNK_VOL],
        }
    }

    pub fn solid(fill: T) -> Self {
        Self {
            is_single_block: true,
            blocks: vec![fill; CHUNK_VOL],
        }
    }
}

impl<T: Clone + PartialEq> Chunk<T> {
    #[inline(always)]
    pub fn set_cell(&mut self, x: i32, y: i32, z: i32, to: T) {
        if self.is_single_block && self.blocks[0] != to {
            self.is_single_block = false;
        }
        if Self::in_bounds(x, y, z) {
            self.blocks[Self::index(x, y, z)] = to;
        } else {
            #[cfg(debug_assertions)]
            panic!("Index({}, {}, {}) is out of bound", x, y, z);
        }
    }
}

impl<T: Copy> Chunk<T> {
    #[inline]
    pub fn blocks(&self) -> impl Iterator<Item = T> {
        ChunkBlockIter::new(self)
    }

    pub fn get_by_index(&self, index: usize) -> T {
        debug_assert!(index < CHUNK_VOL);
        self.blocks[index]
    }
}

impl<T: Copy + PartialEq> Chunk<T> {
    pub fn get_by_index_mut(&mut self, index: usize) -> BlockGarde<'_, T> {
        debug_assert!(index < CHUNK_VOL);
        BlockGarde {
            index,
            chunk: self,
            changed: false,
        }
    }
}

pub struct BlockGarde<'a, T: Copy + PartialEq> {
    index: usize,
    chunk: &'a mut Chunk<T>,
    changed: bool,
}

impl<'a, T: Copy + PartialEq> core::ops::Deref for BlockGarde<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.chunk.blocks[self.index]
    }
}

impl<'a, T: Copy + PartialEq> core::ops::DerefMut for BlockGarde<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed = true;
        &mut self.chunk.blocks[self.index]
    }
}

impl<T: PartialEq + Copy> Drop for BlockGarde<'_, T> {
    fn drop(&mut self) {
        if self.changed && self.chunk.is_single_block {
            let test = if self.index == 0 {
                self.chunk.blocks[1]
            } else {
                self.chunk.blocks[0]
            };
            if self.chunk.blocks[self.index] != test {
                self.chunk.is_single_block = false;
            }
        }
    }
}

impl<T: PartialEq + Copy + Default> Chunk<T> {
    pub fn compress(&self) -> CompressedChunkData<T> {
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
            return CompressedChunkData::Raw(self.blocks.clone());
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

    pub fn decompress(data: &CompressedChunkData<T>) -> Chunk<T> {
        match data {
            CompressedChunkData::Solid(block) => Chunk::solid(*block),
            CompressedChunkData::RunLen(runs) => {
                let mut chunk = Chunk::empty();
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
                let mut chunk = Chunk::empty();
                chunk.blocks.copy_from_slice(&items[..CHUNK_VOL]);
                chunk
            }
            CompressedChunkData::Error(i) => {
                error!("Got Compression Error({i}); this is a bug dont decompress Errors");
                #[cfg(debug_assertions)]
                panic!(
                    "Got Compression Error({i});\n
                Can't Decompress and error;\n
                This will return Chunk::empty() in release
                "
                );
                #[allow(unreachable_code)] // will be when not debug_assertions
                Chunk::empty()
            }
        }
    }
}

struct ChunkBlockIter<'a, T>(crate::utils::BlockIter, &'a Chunk<T>);

impl<'a, T> ChunkBlockIter<'a, T> {
    fn new(chunk: &'a Chunk<T>) -> Self {
        Self(crate::utils::BlockIter::new(), chunk)
    }
}

impl<'a, T: Copy> Iterator for ChunkBlockIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let (x, y, z) = self.0.next()?;
        Some(self.1.blocks[Chunk::<T>::index(x, y, z)])
    }
}

impl From<&ChunkData> for Chunk<BlockType> {
    fn from(data: &ChunkData) -> Self {
        let mut blocks = Vec::with_capacity(CHUNK_VOL);
        let mut same = true;
        let first = data.get_block(0, 0, 0).unwrap_or(BlockType::Void);
        for (x, y, z) in crate::utils::BlockIter::new() {
            let Some(block) = data.get_block(x as u32, y as u32, z as u32) else {
                panic!("Invalid Block at ({}, {}, {}); {:?}", x, y, z, data);
            };
            same &= block == first;
            blocks.push(block);
        }
        Chunk {
            is_single_block: same,
            blocks,
        }
    }
}
