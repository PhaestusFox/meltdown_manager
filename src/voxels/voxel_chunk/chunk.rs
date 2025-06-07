use bevy::{diagnostic::DiagnosticsStore, math::IVec3, platform::collections::HashMap, prelude::*};
use chunk_serde::CompressedChunkData;

use crate::voxels::{
    blocks::Blocks,
    cellular_automata::{CellData, NextStep},
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
        self.map.remove(id)
    }

    pub fn get_chunk(&self, id: &ChunkId) -> Option<Entity> {
        self.map.get(id).cloned()
    }

    fn save_chunk(
        &self,
        chunk: ChunkId,
        data: &Query<(&ChunkData, &Chunk<CellData>)>,
        path: &'static str,
    ) -> Result<(), ChunkManagerError> {
        let entity = self
            .get_chunk(&chunk)
            .ok_or(ChunkManagerError::NoEntity(chunk))?;
        let (blocks, automata) = data.get(entity)?;
        let blocks = Chunk::<Blocks>::from(blocks);
        let blocks = blocks.compress();
        let automata = automata.compress();
        todo!();
        Ok(())
    }

    pub fn update_chunk_order(&mut self) {
        let size = (self.higes - self.lowest).abs();
        self.map
            .sort_by_cached_key(|a, _| a.x + a.z * size.x + a.y * size.x * size.z);
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> {
        self.map.values().cloned()
    }
}

#[derive(thiserror::Error, Debug)]
enum ChunkManagerError {
    #[error("Failed to find Entity for {0}")]
    NoEntity(ChunkId),
    #[error("Failed to get data from Query: {0}")]
    NoData(#[from] bevy::ecs::query::QueryEntityError),
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
}

impl<T: Copy + Default> Chunk<T> {
    #[inline(always)]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> T {
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
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, to: T) {
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

    pub fn get_by_index_mut(&mut self, index: usize) -> &mut T {
        debug_assert!(index < CHUNK_VOL);
        &mut self.blocks[index]
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

struct ChunkBlockIter<'a, T>(crate::utils::BlockIter<30, 30, 30>, &'a Chunk<T>);

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

impl From<&ChunkData> for Chunk<Blocks> {
    fn from(data: &ChunkData) -> Self {
        let mut blocks = Vec::with_capacity(CHUNK_VOL);
        let mut same = true;
        let first = data.get_block(0, 0, 0).unwrap_or(Blocks::Void);
        for (x, y, z) in crate::utils::BlockIter::<30, 30, 30>::new() {
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
