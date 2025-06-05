use bevy::{diagnostic::DiagnosticsStore, math::IVec3, platform::collections::HashMap, prelude::*};
use chunk_serde::CompressedChunkData;

use crate::voxels::{
    blocks::Blocks,
    cellular_automata::CellData,
    map::{CHUNK_ARIA, CHUNK_SIZE, CHUNK_VOL, ChunkData},
};

#[derive(Component, Deref, Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Reflect)]
#[component(immutable, on_insert = ChunkId::on_insert, on_remove = ChunkId::on_remove, on_add = ChunkId::on_add, on_despawn = ChunkId::on_despawn)]
#[require(Transform, Neighbours)]
pub struct ChunkId(IVec3);

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Chunk({},{},{})", self.x, self.y, self.z))
    }
}

impl ChunkId {
    pub const ZERO: ChunkId = ChunkId(IVec3::ZERO);

    pub fn neighbour(&self, direction: NeighbourDirection) -> ChunkId {
        match direction {
            NeighbourDirection::Up => ChunkId(self.0 + IVec3::Y),
            NeighbourDirection::Down => ChunkId(self.0 - IVec3::Y),
            NeighbourDirection::Left => ChunkId(self.0 + IVec3::X),
            NeighbourDirection::Right => ChunkId(self.0 - IVec3::X),
            NeighbourDirection::Front => ChunkId(self.0 + IVec3::Z),
            NeighbourDirection::Back => ChunkId(self.0 - IVec3::Z),
        }
    }

    pub fn new(x: i32, y: i32, z: i32) -> ChunkId {
        Self(IVec3::new(x, y, z))
    }

    pub fn manhattan_distance(self, other: &ChunkId) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()) as u32
    }

    fn on_add(mut world: bevy::ecs::world::DeferredWorld, _ctx: bevy::ecs::component::HookContext) {
        world.resource_mut::<crate::diagnostics::ChunkCount>().inc();
    }

    fn on_despawn(
        mut world: bevy::ecs::world::DeferredWorld,
        _ctx: bevy::ecs::component::HookContext,
    ) {
        world.resource_mut::<crate::diagnostics::ChunkCount>().dec();
    }

    fn on_insert(
        mut world: bevy::ecs::world::DeferredWorld,
        ctx: bevy::ecs::component::HookContext,
    ) {
        let id = *world
            .get::<ChunkId>(ctx.entity)
            .expect("This Just got inserted");
        world
            .get_mut::<Transform>(ctx.entity)
            .expect("Required Componet")
            .translation = (id.0 * CHUNK_SIZE).as_vec3();

        if world.get::<Name>(ctx.entity).is_none() {
            world
                .commands()
                .entity(ctx.entity)
                .insert(Name::new(format!("{}", id)));
        }

        let mut neighbours = world
            .get_mut::<Neighbours>(ctx.entity)
            .expect("Required Componet");
        let too_apply = EmptyNeighboursIter::new(&mut neighbours).collect::<Vec<_>>();

        let manager = world.resource::<ChunkManager>();
        let mut can_apply = Vec::with_capacity(too_apply.len());
        let mut recip = Vec::with_capacity(too_apply.len());
        for (apply, direction) in too_apply {
            if let Some(other) = manager.get_chunk(&id.neighbour(direction)) {
                can_apply.push((apply, other));
                recip.push((other, direction.rev()));
            }
        }

        let mut neighbours = world
            .get_mut::<Neighbours>(ctx.entity)
            .expect("Required Componet");
        for (apply, other) in can_apply {
            apply(&mut neighbours, other);
        }

        for (other, direction) in recip {
            if let Some(mut neighbours) = world.get_mut::<Neighbours>(other) {
                match direction {
                    NeighbourDirection::Up => neighbours.up = Some(ctx.entity),
                    NeighbourDirection::Down => neighbours.down = Some(ctx.entity),
                    NeighbourDirection::Left => neighbours.left = Some(ctx.entity),
                    NeighbourDirection::Right => neighbours.right = Some(ctx.entity),
                    NeighbourDirection::Front => neighbours.front = Some(ctx.entity),
                    NeighbourDirection::Back => neighbours.back = Some(ctx.entity),
                }
            } else {
                warn!("Failed to get Neighbours for {other:?} this is probably a bug");
            }
        }

        let mut manager = world.resource_mut::<ChunkManager>();

        if let Some(old) = manager.insert_chunk(id, ctx.entity) {
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

    pub fn from_translation(mut translation: Vec3) -> Self {
        translation /= CHUNK_SIZE as f32;
        ChunkId(translation.as_ivec3())
    }
}

#[derive(Default, Resource)]
pub struct ChunkManager {
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
    blocks: Vec<T>,
}

impl<T> Clone for Chunk<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
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
        (x + z * CHUNK_SIZE + y * CHUNK_ARIA) as usize
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

    #[inline(always)]
    pub fn set_block(&mut self, x: i32, y: i32, z: i32, to: T) {
        if Self::in_bounds(x, y, z) {
            self.blocks[Self::index(x, y, z)] = to;
        } else {
            #[cfg(debug_assertions)]
            panic!("Index({}, {}, {}) is out of bound", x, y, z);
        }
    }

    pub fn empty() -> Self {
        Self {
            blocks: vec![T::default(); CHUNK_VOL],
        }
    }

    pub fn solid(fill: T) -> Self {
        Self {
            blocks: vec![fill; CHUNK_VOL],
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

#[derive(Component, Default, Debug)]
pub struct Neighbours {
    up: Option<Entity>,
    down: Option<Entity>,
    left: Option<Entity>,
    right: Option<Entity>,
    front: Option<Entity>,
    back: Option<Entity>,
}

struct EmptyNeighboursIter<'a> {
    neighbours: &'a mut Neighbours,
    index: usize,
}

pub struct NeighboursIter<'a> {
    neighbours: &'a Neighbours,
    index: usize,
}

impl Iterator for NeighboursIter<'_> {
    type Item = (NeighbourDirection, Entity);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < 6 {
            let idx = self.index;
            self.index += 1;
            let out = match idx {
                0 => self.neighbours.up(),
                1 => self.neighbours.down(),
                2 => self.neighbours.left(),
                3 => self.neighbours.right(),
                4 => self.neighbours.front(),
                5 => self.neighbours.back(),
                _ => None,
            };
            if let Some(out) = out {
                return Some((NeighbourDirection::from_index(idx), out));
            }
        }
        None
    }
}

