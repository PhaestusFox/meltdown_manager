mod cells;
mod consts;
mod diagnostics;
mod logic;

pub use cells::{BlockProperties, CellData};
pub use consts::*;
pub use diagnostics::{AutomitaDiagnosticChunk, CellMode};

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(diagnostics::plugin)
        .add_systems(FixedUpdate, logic::step_system)
        .add_systems(FixedPreUpdate, logic::set_prev);
}
