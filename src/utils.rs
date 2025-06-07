use crate::voxels::{CHUNK_SIZE, CHUNK_VOL};
use bevy::{
    math::IVec3,
    platform::collections::HashSet,
    prelude::{Deref, DerefMut},
    render::render_graph::Edge,
};

use crate::voxels::cellular_automata::CellId;

pub struct BlockIter {
    x: i32,
    y: i32,
    z: i32,
}

impl BlockIter {
    pub fn new() -> BlockIter {
        BlockIter { x: 0, y: 0, z: 0 }
    }
}

impl Default for BlockIter {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for BlockIter {
    type Item = (i32, i32, i32);
    fn next(&mut self) -> Option<Self::Item> {
        let out = if self.y >= CHUNK_SIZE {
            return None;
        } else {
            (self.x, self.y, self.z)
        };
        self.x += 1;
        if self.x >= CHUNK_SIZE {
            self.x -= CHUNK_SIZE;
            self.z += 1;
        }
        if self.z >= CHUNK_SIZE {
            self.z -= CHUNK_SIZE;
            self.y += 1
        }
        Some(out)
    }
}

pub struct CoreIter(IVec3);

impl CoreIter {
    pub fn new() -> Self {
        CoreIter(IVec3::ONE)
    }
}

impl CoreIter {
    const SIZE: i32 = CHUNK_SIZE - 2;
}

impl Iterator for CoreIter {
    type Item = CellId;
    fn next(&mut self) -> Option<Self::Item> {
        //let out = if self.y >= CHUNK_SIZE {
        //     return None;
        // } else {
        //     (self.x, self.y, self.z)
        // };
        // self.x += 1;
        // if self.x >= CHUNK_SIZE {
        //     self.x -= CHUNK_SIZE;
        //     self.z += 1;
        // }
        // if self.z >= CHUNK_SIZE {
        //     self.z -= CHUNK_SIZE;
        //     self.y += 1
        // }
        if self.0.y > Self::SIZE {
            return None;
        }
        let cell = CellId::from_vec(self.0);
        self.0.x += 1;
        if self.0.x > CoreIter::SIZE {
            self.0.x = 1;
            self.0.z += 1;
        }
        if self.0.z > CoreIter::SIZE {
            self.0.z = 1;
            self.0.y += 1;
        }
        Some(cell)
    }
}

#[derive(Deref, DerefMut)]
pub struct EdgeIter(IVec3);

impl EdgeIter {
    pub fn new() -> Self {
        EdgeIter(IVec3::ZERO)
    }
}

impl Iterator for EdgeIter {
    type Item = CellId;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.y >= CHUNK_SIZE as i32 {
            return None;
        }
        let edge = self.0;
        if self.y == 0 || self.y == CHUNK_SIZE - 1 {
            self.x += 1;
        } else if self.z == 0 || self.z == CHUNK_SIZE - 1 {
            self.x += 1;
        } else {
            self.x += CHUNK_SIZE - 1;
        };
        if self.x >= CHUNK_SIZE {
            self.x = 0;
            self.z += 1;
        }
        if self.z >= CHUNK_SIZE {
            self.z = 0;
            self.y += 1;
        }
        Some(CellId::from_vec(edge))
    }
}

#[test]
fn edge_is_not_in_core() {
    let edge = EdgeIter::new().collect::<HashSet<_>>();
    let core = CoreIter::new().collect::<HashSet<_>>();
    let all = BlockIter::new()
        .map(|(x, y, z)| CellId::new(x, y, z))
        .collect::<HashSet<_>>();
    assert_eq!(edge.len(), CHUNK_VOL - (CHUNK_SIZE - 2).pow(3) as usize);
    assert_eq!(core.len(), (CHUNK_SIZE - 2).pow(3) as usize);
    assert_eq!(edge.len() + core.len(), CHUNK_VOL);
    assert_eq!(all.len(), CHUNK_VOL);
    assert_eq!(all, edge.union(&core).cloned().collect::<HashSet<_>>());

    // assert!(edge.union())
}
