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
        let mut cell = neighbours.get(id);
        debug_assert!(
            cell.get_block() != Blocks::Void,
            "Cell {:?} in {:?} is void",
            id,
            neighbours.root()
        );
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

        if step.contains(StepMode::PHASE_CHANGE) {
            cell.set_phase();
        };
        cell.flags.remove(CellFlags::MOVE_ALL);

        if step.contains(StepMode::BROWNIAN)
            && cell
                .flags
                .intersects(CellFlags::IS_LIQUID | CellFlags::IS_GAS)
        // dont do brownian motion for solids
        {
            for i in 0..6 {
                let (target, flag) = match i {
                    0 => (id.up(), CellFlags::MOVE_UP),
                    1 => (id.down(), CellFlags::MOVE_DOWN),
                    2 => (id.left(), CellFlags::MOVE_LEFT),
                    3 => (id.right(), CellFlags::MOVE_RIGHT),
                    4 => (id.forward(), CellFlags::MOVE_FORWARD),
                    5 => (id.backward(), CellFlags::MOVE_BACK),
                    _ => unreachable!(),
                };
                let target = neighbours.get(target);
                // break as soon as we have found a move
                if cell.is_liquid() {
                    if i == 0 // if moving up
                        && target.is_liquid() // check target is liquid --- dont want liquids floating on gases
                        && target.properties().density >= cell.properties().density
                    // check density --- can only swap if decity is same or lower then target
                    {
                        if cell.get_block() == Blocks::Water && target.get_block() == Blocks::Air {
                            println!("Water moving up in Air");
                        }
                        cell.flags.insert(CellFlags::MOVE_UP);
                        break;
                    } else if i != 0 && target.can_move()
                    // check if target is liquid or gas
                    {
                        cell.flags.insert(flag);
                        break;
                    }
                } else {
                    if i == 1 // if moving down
                && target.is_gas() // check target is gas --- dont want gases sinking in liquids
                    && target.properties().density <= cell.properties().density
                    {
                        if cell.get_block() == Blocks::Air && target.get_block() == Blocks::Water {
                            println!("Air moving Down in Water");
                        }
                        cell.flags.insert(CellFlags::MOVE_DOWN);
                        break;
                    } else if i != 1 && target.can_move()
                    // check if target is liquid or gas
                    {
                        cell.flags.insert(flag);
                        break;
                    }
                }
            }
        } else if step.contains(StepMode::GRAVITY) {
            cell.flags.remove(CellFlags::MOVE_ALL);
            let block = cell.get_block();
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
                    cell.flags.set(CellFlags::MOVE_DOWN, true);
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
                    cell.flags.set(CellFlags::MOVE_UP, true);
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
