use bevy::prelude::*;
use bevy_console::reply;
use bevy_console::{ConsoleCommand, clap::Parser, reply_failed};

use crate::voxels::cellular_automata::Cells;
use crate::voxels::{ChunkId, ChunkManager};

use base64::prelude::*;

/// Highlight command that allows one to control the gizmo highlighting of the chunk you're currently in
#[derive(Parser, ConsoleCommand, Debug)]
#[command(name = "export")]
pub enum Export {
    Chunk {
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
    },
    World,
    Saved {
        #[arg(value_name = "FILE", default_value = "")]
        file: String,
    },
}

/// Highlight command that allows one to control the gizmo highlighting of the chunk you're currently in
#[derive(Parser, ConsoleCommand, Debug)]
#[command(name = "import")]
pub enum Import {
    Chunk {
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
        #[arg(value_name = "data")]
        data: String,
    },
    World {
        #[arg(value_name = "data")]
        data: String,
    },
    Saved {
        #[arg(value_name = "FILE")]
        file: String,
        #[arg(value_name = "data")]
        data: String,
    },
}

pub fn chunk_export_command(
    mut log: ConsoleCommand<Export>,
    manager: Res<ChunkManager>,
    chunks: Query<&Cells>,
    store: Res<bevy_pkv::PkvStore>,
    tick: Res<crate::voxels::cellular_automata::VoxelTick>,
) {
    if let Some(Ok(c)) = log.take() {
        match c {
            Export::Chunk { x, y, z } => {
                let chunk_id = ChunkId::new(x, y, z);
                let data = match manager.save_compressed_chunk(chunk_id, &chunks) {
                    Ok(data) => data,
                    Err(e) => {
                        reply_failed!(log, "Failed to save chunk: {}", e);
                        return;
                    }
                };
                reply!(
                    log,
                    "Chunk ({}, {}, {}): {}",
                    x,
                    y,
                    z,
                    BASE64_STANDARD_NO_PAD.encode(&data)
                );
            }
            Export::World => {
                let data = match manager.save_compressed_world(&chunks, tick.get()) {
                    Ok(data) => data,
                    Err(e) => {
                        reply_failed!(log, "Failed to save world: {}", e);
                        return;
                    }
                };
                reply!(log, "World: {}", BASE64_STANDARD_NO_PAD.encode(&data));
            }
            Export::Saved { file } => {
                let path = if file.is_empty() { "auto" } else { &file };
                let data = match store.get::<Vec<u8>>(path) {
                    Ok(data) => data,
                    Err(e) => {
                        reply_failed!(log, "Failed to retrieve data from store: {}", e);
                        return;
                    }
                };
                reply!(
                    log,
                    "Saved data from file '{}': {}",
                    path,
                    BASE64_STANDARD_NO_PAD.encode(&data)
                );
            }
        }
    }
}

pub fn chunk_import_command(
    mut log: ConsoleCommand<Import>,
    manager: Res<ChunkManager>,
    mut store: ResMut<bevy_pkv::PkvStore>,
    mut commands: Commands,
) {
    if let Some(Ok(c)) = log.take() {
        match c {
            Import::Chunk { x, y, z, data } => {
                let chunk_id = ChunkId::new(x, y, z);
                let decoded_data = match BASE64_STANDARD_NO_PAD.decode(data) {
                    Ok(data) => data,
                    Err(e) => {
                        reply_failed!(log, "Failed to decode data: {}", e);
                        return;
                    }
                };
                if decoded_data.starts_with(b"PhoxK") {
                    if let Err(e) =
                        manager.load_compressed_chunk(chunk_id, &decoded_data, &mut commands)
                    {
                        reply_failed!(log, "Failed to load chunk: {}", e);
                    } else {
                        reply!(log, "Chunk ({}, {}, {}) loaded successfully.", x, y, z);
                    }
                } else if let Err(e) = manager.load_chunk(chunk_id, &decoded_data, &mut commands) {
                    reply_failed!(log, "Failed to load chunk: {}", e);
                } else {
                    reply!(log, "Chunk ({}, {}, {}) loaded successfully.", x, y, z);
                }
            }
            Import::World { data } => {
                let decoded_data = match BASE64_STANDARD_NO_PAD.decode(data) {
                    Ok(data) => data,
                    Err(e) => {
                        reply_failed!(log, "Failed to decode data: {}", e);
                        return;
                    }
                };
                if decoded_data.starts_with(b"PhoxW") {
                    if let Err(e) = manager.load_world(&decoded_data, &mut commands) {
                        reply_failed!(log, "Failed to load world: {}", e);
                    } else {
                        reply!(log, "World loaded successfully.");
                    }
                } else if let Err(e) = manager.load_compressed_world(&decoded_data, &mut commands) {
                    reply_failed!(log, "Failed to load world: {}", e);
                } else {
                    reply!(log, "World loaded successfully.");
                }
            }
            Import::Saved { file, data } => {
                let path = if file.is_empty() { "auto" } else { &file };
                let decoded_data = match BASE64_STANDARD_NO_PAD.decode(data) {
                    Ok(data) => data,
                    Err(e) => {
                        reply_failed!(log, "Failed to decode data: {}", e);
                        return;
                    }
                };
                if decoded_data.starts_with(b"PhoxK") || decoded_data.starts_with(b"PhoxM") {
                    let w = if decoded_data.starts_with(b"PhoxK") {
                        "world"
                    } else {
                        "chunk"
                    };
                    reply_failed!(
                        log,
                        "Data appears to be compressed format. It can only be loaded directly as a {}, used as a saved file.",
                        w
                    );
                    return;
                }
                if let Err(e) = store.set(path, &decoded_data) {
                    reply_failed!(log, "Failed to save chunk data to store: {}", e);
                } else {
                    reply!(log, "Data saved to file '{}'.", path);
                }
            }
        }
    }
}
