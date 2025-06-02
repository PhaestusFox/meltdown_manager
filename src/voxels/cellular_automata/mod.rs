const K_AT_20C: FixedNum = FixedNum::lit("293.15");
const ATM_1: FixedNum = FixedNum::lit("101.325");
const STD_CHARGE: FixedNum = FixedNum::lit("0");
mod diagnostics;
pub type FixedNum = fixed::types::I22F10;
use super::voxel_chunk::Chunk;

#[derive(Component)]
pub struct PrevioseStep(Chunk<CellData>);

pub use cells::{BlockProperties, CellData};
pub use diagnostics::{AutomitaDiagnosticChunk, CellMode};

mod cells;
mod logic;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(diagnostics::plugin)
        .add_systems(FixedUpdate, logic::step_system)
        .add_systems(FixedPreUpdate, logic::set_prev);
}
