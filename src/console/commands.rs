use bevy::{prelude::*, scene::ron::de};
use bevy_console::{ConsoleCommand, clap::Parser, reply, reply_failed};

use crate::voxels::{ChunkId, ChunkManager};

/// Highlight command that allows one to control the gizmo highlighting of the chunk you're currently in
#[derive(Parser, ConsoleCommand)]
#[command(name = "chunk_highlight")]
pub struct ChunkHighlightCommand {
    state: Option<String>,
}

#[derive(Default)]
pub enum HighlightState {
    #[default]
    Off,
    Select(ChunkId),
    Deselect(ChunkId),
    All,
}

impl HighlightState {
    pub fn toggle(&mut self) {
        match self {
            HighlightState::Off => *self = HighlightState::All,
            HighlightState::All => *self = HighlightState::Off,
            HighlightState::Select(_) => *self = HighlightState::Off,
            HighlightState::Deselect(_) => *self = HighlightState::Off,
        }
    }
}

// Our resource to control the chunk highlighting
#[derive(Resource, Default)]
pub struct ChunkHighlightState {
    pub state: HighlightState,
}

pub fn chunk_highlight_command(
    mut log: ConsoleCommand<ChunkHighlightCommand>,
    mut chunk_highlight_state: ResMut<ChunkHighlightState>,
    manager: Res<ChunkManager>,
) {
    if let Some(Ok(command)) = log.take() {
        match command.state.as_deref() {
            Some("on") | Some("true") => {
                chunk_highlight_state.state = HighlightState::All;
            }
            Some("off") | Some("false") => {
                chunk_highlight_state.state = HighlightState::Off;
            }
            Some(other) => {
                if other.starts_with("select") {
                    println!("Selecting chunk: {}", &other[7..]);
                } else if other.starts_with("deselect") {
                    println!("Deselecting chunk: {}", &other[9..]);
                } else {
                    println!("toggling chunk {}", other);
                }
            }
            None => chunk_highlight_state.state.toggle(),
        }
    }
}

// Example system that would use the ChunkHighlightState
pub fn apply_chunk_highlight(
    chunk_highlight_state: Res<ChunkHighlightState>,
    mut gizmos: ResMut<Assets<GizmoAsset>>,
    chunk_manager: Res<ChunkManager>,
    mut commands: Commands,
    old: Query<Entity, (With<ChunkId>, With<Gizmo>)>,
    all: Query<Entity, With<ChunkId>>,
) {
    match chunk_highlight_state.state {
        HighlightState::Off => {
            for entity in old.iter() {
                // Remove the gizmo from the entity
                commands.entity(entity).remove::<Gizmo>();
            }
        }
        HighlightState::Select(chunk_id) => {
            // Logic to highlight the specific chunk
            info!("Highlighting chunk: {:?}", chunk_id);
            let Some(chunk) = chunk_manager.get_chunk(&chunk_id) else {
                // reply_failed!("Chunk not found: {:?}", chunk_id);
                return;
            };
            let mut g = GizmoAsset::new();
            g.cuboid(
                Transform::from_scale(Vec3::splat(30.)),
                Color::linear_rgb(0.1, 0.1, 1.),
            );
            commands.entity(chunk).insert(Gizmo {
                handle: gizmos.add(g),
                ..Default::default()
            });
        }
        HighlightState::Deselect(chunk_id) => {
            // Logic to deselect the specific chunk
            info!("Deselecting chunk: {:?}", chunk_id);
            if let Some(entity) = chunk_manager.get_chunk(&chunk_id) {
                commands.entity(entity).remove::<Gizmo>();
            } else {
                // reply_failed!("Chunk not found for deselection: {:?}", chunk_id);
            }
        }
        HighlightState::All => {
            // Logic to highlight all chunks
            info!("Highlighting all chunks");
            for entity in &all {
                let mut g = GizmoAsset::new();
                g.cuboid(
                    Transform::from_scale(Vec3::splat(30.)).with_translation(Vec3::splat(15.)),
                    Color::linear_rgb(0.1, 1., 0.1),
                );
                commands.entity(entity).insert(Gizmo {
                    handle: gizmos.add(g),
                    ..Default::default()
                });
            }
        }
    }
}
