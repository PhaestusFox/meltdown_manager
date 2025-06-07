use bevy::prelude::*;
use bevy_console::{ConsoleCommand, clap::Parser, reply, reply_failed};

/// Highlight command that allows one to control the gizmo highlighting of the chunk you're currently in
#[derive(Parser, ConsoleCommand)]
#[command(name = "chunk_highlight")]
pub struct ChunkHighlightCommand {
    state: Option<String>,
}

// Our resource to control the chunk highlighting
#[derive(Resource, Default)]
pub struct ChunkHighlightState {
    pub state: bool,
}

pub fn chunk_highlight_command(
    mut log: ConsoleCommand<ChunkHighlightCommand>,
    mut chunk_highlight_state: ResMut<ChunkHighlightState>,
) {
    if let Some(Ok(command)) = log.take() {
        let new_state = match command.state.as_deref() {
            Some("true") | Some("1") => Some(true),
            Some("false") | Some("0") => Some(false),
            Some(other) => {
                reply_failed!(
                    log,
                    "Invalid value for state: '{}'. Please use 'true', 'false', '1', or '0'.",
                    other
                );
                return;
            }
            None => None,
        };

        if let Some(state_value) = new_state {
            chunk_highlight_state.state = state_value;
            log.reply_ok(format!(
                "Turned chunk highlighting state to {:?}",
                chunk_highlight_state.state
            ));
        } else {
            log.reply_ok(format!(
                "Chunk highlighting is currently: {:?}",
                chunk_highlight_state.state
            ));
        }
    }
}

// Example system that would use the ChunkHighlightState
fn apply_chunk_highlight(chunk_highlight_state: Res<ChunkHighlightState>) {
    if chunk_highlight_state.state {
        println!("Applying chunk highlighting...");
    } else {
    }
}
