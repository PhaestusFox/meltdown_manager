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
    let step = StepMode::from_bits_retain(tick);
    for (id, data) in chunk {
        let Some(mut cell) = neighbours.get(id) else {
            #[cfg(debug_assertions)]
            warn!("Cell {:?} in {:?} is out of bounds?", id, neighbours.root());
            continue; // skip if cell is void
        };
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                cell.get_block() != Blocks::Void,
                "Cell {:?} in {:?} is void",
                id,
                neighbours.root()
            );
        }
        for neighbour_id in id.neighbours() {
            let Some(neighbour_data) = neighbours.get(neighbour_id) else {
                continue; // skip if neighbour is out of bounds
            };
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

        if step.contains(StepMode::PHASE_CHANGE) {
            cell.set_phase();
        };
        cell.flags.remove(CellFlags::MOVE_ALL);

        // if step.contains(StepMode::BROWNIAN)
        //     && cell
        //         .flags
        //         .intersects(CellFlags::IS_LIQUID | CellFlags::IS_GAS)
        // // dont do brownian motion for solids
        // {
        //     for i in 0..6 {
        //         let (target, flag) = match i {
        //             0 => (id.up(), CellFlags::MOVE_UP),
        //             1 => (id.down(), CellFlags::MOVE_DOWN),
        //             2 => (id.left(), CellFlags::MOVE_LEFT),
        //             3 => (id.right(), CellFlags::MOVE_RIGHT),
        //             4 => (id.forward(), CellFlags::MOVE_FORWARD),
        //             5 => (id.backward(), CellFlags::MOVE_BACK),
        //             _ => unreachable!(),
        //         };
        //         let Some(target) = neighbours.get(target) else {
        //             continue; // skip if target is out of bounds
        //         };
        //         if !target.can_move() {
        //             continue; // skip if target cannot move
        //         }
        //         // break as soon as we have found a move
        //         if cell.is_liquid() {
        //             if i == 0 {
        //                 // skip up on liquids
        //                 continue;
        //             }
        //             if i == 1 {
        //                 if target.is_gas() {
        //                     cell.flags.insert(CellFlags::MOVE_DOWN);
        //                     break;
        //                 } else if target.properties().density < cell.properties().density {
        //                     // liquids can move down if the target is a liquid with lower density
        //                     cell.flags.insert(CellFlags::MOVE_DOWN);
        //                     break;
        //                 } else {
        //                     // todo fixed rng choice?
        //                 }
        //             }
        //         }
        //         if cell.is_gas() {
        //             if i == 1 {
        //                 // skip down on gases
        //                 continue;
        //             }
        //             if i == 0 {
        //                 if target.is_liquid() {
        //                     cell.flags.insert(CellFlags::MOVE_UP);
        //                     break;
        //                 } else if target.properties().density > cell.properties().density {
        //                     // gases can move up if the target is a gas with higher density
        //                     cell.flags.insert(CellFlags::MOVE_UP);
        //                     break;
        //                 } else {
        //                     // todo fixed rng choice?
        //                 }
        //             }
        //         }
        //     }
        // } else
        if step.contains(StepMode::GRAVITY) {
            // if neighbours.root() == ChunkId::new(2, 0, -3) && id.x < 5 && id.y == 0 && id.z == 0 {
            //     println!(
            //         "{}@{id:?} down: {:#?}",
            //         cell.get_block(),
            //         neighbours.get(id.down())
            //     );
            // }
            // if neighbours.root() == ChunkId::new(2, -1, -3) && id.x < 5 && id.y == 29 && id.z == 0 {
            //     println!(
            //         "{}@{id:?} up: {:#?}",
            //         cell.get_block(),
            //         neighbours.get(id.up())
            //     );
            // }

            let flag = check_gravity(id, &cell, &neighbours);
            cell.flags |= flag;
        }
        #[cfg(debug_assertions)]
        max.max(&cell);
        *data = cell;
    }
    max
}

fn check_gravity(id: CellId, cell: &CellData, neighbours: &ChunkGared) -> CellFlags {
    if !cell.can_move() {
        return CellFlags::empty();
    }
    if cell.is_liquid() {
        // liquids fall down
        let Some(other) = neighbours.get(id.down()) else {
            return CellFlags::empty();
        };
        if !other.can_move() {
            return CellFlags::empty();
        }
        if other.is_gas() || other.properties().density < cell.properties().density {
            CellFlags::MOVE_DOWN
        } else {
            // liquids cannot move down if the target is a liquid with higher density
            CellFlags::empty()
        }
    } else {
        // gases rise up
        let Some(other) = neighbours.get(id.up()) else {
            return CellFlags::empty();
        };
        if !other.can_move() {
            return CellFlags::empty();
        }
        if other.is_liquid() || other.properties().density > cell.properties().density {
            CellFlags::MOVE_UP
        } else {
            // gases cannot move up if the target is a gas with lower density
            CellFlags::empty()
        }
    }
}

bitflags::bitflags! {
    #[derive(Default, Clone, Copy)]
    pub struct StepMode: usize {
        const PHASE_CHANGE = 1;
        const GRAVITY = 3;
        const BROWNIAN = !(-1 << 3) as usize;
        const PHYSICS = StepMode::GRAVITY.bits() | StepMode::BROWNIAN.bits();
    }
}

pub fn is_step(step: StepMode) -> impl Fn(Res<VoxelTick>) -> bool {
    move |tick: Res<VoxelTick>| StepMode::from_bits_retain(tick.get()).intersects(step)
}

#[cfg(test)]
mod test;
