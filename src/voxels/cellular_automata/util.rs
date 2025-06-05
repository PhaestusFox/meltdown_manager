use bevy::{
    math::IVec3,
    prelude::{Component, Deref, DerefMut},
};

use crate::voxels::{blocks::Blocks, cellular_automata::CellData, map::ChunkData};
const CHUNK_SIZE: i32 = crate::voxels::map::CHUNK_SIZE;
pub type Cells = crate::voxels::Chunk<CellData>;

#[derive(Component)]
pub struct NextStep {
    pub has_run: bool,
    pub(super) chunk: Cells,
}

impl Default for NextStep {
    fn default() -> Self {
        NextStep {
            has_run: false,
            chunk: Cells::solid(CellData::THE_VOID),
        }
    }
}

impl NextStep {
    /// this is so if I need I can find anyone touching the future state of a chunk
    /// Returns `None` if the step has not been simulated.
    /// If you changed it before it ran your change would be overwritten.
    pub fn borrow_mut(&mut self) -> Option<&mut Cells> {
        if !self.has_run {
            None
        } else {
            Some(&mut self.chunk)
        }
    }
}

#[derive(Clone, Copy, Deref, DerefMut, Debug)]
pub struct CellId(IVec3);

pub struct CellNeighbourIter(CellId, u8);

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

    pub fn neighbours(&self) -> impl Iterator<Item = CellId> {
        CellNeighbourIter(*self, 0)
    }
}

pub struct ChunkIter<'a> {
    id: CellId,
    data: std::slice::IterMut<'a, CellData>,
}

impl<'a> ChunkIter<'a> {
    pub fn new(chunk: &'a mut Cells) -> Self {
        let id = CellId(IVec3::new(0, 0, 0));
        ChunkIter {
            id,
            data: chunk.iter_mut(),
        }
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = (CellId, &'a mut CellData);

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
        self.data.next().map(|cell| (id, cell))
    }
}

pub struct ChunkBlock<'a> {
    core: &'a mut Cells,
    neighbours: ChunkGared<'a>,
}

pub struct ChunkGared<'a> {
    chunk: [Option<&'a Cells>; 7],
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
    pub fn new(chunks: [Option<&'a Cells>; 7]) -> Self {
        ChunkGared { chunk: chunks }
    }

    pub fn get(&self, id: CellId) -> CellData {
        let index = GaredIndex::from_id(id);
        let normalized_id = index.normalize_id(id);
        let Some(chunk) = self.get_chunk(index) else {
            return CellData::THE_VOID;
        };
        let index = Cells::index(normalized_id.x, normalized_id.y, normalized_id.z);
        chunk.get_by_index(index)
    }

    fn get_chunk(&self, index: GaredIndex) -> Option<&'a Cells> {
        self.chunk[index.to_index()]
    }
}
