mod batching;
mod cells;
mod consts;
mod logic;
mod util;

pub use crate::voxels::map::ChunkData;
pub use batching::{BatchingStep, can_fuck_with_next_step, can_modify_next_step, can_modify_world};
use bevy::prelude::*;
pub use cells::{BlockProperties, CellData, CellFlags};
pub use consts::*;
pub use logic::step;
pub use util::*;

mod debugging;

pub fn plugin(app: &mut App) {
    app.add_plugins(batching::plugin);
    #[cfg(debug_assertions)]
    app.add_plugins(debugging::plugin);
}
