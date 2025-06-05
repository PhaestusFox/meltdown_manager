const CHUNK_SIZE: i32 = crate::voxels::map::CHUNK_SIZE ;
const SUM_DIVISOR: FixedNum = FixedNum::lit("6.0");

use crate::voxels::{
    blocks::{self, Blocks},
    cellular_automata::{FixedNum, cells::CellFlags},
    map::ChunkData,
    voxel_chunk::chunk::{Chunk, Neighbours},
};

use super::*;

pub fn step<'a>(chunk: ChunkIter<'a>, neighbours: ChunkGared<'a>) {
    for (id, data) in chunk {
        let mut cell = neighbours.get(id);
        debug_assert!(cell.block != Blocks::Void, "Cell at {:?} is void", id);
        let block_properties = cell.block.block_properties();
        for neighbour_id in id.neighbours() {
            let neighbour_data = neighbours.get(neighbour_id);
            let t1 = cell.temperature();
            let t2 = neighbour_data.temperature();
            let delta_t = t2 - t1;
            let g = cell.lookup_g(neighbour_data.block);
            let heat_transfer = g * delta_t;
            cell.energy += heat_transfer;
        }
        cell.energy += block_properties.heat;
        cell.set_phase();
        *data = cell;
    }
}

pub fn step_diag<'a>(chunk: ChunkIter<'a>, neighbours: ChunkGared<'a>) -> CellData {
    let mut max = CellData::MIN;
    for (id, data) in chunk {
        let mut cell = neighbours.get(id);
        debug_assert!(cell.block != Blocks::Void, "Cell at {:?} is void", id);
        let block_properties = cell.block.block_properties();
        for neighbour_id in id.neighbours() {
            let neighbour_data = neighbours.get(neighbour_id);
            let t1 = cell.temperature();
            let t2 = neighbour_data.temperature();
            let delta_t = t2 - t1;
            let g = cell.lookup_g(neighbour_data.block);
            let heat_transfer = g * delta_t;
            cell.energy += heat_transfer;
        }
        cell.energy += block_properties.heat;
        cell.set_phase();
        max.max(&cell);
        *data = cell;
    }
    max
}

#[test]
fn test_cell_compression() {
    let copper = CellData {
        block: Blocks::Copper,
        energy: FixedNum::from_num(4000),
        charge: FixedNum::from_num(10),
        presure: FixedNum::from_num(1),
        flags: CellFlags::empty(),
    };

    let uranium = CellData {
        block: Blocks::Uranium,
        energy: FixedNum::from_num(1000),
        charge: FixedNum::from_num(5),
        presure: FixedNum::from_num(2),
        flags: CellFlags::empty(),
    };

    assert_ne!(copper.temperature(), uranium.temperature());
}
