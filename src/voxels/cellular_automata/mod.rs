mod batching;
mod cells;
mod consts;
mod logic;
mod util;

pub use crate::voxels::map::ChunkData;
pub use batching::set_prev;
use bevy::prelude::*;
pub use cells::{BlockProperties, CellData, CellFlags};
pub use consts::*;
pub use logic::step;
pub use util::*;

pub fn plugin(app: &mut App) {
    app.add_systems(FixedUpdate, batching::step_system)
        .add_systems(FixedPostUpdate, set_prev);
}
