use bevy::prelude::*;
use bevy_console::reply;
use bevy_console::{ConsoleCommand, clap::Parser, reply_failed};

use crate::player::Player;
use crate::utils::BlockIter;
use crate::voxels::cellular_automata::Cells;
use crate::voxels::map::ChunkData;
use crate::voxels::{ChunkId, ChunkManager};

/// Highlight command that allows one to control the gizmo highlighting of the chunk you're currently in
#[derive(Parser, ConsoleCommand, Debug)]
#[command(name = "save")]
pub enum SaveCommand {
    Chunk {
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
        file: Option<String>,
    },
    World {
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
        file: Option<String>,
    },
}

/// Highlight command that allows one to control the gizmo highlighting of the chunk you're currently in
#[derive(Parser, ConsoleCommand, Debug)]
#[command(name = "save")]
pub enum LoadCommand {
    Chunk {
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
        file: Option<String>,
    },
    World {
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
        file: Option<String>,
    },
}

// Our resource to control the chunk highlighting
#[derive(Resource, Default)]
pub struct RedrawState {
    target: Option<Entity>,
}

pub fn init(app: &mut App) {
    app.init_resource::<RedrawState>()
        .add_systems(Update, apply_redraw.run_if(resource_changed::<RedrawState>));
}

pub fn chunk_save_command(
    mut log: ConsoleCommand<SaveCommand>,
    mut state: ResMut<RedrawState>,
    manager: Res<ChunkManager>,
    player: Single<&Transform, With<Player>>,
) {
    if let Some(Ok(c)) = log.take() {
        // match c {
        // }
        return;
    }
}

// Example system that would use the ChunkHighlightState
pub fn apply_redraw(
    state: Res<RedrawState>,
    mut blocks: Query<(&mut ChunkData, &Cells)>,
    mut mesher: ResMut<phoxels::ChunkMesher>,
) {
    if let Some(target) = state.target {
        let Ok((mut chunk_data, cells)) = blocks.get_mut(target) else {
            return;
        };
        for (x, y, z) in BlockIter::new() {
            let block = cells.get_cell(x, y, z).get_block_type();
            chunk_data.set_block(x as u32, y as u32, z as u32, block);
        }
        mesher.add_to_queue(target);
    }
}
