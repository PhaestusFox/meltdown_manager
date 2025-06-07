use bevy::{prelude::*, scene::ron::de};
use bevy_console::{ConsoleCommand, clap::Parser, reply, reply_failed};

use crate::voxels::ChunkId;

/// Highlight command that allows one to control the gizmo highlighting of the chunk you're currently in
#[derive(Parser, ConsoleCommand)]
#[command(name = "chunk_highlight")]
pub struct ChunkHighlightCommand {
    state: Option<String>,
}

#[derive(Default)]
enum HighlightState {
    #[default]
    Off,
    Select(ChunkId),
    All,
}

// Our resource to control the chunk highlighting
#[derive(Resource, Default)]
pub struct ChunkHighlightState {
    pub state: HighlightState,
}

pub fn chunk_highlight_command(
    mut log: ConsoleCommand<ChunkHighlightCommand>,
    mut chunk_highlight_state: ResMut<ChunkHighlightState>,
) {
    if let Some(Ok(command)) = log.take() {
        println!("Command received: {:?}", command.state);
    }
}

// Example system that would use the ChunkHighlightState
pub fn apply_chunk_highlight(chunk_highlight_state: Res<ChunkHighlightState>) {
    match chunk_highlight_state.state {
        HighlightState::Off => {
            // Logic to turn off highlighting
        }
        HighlightState::Select(chunk_id) => {
            // Logic to highlight the specific chunk
            info!("Highlighting chunk: {:?}", chunk_id);
        }
        HighlightState::All => {
            // Logic to highlight all chunks
            info!("Highlighting all chunks");
        }
    }
}
