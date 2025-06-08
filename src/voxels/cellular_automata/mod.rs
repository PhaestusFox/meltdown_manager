mod batching;
mod cells;
mod consts;
mod logic;
mod util;

use crate::voxels::VoidNeighbours;
pub use crate::voxels::map::ChunkData;
pub use batching::{
    BatchingStep, VoxelStep, can_fuck_with_next_step, can_modify_next_step, can_modify_world,
};
use bevy::prelude::*;
pub use cells::{CellData, CellFlags};
pub use consts::*;
pub use logic::{StepMode, step};
pub use util::*;

mod debugging;

pub fn plugin(app: &mut App) {
    app.add_plugins(batching::plugin);
    #[cfg(debug_assertions)]
    app.add_plugins(debugging::plugin);
    app.init_resource::<VoxelTick>()
        .init_resource::<TargetTick>()
        .register_type::<VoxelTick>()
        .register_type::<TargetTick>();
    app.init_resource::<VoidNeighbours>();
}

#[derive(Resource, Default, Reflect)]
pub struct VoxelTick(u64);

#[derive(Resource, Default, Reflect)]
pub struct TargetTick(u64);

impl VoxelTick {
    pub fn new(tick: u64) -> Self {
        Self(tick)
    }

    fn inc(&mut self) {
        self.0 += 1;
    }

    pub fn get(&self) -> u64 {
        self.0
    }

    fn mode(&self) -> StepMode {
        StepMode::from_bits_retain(self.0)
    }
}

impl TargetTick {
    pub fn new(tick: u64) -> Self {
        Self(tick)
    }
    pub fn get(&self) -> u64 {
        self.0
    }

    pub fn inc(&mut self) {
        self.0 += 1;
    }

    pub fn set(&mut self, to: u64) {
        warn!("Target Time was set; only do this for testing at the moment!");
        self.0 = to;
    }
}
