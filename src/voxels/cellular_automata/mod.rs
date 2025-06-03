mod batching;
mod cells;
mod consts;
mod diagnostics;
mod logic;
mod util;

pub use cells::{BlockProperties, CellData};
pub use consts::*;
pub use diagnostics::{AutomitaDiagnosticChunk, CellMode};
pub use util::*;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(diagnostics::plugin)
        .add_systems(FixedUpdate, batching::step_system)
        .add_systems(FixedPostUpdate, batching::set_prev);
}