impl<'a> EmptyNeighboursIter<'a> {
    fn new(neighbours: &'a mut Neighbours) -> Self {
        Self {
            neighbours,
            index: 0,
        }
    }
}

impl<'a> Iterator for EmptyNeighboursIter<'a> {
    type Item = (fn(&mut Neighbours, Entity), NeighbourDirection);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= 6 {
            return None;
        }
        let idx = self.index;
        let entry = match idx {
            0 => self.neighbours.up.is_none(),
            1 => self.neighbours.down.is_none(),
            2 => self.neighbours.left.is_none(),
            3 => self.neighbours.right.is_none(),
            4 => self.neighbours.front.is_none(),
            5 => self.neighbours.back.is_none(),
            _ => unreachable!(),
        };
        self.index += 1;
        if entry {
            let f: fn(&mut Neighbours, Entity) = match idx {
                0 => |n, e| n.up = Some(e),
                1 => |n, e| n.down = Some(e),
                2 => |n, e| n.left = Some(e),
                3 => |n, e| n.right = Some(e),
                4 => |n, e| n.front = Some(e),
                5 => |n, e| n.back = Some(e),
                _ => unreachable!(),
            };
            let id = NeighbourDirection::from_index(idx);
            Some((f, id))
        } else {
            self.next()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NeighbourDirection {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

impl NeighbourDirection {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Up,
            1 => Self::Down,
            2 => Self::Left,
            3 => Self::Right,
            4 => Self::Front,
            5 => Self::Back,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!(); // this should never happen, but if it does, panic in debug mode
                #[allow(unreachable_code)]
                Self::Up // default to up if in release mode
            }
        }
    }

    pub fn rev(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Front => Self::Back,
            Self::Back => Self::Front,
        }
    }
}

impl Neighbours {
    pub fn up(&self) -> Option<Entity> {
        self.up
    }
    pub fn down(&self) -> Option<Entity> {
        self.down
    }
    pub fn left(&self) -> Option<Entity> {
        self.left
    }
    pub fn right(&self) -> Option<Entity> {
        self.right
    }
    pub fn front(&self) -> Option<Entity> {
        self.front
    }
    pub fn back(&self) -> Option<Entity> {
        self.back
    }

    pub fn iter(&self) -> NeighboursIter {
        NeighboursIter {
            neighbours: self,
            index: 0,
        }
    }
}

impl From<&ChunkData> for Chunk<Blocks> {
    fn from(data: &ChunkData) -> Self {
        let mut blocks = Vec::with_capacity(CHUNK_VOL);
        for (x, y, z) in crate::utils::BlockIter::<30, 30, 30>::new() {
            let Some(block) = data.get_block(x as u32, y as u32, z as u32) else {
                panic!("Invalid Block at ({}, {}, {}); {:?}", x, y, z, data);
            };
            blocks.push(block);
        }
        Chunk { blocks }
    }
}
