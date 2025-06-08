use bevy::prelude::*;
use bevy_console::{ConsoleCommand, clap::Parser, reply, reply_failed};

use crate::console::AxisPointer;
use crate::voxels::{CHUNK_SIZE, ChunkId, ChunkManager};

/// Highlight command that allows one to control the gizmo highlighting of the chunk you're currently in
#[derive(Parser, ConsoleCommand, Debug)]
#[command(name = "neighbors")]
pub enum NeighborsCommand {
    Off,
    Select {
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
    },
    Targets {
        #[arg(value_name = "X", default_value = "0")]
        x: i32,
        #[arg(value_name = "Y", default_value = "0")]
        y: i32,
        #[arg(value_name = "Z", default_value = "0")]
        z: i32,
    },
}

#[derive(Default)]
pub enum HighlightState {
    #[default]
    Off,
    Select(ChunkId),
    Targets(ChunkId),
}

// Our resource to control the chunk highlighting
#[derive(Resource, Default)]
pub struct NeighborsState {
    root: Option<Entity>,
    pub mode: HighlightState,
}

pub fn init(app: &mut App) {
    app.init_resource::<NeighborsState>().add_systems(
        Update,
        apply_neighbours_highlight.run_if(resource_changed::<NeighborsState>),
    );
}

pub fn chunk_neighbore_command(
    mut log: ConsoleCommand<NeighborsCommand>,
    mut chunk_highlight_state: ResMut<NeighborsState>,
    manager: Res<ChunkManager>,
) {
    if let Some(Ok(c)) = log.take() {
        match c {
            NeighborsCommand::Off => {
                chunk_highlight_state.mode = HighlightState::Off;
                reply!(log, "Neighbors highlighting turned off.");
            }
            NeighborsCommand::Select { x, y, z } => {
                let chunk_id = ChunkId::new(x, y, z);
                if let Some(chunk_entity) = manager.get_chunk(&chunk_id) {
                    chunk_highlight_state.mode = HighlightState::Select(chunk_id);
                    chunk_highlight_state.root = Some(chunk_entity);
                    reply!(
                        log,
                        "Neighbors highlighting turned on for chunk: {:?}",
                        chunk_id
                    );
                } else {
                    reply_failed!(log, "Chunk {:?} does not exist.", chunk_id);
                }
            }
            NeighborsCommand::Targets { x, y, z } => {
                let chunk_id = ChunkId::new(x, y, z);
                if let Some(chunk_entity) = manager.get_chunk(&chunk_id) {
                    chunk_highlight_state.mode = HighlightState::Targets(chunk_id);
                    chunk_highlight_state.root = Some(chunk_entity);
                    reply!(
                        log,
                        "Neighbors highlighting turned on for chunk: {:?}",
                        chunk_id
                    );
                } else {
                    reply_failed!(log, "Chunk {:?} does not exist.", chunk_id);
                }
            }
        }
    }
}

use crate::voxels::Neighbours;

// Example system that would use the ChunkHighlightState
pub fn apply_neighbours_highlight(
    state: Res<NeighborsState>,
    mut gizmos: ResMut<Assets<GizmoAsset>>,
    chunk_manager: Res<ChunkManager>,
    mut commands: Commands,
    old: Query<Entity, With<Gizmo>>,
    all: Query<(Entity, &Neighbours), With<ChunkId>>,
) {
    match state.mode {
        HighlightState::Off => {
            if let Some(root) = state.root {
                commands.entity(root).remove::<AxisPointer>();
            }
            for entity in old.iter() {
                // Remove the gizmo from the entity
                commands.entity(entity).remove::<Gizmo>();
            }
        }
        HighlightState::Select(chunk_id) => {
            let Some(chunk_entity) = chunk_manager.get_chunk(&chunk_id) else {
                error!("Chunk {:?} not found in manager", chunk_id);
                return;
            };
            let Some((_, neighbors)) = all.get(chunk_entity).ok() else {
                error!("Chunk {:?} does not have neighbors", chunk_id);
                return;
            };
            commands.entity(chunk_entity).insert(AxisPointer::new());
            for (neighbor, target) in neighbors.iter() {
                let color = match neighbor {
                    crate::voxels::NeighbourDirection::Up => Color::linear_rgb(0., 1., 0.), // Green for Up + y
                    crate::voxels::NeighbourDirection::Down => Color::linear_rgb(1., 1., 0.), // Yellow for Down -Y
                    crate::voxels::NeighbourDirection::Left => Color::linear_rgb(1., 0., 1.), // Magenta for Left -x
                    crate::voxels::NeighbourDirection::Right => Color::linear_rgb(1., 0., 0.), // Red for Right +x
                    crate::voxels::NeighbourDirection::Front => Color::linear_rgb(0., 0., 1.), // Blue for Front +z
                    crate::voxels::NeighbourDirection::Back => Color::linear_rgb(0., 1., 1.), // Cyan for Back -z
                };
                let mut g = GizmoAsset::new();
                g.cuboid(
                    Transform::from_scale(Vec3::splat(CHUNK_SIZE as f32))
                        .with_translation(Vec3::splat(CHUNK_SIZE as f32 / 2.)),
                    color,
                );
                commands.entity(target).insert(Gizmo {
                    handle: gizmos.add(g),
                    ..Default::default()
                });
            }
        }
        HighlightState::Targets(chunk_id) => {
            let Some(chunk_entity) = chunk_manager.get_chunk(&chunk_id) else {
                error!("Chunk {:?} not found in manager", chunk_id);
                return;
            };
            commands.entity(chunk_entity).insert(AxisPointer::new());
            for (entity, neighbors) in all {
                for (neighbor, target) in neighbors.iter() {
                    let color = match neighbor {
                        crate::voxels::NeighbourDirection::Up => Color::linear_rgb(0., 1., 0.), // Green for Up + y
                        crate::voxels::NeighbourDirection::Down => Color::linear_rgb(1., 1., 0.), // Yellow for Down -Y
                        crate::voxels::NeighbourDirection::Left => Color::linear_rgb(1., 0., 1.), // Magenta for Left -x
                        crate::voxels::NeighbourDirection::Right => Color::linear_rgb(1., 0., 0.), // Red for Right +x
                        crate::voxels::NeighbourDirection::Front => Color::linear_rgb(0., 0., 1.), // Blue for Front +z
                        crate::voxels::NeighbourDirection::Back => Color::linear_rgb(0., 1., 1.), // Cyan for Back -z
                    };
                    if target == chunk_entity {
                        let mut g = GizmoAsset::new();
                        g.cuboid(
                            Transform::from_scale(Vec3::splat(30.))
                                .with_translation(Vec3::splat(15.)),
                            color,
                        );
                        commands.entity(entity).insert(Gizmo {
                            handle: gizmos.add(g),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
}
