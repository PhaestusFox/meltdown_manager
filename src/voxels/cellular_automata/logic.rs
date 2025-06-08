const CHUNK_SIZE: i32 = crate::voxels::map::CHUNK_SIZE;
const SUM_DIVISOR: FixedNum = FixedNum::lit("6.0");

use crate::voxels::{
    block::BlockType,
    cellular_automata::{FixedNum, cells::CellFlags},
};

use super::*;

pub fn step<'a>(chunk: ChunkIter<'a>, neighbours: ChunkGared<'a>, tick: u64) {
    step_diag(chunk, neighbours, tick);
}

use fastrand::Rng;

pub fn step_diag<'a>(chunk: ChunkIter<'a>, neighbours: ChunkGared<'a>, tick: u64) -> CellData {
    let mut max = CellData::MIN;
    let mut rng = Rng::new();
    for (id, data) in chunk {
        let Some(mut cell) = neighbours.get(id) else {
            #[cfg(debug_assertions)]
            warn!("Cell {:?} in {:?} is out of bounds?", id, neighbours.root());
            continue; // skip if cell is void
        };
        #[cfg(debug_assertions)]
        {
            debug_assert!(
                cell.get_block_type() != BlockType::Void,
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
            let g = cell.lookup_g(neighbour_data.get_block_type());
            let heat_transfer = g * delta_t;
            cell.energy += heat_transfer;
        }
        if cell.get_block_type() == BlockType::Uranium {
            cell.energy += FixedNum::lit("10."); // hack to add uranium heat without changing my meta code
        }

        cell.flags.remove(CellFlags::MOVE_ALL);
        match tick & 0b11 {
            0b00 => {
                cell.set_phase();
            }
            0b01 => {
                cell.flags |= check_gravity(id, &cell, &neighbours);
            }
            0b10 => {
                cell.set_phase();
                if cell.can_move() {
                    match (tick >> 2) & 0b111 {
                        0b000 => {
                            cell.flags |= do_brownian(id, id.x & 1 == 1, true, &cell, &neighbours);
                        }
                        0b010 => {
                            cell.flags |= do_brownian(id, id.z & 1 == 1, false, &cell, &neighbours);
                        }
                        0b100 => {
                            cell.flags |= do_brownian(id, id.x & 1 == 0, true, &cell, &neighbours);
                        }
                        0b110 => {
                            cell.flags |= do_brownian(id, id.z & 1 == 0, false, &cell, &neighbours);
                        }
                        _ => {
                            rng.seed(tick as u64 ^ id.y as u64);
                            let odd = rng.i32(0..=1);
                            if rng.bool() {
                                cell.flags |=
                                    do_brownian(id, id.x & 1 == odd, true, &cell, &neighbours);
                            } else {
                                cell.flags |=
                                    do_brownian(id, id.z & 1 == odd, false, &cell, &neighbours);
                            }
                        }
                    }
                }
            }
            0b11 => {
                cell.flags |= check_gravity(id, &cell, &neighbours);
            }
            _ => unreachable!(),
        }
        #[cfg(debug_assertions)]
        max.max(&cell);
        *data = cell;
    }
    max
}

fn check_gravity(id: CellId, cell: &CellData, neighbours: &ChunkGared) -> CellFlags {
    if !cell.can_move() {
        // check if the cell can move
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

fn do_brownian(
    id: CellId,
    odd: bool,
    x: bool,
    cell: &CellData,
    neighbours: &ChunkGared,
) -> CellFlags {
    let (target, will_move) = match (x, odd) {
        (true, true) => (id.left(), CellFlags::MOVE_LEFT),
        (true, false) => (id.right(), CellFlags::MOVE_RIGHT),
        (false, true) => (id.forward(), CellFlags::MOVE_FORWARD),
        (false, false) => (id.backward(), CellFlags::MOVE_BACK),
    };
    let Some(other) = neighbours.get(target) else {
        return CellFlags::empty();
    };
    if !other.can_move() {
        return CellFlags::empty();
    }
    if cell.is_gas() && other.is_liquid() || cell.is_liquid() && other.is_gas() {
        will_move
    } else {
        // gases cannot move into liquids and vice versa
        CellFlags::empty()
    }
}

bitflags::bitflags! {
    #[derive(Default, Clone, Copy, Debug)]
    pub struct StepMode: u64 {
        const PHASE_CHANGE = 1;
        const GRAVITY = 1<<1;
        const BROWNIAN = 1<<2;
        const PHYSICS = StepMode::GRAVITY.bits() | StepMode::BROWNIAN.bits();
    }
}

impl std::fmt::Display for StepMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}", self))
    }
}

pub fn is_step(step: StepMode) -> impl Fn(Res<VoxelTick>) -> bool {
    move |tick: Res<VoxelTick>| StepMode::from_bits_retain(tick.get()).intersects(step)
}

#[cfg(test)]
mod test;
