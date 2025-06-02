use bevy::prelude::*;

use crate::voxels::Chunk;

use super::{CellData, PrevioseStep};

pub fn step_system(
    chunks: Query<(&Chunk<CellData>, &PrevioseStep)>
) {
    
}
