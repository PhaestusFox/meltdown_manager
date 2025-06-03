const CHUNK_SIZE: i32 = crate::voxels::map::CHUNK_SIZE as i32;
const SUM_DIVISOR: FixedNum = FixedNum::lit("6.0");

use crate::voxels::{
    blocks::{self, Blocks},
    cellular_automata::FixedNum,
    map::ChunkData,
    voxel_chunk::chunk::{Chunk, Neighbours},
};

use super::*;

pub fn step<'a>(chunk: ChunkIter<'a>, neighbours: ChunkGared<'a>) -> CellData {
    let mut max = CellData::MIN;
    for (id, data, block) in chunk {
        let mut sum = CellData::ZERO;
        for neighbour_id in id.neighbours() {
            let neighbour_data = neighbours.get(neighbour_id);
            sum += neighbour_data;
        }
        sum /= SUM_DIVISOR;
        sum.temperature += block.block_properties().heat;
        *data = sum;
        data.presure = FixedNum::ZERO; // Placeholder for pressure logic
        data.charge = FixedNum::ZERO; // Placeholder for charge logic
        max.max(data);
    }
    max
}
