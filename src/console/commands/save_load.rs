use bevy::prelude::*;
use bevy_console::{ConsoleCommand, clap::Parser, reply_failed};

use crate::voxels::cellular_automata::Cells;
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
        #[arg(value_name = "FILE", default_value = "")]
        file: String,
    },
    World {
        #[arg(value_name = "FILE", default_value = "")]
        file: String,
    },
}

/// Highlight command that allows one to control the gizmo highlighting of the chunk you're currently in
#[derive(Parser, ConsoleCommand, Debug)]
#[command(name = "load")]
pub enum LoadCommand {
    Chunk {
        #[arg(value_name = "FILE")]
        file: String,
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
    },
    World {
        #[arg(value_name = "FILE")]
        file: String,
    },
}

// Our resource to control the chunk highlighting
#[derive(Resource, Default)]
pub struct RedrawState {
    target: Option<Entity>,
}

pub fn init(app: &mut App) {}

pub fn chunk_save_command(
    mut log: ConsoleCommand<SaveCommand>,
    manager: Res<ChunkManager>,
    chunks: Query<&Cells>,
    mut store: ResMut<bevy_pkv::PkvStore>,
    tick: Res<crate::voxels::cellular_automata::VoxelTick>,
) {
    if let Some(Ok(c)) = log.take() {
        match c {
            SaveCommand::Chunk { x, y, z, file } => {
                let path = if file.is_empty() {
                    format!("chunk_{}_{}_{}", x, y, z)
                } else {
                    file
                };
                let chunk_id = ChunkId::new(x, y, z);
                let data = match manager.save_chunk(chunk_id, &chunks) {
                    Ok(d) => d,
                    Err(e) => {
                        reply_failed!(log, "Failed to save chunk: {}", e);
                        return;
                    }
                };
                if let Err(e) = store.set(path, &data) {
                    reply_failed!(log, "Failed to save chunk data to store: {}", e);
                }
            }
            SaveCommand::World { file } => {
                let path = if file.is_empty() { "auto" } else { &file };

                let data = match manager.save_world(&chunks, tick.get()) {
                    Ok(d) => d,
                    Err(e) => {
                        reply_failed!(log, "Failed to save world: {}", e);
                        return;
                    }
                };
                if let Err(e) = store.set(path, &data) {
                    reply_failed!(log, "Failed to save world data to store: {}", e);
                }
            }
        }
    }
}

pub fn chunk_load_command(
    mut log: ConsoleCommand<LoadCommand>,
    manager: Res<ChunkManager>,
    store: Res<bevy_pkv::PkvStore>,
    mut commands: Commands,
) {
    if let Some(Ok(c)) = log.take() {
        match c {
            LoadCommand::Chunk { x, y, z, file } => {
                let Ok(data) = store.get::<Vec<u8>>(&file) else {
                    reply_failed!(log, "No data found for file: {}", file);
                    return;
                };
                let chunk_id = ChunkId::new(x, y, z);
                if let Err(e) = manager.load_chunk(chunk_id, &data, &mut commands) {
                    reply_failed!(log, "Failed to save chunk: {}", e);
                }
            }
            LoadCommand::World { file } => {
                let path = if file.is_empty() { "auto" } else { &file };

                let Ok(data) = store.get::<Vec<u8>>(&path) else {
                    reply_failed!(log, "No data found for file: {}", file);
                    return;
                };
                if let Err(e) = manager.load_world(&data, &mut commands) {
                    reply_failed!(log, "Failed to save chunk: {}", e);
                }
            }
        }
    }
}
