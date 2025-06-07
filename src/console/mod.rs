use bevy::prelude::*;
use bevy_console::*;

pub(crate) mod commands;

pub fn plugin(app: &mut App) {
    app.add_plugins(bevy_console::ConsolePlugin);
    app.add_console_command::<commands::ChunkHighlightCommand, _>(commands::apply_chunk_highlight);
}
