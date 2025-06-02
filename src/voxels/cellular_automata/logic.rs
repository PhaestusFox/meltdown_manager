use bevy::prelude::*;

const CHUNK_SIZE: i32 = crate::voxels::map::CHUNK_SIZE as i32;
const SUM_DIVISOR: FixedNum = FixedNum::lit("6.0");

use crate::voxels::{
    blocks::{self, Blocks},
    cellular_automata::FixedNum,
    map::ChunkData,
    voxel_chunk::chunk::{Chunk, Neighbours},
};

use super::CellData;

#[derive(Component)]
pub struct PreviousStep(Chunk<CellData>);

#[derive(Clone, Copy, Deref, DerefMut)]
struct CellId(IVec3);

struct CellNeighbourIter(CellId, u8);

impl Iterator for CellNeighbourIter {
    type Item = CellId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.1 >= 6 {
            return None;
        }
        let offset = match self.1 {
            0 => IVec3::new(0, 1, 0),  // Up
            1 => IVec3::new(0, -1, 0), // Down
            2 => IVec3::new(1, 0, 0),  // Right
            3 => IVec3::new(-1, 0, 0), // Left
            4 => IVec3::new(0, 0, 1),  // Forward
            5 => IVec3::new(0, 0, -1), // Backward
            _ => unreachable!(),
        };
        self.1 += 1;
        Some(CellId(self.0.0 + offset))
    }
}

impl CellId {
    fn new(x: i32, y: i32, z: i32) -> Self {
        CellId(IVec3::new(x, y, z))
    }

    fn neighbours(&self) -> impl Iterator<Item = CellId> {
        CellNeighbourIter(*self, 0)
    }
}

struct ChunkIter<'a> {
    id: CellId,
    data: std::slice::IterMut<'a, CellData>,
    block_type: std::slice::Iter<'a, Blocks>,
}

impl<'a> ChunkIter<'a> {
    fn new(chunk: &'a mut Chunk<CellData>, blocks: &'a ChunkData) -> Self {
        let id = CellId(IVec3::new(0, 0, 0));
        ChunkIter {
            id,
            data: chunk.iter_mut(),
            block_type: blocks.iter(),
        }
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = (CellId, &'a mut CellData, Blocks);

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.id;
        self.id.x += 1;
        if self.id.x >= CHUNK_SIZE {
            self.id.x = 0;
            self.id.z += 1;
        }
        if self.id.z >= CHUNK_SIZE {
            self.id.z = 0;
            self.id.y += 1;
        }
        self.data.next().map(|cell| {
            (
                id,
                cell,
                self.block_type.next().cloned().unwrap_or(Blocks::Void),
            )
        })
    }
}

struct ChunkBlock<'a> {
    core: &'a mut Chunk<CellData>,
    neighbours: ChunkGared<'a>,
}

struct ChunkGared<'a> {
    chunk: [Option<&'a Chunk<CellData>>; 7],
}

#[derive(Clone, Copy)]
enum GaredIndex {
    Center = 0,
    Up = 1,
    Down = 1 << 1,
    Right = 1 << 2,
    Left = 1 << 3,
    Forward = 1 << 4,
    Backward = 1 << 5,
}

