const CHUNK_SIZE: i32 = crate::voxels::map::CHUNK_SIZE;
const SUM_DIVISOR: FixedNum = FixedNum::lit("6.0");

use crate::voxels::{
    blocks::{self, Blocks},
    cellular_automata::{FixedNum, cells::CellFlags},
    map::ChunkData,
    voxel_chunk::*,
};

use super::*;

pub fn step<'a>(chunk: ChunkIter<'a>, neighbours: ChunkGared<'a>, tick: usize) {
    step_diag(chunk, neighbours, tick);
}

pub fn step_diag<'a>(chunk: ChunkIter<'a>, neighbours: ChunkGared<'a>, tick: usize) -> CellData {
    let mut max = CellData::MIN;
    let step = Steps::from_bits_retain(tick);
    for (id, data) in chunk {
        let mut cell = neighbours.get(id);
        debug_assert!(cell.get_block() != Blocks::Void, "Cell at {:?} is void", id);
        for neighbour_id in id.neighbours() {
            let neighbour_data = neighbours.get(neighbour_id);
            let t1 = cell.temperature();
            let t2 = neighbour_data.temperature();
            let delta_t = t2 - t1;
            let g = cell.lookup_g(neighbour_data.get_block());
            let heat_transfer = g * delta_t;
            cell.energy += heat_transfer;
        }
        if cell.get_block() == Blocks::Uranium {
            cell.energy += FixedNum::lit("10."); // hack to add uranium heat without changing my meta code
        }

        if step.contains(Steps::PHASE_CHANGE) {
            cell.set_phase();
        };
        if step.contains(Steps::GRAVITY) {
            let block = cell.get_block();
            cell.flags.remove(CellFlags::SINK | CellFlags::FLOAT);
            if cell.flags.contains(CellFlags::IS_LIQUID) {
                let down = neighbours.get(id.down());
                let ob = down.get_block();
                if ob != Blocks::Void // dont swap with void
                    && ob != block // dont swap with same block
                    && down
                        .flags
                        .intersects(CellFlags::IS_LIQUID | CellFlags::IS_GAS) // liquids will swap with liquids and gases
                    && down.get_block().properties().density < cell.get_block().properties().density
                // swawp if down is less dense
                {
                    cell.flags.set(CellFlags::SINK, true);
                }
            } else if cell.flags.contains(CellFlags::IS_GAS) {
                let up = neighbours.get(id.up());
                let ob = up.get_block();
                if up.get_block() != Blocks::Void // dont swap with void
                    && ob != block // dont swap with same block
                    && up
                        .flags
                        .intersects(CellFlags::IS_LIQUID | CellFlags::IS_GAS) // gases will swap with liquids and gases
                    && up.get_block().properties().density > cell.get_block().properties().density
                // swap if up is more dense
                {
                    cell.flags.set(CellFlags::FLOAT, true);
                }
            }
        }
        #[cfg(debug_assertions)]
        max.max(&cell);
        *data = cell;
    }
    max
}

bitflags::bitflags! {
    #[derive(Default, Clone, Copy)]
    pub struct Steps: usize {
        const PHASE_CHANGE = 1;
        const GRAVITY = 2;
    }
}

pub fn is_step(step: Steps) -> impl Fn(Res<VoxelTick>) -> bool {
    move |tick: Res<VoxelTick>| Steps::from_bits_retain(tick.get()).intersects(step)
}
