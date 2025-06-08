use bevy::prelude::*;
use bevy_console::{ConsoleCommand, clap::Parser, reply, reply_failed};

use crate::{
    player::Player,
    voxels::{CHUNK_SIZE, ChunkId, ChunkManager},
};

/// Highlight command that allows one to control the gizmo highlighting of the chunk you're currently in
#[derive(Parser, ConsoleCommand, Debug)]
#[command(name = "highlight")]
pub enum ChunkHighlightCommand {
    Toggle,
    On,
    Off,
    Select {
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
    },
    Deselect {
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
    },
    Player,
}

pub fn init(app: &mut App) {
    app.init_resource::<ChunkHighlightState>().add_systems(
        Update,
        apply_chunk_highlight.run_if(resource_changed::<ChunkHighlightState>),
    );
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
    player: Single<&Transform, With<Player>>,
) {
    if let Some(Ok(c)) = log.take() {
        match c {
            ChunkHighlightCommand::Toggle => chunk_highlight_state.state.toggle(),
            ChunkHighlightCommand::On => {
                chunk_highlight_state.state = HighlightState::All;
            }
            ChunkHighlightCommand::Off => {
                chunk_highlight_state.state = HighlightState::Off;
            }
            ChunkHighlightCommand::Select { x, y, z } => {
                let to = ChunkId::new(x, y, z);
                if manager.get_chunk(&to).is_some() {
                    chunk_highlight_state.state = HighlightState::Select(to);
                } else {
                    reply_failed!(log, "Chunk not found: {:?}", to);
                }
            }
            ChunkHighlightCommand::Deselect { x, y, z } => {
                let to = ChunkId::new(x, y, z);
                if manager.get_chunk(&to).is_some() {
                    chunk_highlight_state.state = HighlightState::Deselect(to);
                } else {
                    reply_failed!(log, "Chunk not found: {:?}", to);
                }
            }
            ChunkHighlightCommand::Player => {
                let chunk = ChunkId::from_translation(player.translation);
                if manager.get_chunk(&chunk).is_some() {
                    chunk_highlight_state.state = HighlightState::Select(chunk);
                    reply!(log, "Highlighting chunk: {:?}", chunk);
                } else {
                    reply_failed!(log, "Player is not in a chunk.");
                }
            }
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
            if let Some(chunk) = chunk_manager.get_chunk(&chunk_id) {
                let mut g = GizmoAsset::new();
                g.cuboid(
                    Transform::from_scale(Vec3::splat(CHUNK_SIZE as f32))
                        .with_translation(Vec3::splat((CHUNK_SIZE / 2) as f32)),
                    Color::linear_rgb(0.1, 0.1, 1.),
                );
                commands.entity(chunk).insert(Gizmo {
                    handle: gizmos.add(g),
                    ..Default::default()
                });
            };
        }
        HighlightState::Deselect(chunk_id) => {
            if let Some(entity) = chunk_manager.get_chunk(&chunk_id) {
                commands.entity(entity).remove::<Gizmo>();
            }
        }
        HighlightState::All => {
            for entity in &all {
                let mut g = GizmoAsset::new();
                g.cuboid(
                    Transform::from_scale(Vec3::splat(CHUNK_SIZE as f32))
                        .with_translation(Vec3::splat((CHUNK_SIZE / 2) as f32)),
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