impl GaredIndex {
    fn from_id(id: CellId) -> GaredIndex {
        let mut index = 0;
        index |= if id.0.y >= CHUNK_SIZE {
            GaredIndex::Up as usize
        } else {
            0
        };
        index |= if id.0.y < 0 {
            GaredIndex::Down as usize
        } else {
            0
        };
        index |= if id.0.x >= CHUNK_SIZE {
            GaredIndex::Right as usize
        } else {
            0
        };
        index |= if id.0.x < 0 {
            GaredIndex::Left as usize
        } else {
            0
        };
        index |= if id.0.z >= CHUNK_SIZE {
            GaredIndex::Forward as usize
        } else {
            0
        };
        index |= if id.0.z < 0 {
            GaredIndex::Backward as usize
        } else {
            0
        };
        match index {
            0 => GaredIndex::Center,
            1 => GaredIndex::Up,
            2 => GaredIndex::Down,
            4 => GaredIndex::Right,
            8 => GaredIndex::Left,
            16 => GaredIndex::Forward,
            32 => GaredIndex::Backward,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!("Invalid GaredIndex: {}", index);
                #[allow(unreachable_code)]
                GaredIndex::Center // Fallback to center if invalid
            }
        }
    }

    fn to_index(self) -> usize {
        match self {
            GaredIndex::Center => 0,
            GaredIndex::Up => 1,
            GaredIndex::Down => 2,
            GaredIndex::Right => 3,
            GaredIndex::Left => 4,
            GaredIndex::Forward => 5,
            GaredIndex::Backward => 6,
        }
    }

    fn normalize_id(&self, mut id: CellId) -> CellId {
        match self {
            GaredIndex::Center => {}
            GaredIndex::Up => id.y = 0,
            GaredIndex::Down => {
                id.y = CHUNK_SIZE - 1;
            }
            GaredIndex::Right => {
                id.x = 0;
            }
            GaredIndex::Left => {
                id.x = CHUNK_SIZE - 1;
            }
            GaredIndex::Forward => {
                id.z = 0;
            }
            GaredIndex::Backward => {
                id.z = CHUNK_SIZE - 1;
            }
        }
        id
    }
}

impl<'a> ChunkGared<'a> {
    fn new(chunks: [Option<&'a Chunk<CellData>>; 7]) -> Self {
        ChunkGared { chunk: chunks }
    }

    fn get(&self, id: CellId) -> CellData {
        let index = GaredIndex::from_id(id);
        let normalized_id = index.normalize_id(id);
        let Some(chunk) = self.get_chunk(index) else {
            return CellData::THE_VOID;
        };
        let index = Chunk::<CellData>::index(normalized_id.x, normalized_id.y, normalized_id.z);
        chunk.get_by_index(index)
    }

    fn get_chunk(&self, index: GaredIndex) -> Option<&'a Chunk<CellData>> {
        self.chunk[index.to_index()]
    }
}

pub fn step_system(
    max: NonSend<super::diagnostics::MaxValue>,
    start_state: Query<&PreviousStep>,
    mut new_state: Query<(Entity, &mut Chunk<CellData>, &Neighbours, &ChunkData)>,
) {
    let sender = max.get_sender();
    new_state.par_iter_mut().for_each_init(
        || sender.clone(),
        |max, (center, mut chunk, neighbours, blocks)| {
            let Ok(center_pre) = start_state.get(center) else {
                return;
            };
            let mut chunks = [Some(&center_pre.0), None, None, None, None, None, None];
            for (i, n) in neighbours.iter() {
                if let Ok(neighbour) = start_state.get(n) {
                    chunks[i + 1] = Some(&neighbour.0);
                }
            }

            let out = step(
                ChunkIter::new(chunk.as_mut(), blocks),
                ChunkGared::new(chunks),
            );
            max.send(out);
        },
    );
}

fn step<'a>(chunk: ChunkIter<'a>, neighbours: ChunkGared<'a>) -> CellData {
    let mut max = CellData::MIN;
    for (id, data, block) in chunk {
        let mut sum = FixedNum::ZERO;
        for neighbour_id in id.neighbours() {
            let neighbour_data = neighbours.get(neighbour_id);
            sum += neighbour_data.temperature;
        }
        sum /= SUM_DIVISOR;
        sum += block.block_properties().heat;
        data.temperature = sum;
        data.presure = FixedNum::ZERO; // Placeholder for pressure logic
        data.charge = FixedNum::ZERO; // Placeholder for charge logic
        max.max(data);
    }
    max
}

pub fn set_prev(
    mut chunks: Query<(&mut Chunk<CellData>, &mut PreviousStep)>,
    mut to_init: Query<(Entity, &mut Chunk<CellData>), Without<PreviousStep>>,
    mut commands: Commands,
) {
    for (mut chunk, mut last) in &mut chunks {
        std::mem::swap(&mut last.0, &mut chunk);
    }
    for (entity, mut chunk) in &mut to_init {
        let prev = PreviousStep(std::mem::replace(chunk.as_mut(), Chunk::empty()));
        commands.entity(entity).insert(prev);
    }
}
