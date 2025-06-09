use bevy::prelude::*;

use crate::{
    GameState,
    player::Player,
    voxels::{CHUNK_SIZE, ChunkId, ChunkManager, block::BlockType, cellular_automata::Cells},
};

/// Simple raycast hit information
#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub distance: f32,
    pub world_position: Vec3,
    pub voxel_position: IVec3,
    pub block_type: BlockType,
}

/// System to handle voxel interaction - FIXED VERSION
pub fn handle_voxel_interaction(
    camera_query: Query<&Transform, (With<Camera3d>, With<Player>)>,
    chunk_manager: Res<ChunkManager>,
    mut chunks_query: Query<(&ChunkId, &mut Cells)>, // Single mutable query
    input: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    // Get camera position and forward direction
    let start_pos = camera_transform.translation;
    let forward = camera_transform.forward().as_vec3();
    let max_distance = 10.0;

    // Left click to remove block
    if input.just_pressed(MouseButton::Left) {
        if let Some(solid_hit) = raycast_for_solid_block(
            start_pos,
            forward,
            max_distance,
            &chunk_manager,
            &chunks_query,
        ) {
            println!(
                "Removing block at {:?}: {:?}",
                solid_hit.voxel_position, solid_hit.block_type
            );

            if set_block_at_position(
                solid_hit.voxel_position,
                BlockType::Air,
                &chunk_manager,
                &mut chunks_query,
            ) {
                println!("Successfully removed block");
            } else {
                println!("Failed to remove block - chunk not found");
            }
        } else {
            println!("No solid block found in range");
        }
    }

    // Right click to place block
    if input.just_pressed(MouseButton::Right) {
        if let Some(solid_hit) = raycast_for_solid_block(
            start_pos,
            forward,
            max_distance,
            &chunk_manager,
            &chunks_query,
        ) {
            if let Some(placement_pos) =
                find_placement_position(forward, &solid_hit, &chunk_manager, &chunks_query)
            {
                // Choose block type based on key pressed
                let block_type = if keys.pressed(KeyCode::Digit1) {
                    BlockType::Copper
                } else if keys.pressed(KeyCode::Digit2) {
                    BlockType::Iron
                } else if keys.pressed(KeyCode::Digit3) {
                    BlockType::Steel
                } else if keys.pressed(KeyCode::Digit4) {
                    BlockType::Uranium
                } else if keys.pressed(KeyCode::Digit5) {
                    BlockType::Water
                } else {
                    BlockType::Copper // Default
                };

                println!(
                    "Placing {:?} block at {:?} (next to {:?})",
                    block_type, placement_pos, solid_hit.voxel_position
                );

                if set_block_at_position(
                    placement_pos,
                    block_type,
                    &chunk_manager,
                    &mut chunks_query,
                ) {
                    println!("Successfully placed block");
                } else {
                    println!("Failed to place block - chunk not found");
                }
            } else {
                println!(
                    "No suitable placement position found near {:?}",
                    solid_hit.voxel_position
                );
            }
        } else {
            println!("No solid block found to place against");
        }
    }

    // Debug: Show raycast info
    if input.just_pressed(MouseButton::Middle) {
        println!("=== Raycast Debug ===");
        println!("Start: {:?}", start_pos);
        println!("Direction: {:?}", forward);

        if let Some(solid_hit) = raycast_for_solid_block(
            start_pos,
            forward,
            max_distance,
            &chunk_manager,
            &chunks_query,
        ) {
            println!(
                "First solid block: {:?} at {:?} (distance: {:.2})",
                solid_hit.block_type, solid_hit.voxel_position, solid_hit.distance
            );

            if let Some(placement_pos) =
                find_placement_position(forward, &solid_hit, &chunk_manager, &chunks_query)
            {
                println!("Placement position: {:?}", placement_pos);
            } else {
                println!("No placement position available");
            }
        } else {
            println!("No solid block found");
        }
    }
}

// Update the helper functions to work with mutable queries
/// Get block type at a world position - Updated for mutable query
fn get_block_at_position(
    voxel_pos: IVec3,
    chunk_manager: &ChunkManager,
    chunks_query: &Query<(&ChunkId, &mut Cells)>,
) -> BlockType {
    // Calculate which chunk this voxel belongs to
    let chunk_id = ChunkId(IVec3::from_array([
        voxel_pos.x.div_euclid(CHUNK_SIZE),
        voxel_pos.y.div_euclid(CHUNK_SIZE),
        voxel_pos.z.div_euclid(CHUNK_SIZE),
    ]));

    // Calculate local position within the chunk
    let local_pos = IVec3::new(
        voxel_pos.x.rem_euclid(CHUNK_SIZE),
        voxel_pos.y.rem_euclid(CHUNK_SIZE),
        voxel_pos.z.rem_euclid(CHUNK_SIZE),
    );

    // Find the chunk entity and get the block
    for (id, cells) in chunks_query.iter() {
        if *id == chunk_id {
            let cell = cells.get_cell(local_pos.x, local_pos.y, local_pos.z);
            return cell.get_block_type();
        }
    }

    // If chunk not found, return Void (unloaded area)
    BlockType::Void
}

/// Set block type at a world position - Updated for mutable query
fn set_block_at_position(
    voxel_pos: IVec3,
    block_type: BlockType,
    chunk_manager: &ChunkManager,
    chunks_query: &mut Query<(&ChunkId, &mut Cells)>,
) -> bool {
    // Calculate which chunk this voxel belongs to
    let chunk_id = ChunkId(IVec3::from_array([
        voxel_pos.x.div_euclid(CHUNK_SIZE),
        voxel_pos.y.div_euclid(CHUNK_SIZE),
        voxel_pos.z.div_euclid(CHUNK_SIZE),
    ]));

    // Calculate local position within the chunk
    let local_pos = IVec3::new(
        voxel_pos.x.rem_euclid(CHUNK_SIZE),
        voxel_pos.y.rem_euclid(CHUNK_SIZE),
        voxel_pos.z.rem_euclid(CHUNK_SIZE),
    );

    // Find the chunk entity and set the block
    for (id, mut cells) in chunks_query.iter_mut() {
        if *id == chunk_id {
            let mut cell = cells.get_cell(local_pos.x, local_pos.y, local_pos.z);
            cell.set_block_type(block_type);
            return true;
        }
    }

    false // Chunk not found
}

/// Updated raycast function to work with mutable query
pub fn raycast_for_solid_block(
    start_pos: Vec3,
    direction: Vec3,
    max_distance: f32,
    chunk_manager: &ChunkManager,
    chunks_query: &Query<(&ChunkId, &mut Cells)>,
) -> Option<RaycastHit> {
    let direction = direction.normalize();
    let step_size = 0.1; // Small steps for accuracy
    let max_steps = (max_distance / step_size) as i32;

    for step in 0..max_steps {
        let distance = step as f32 * step_size;
        let world_pos = start_pos + direction * distance;
        let voxel_pos = IVec3::new(
            world_pos.x.floor() as i32,
            world_pos.y.floor() as i32,
            world_pos.z.floor() as i32,
        );

        let block_type = get_block_at_position(voxel_pos, chunk_manager, chunks_query);

        // Check if this is a solid block
        if block_type != BlockType::Air && block_type != BlockType::Void {
            return Some(RaycastHit {
                distance,
                world_position: world_pos,
                voxel_position: voxel_pos,
                block_type,
            });
        }
    }

    None
}

/// Updated placement function to work with mutable query
pub fn find_placement_position(
    direction: Vec3,
    solid_hit: &RaycastHit,
    chunk_manager: &ChunkManager,
    chunks_query: &Query<(&ChunkId, &mut Cells)>,
) -> Option<IVec3> {
    let direction = direction.normalize();

    // Try placing one block back along the ray from the solid block
    let back_step = -direction * 1.0;
    let placement_pos = solid_hit.voxel_position
        + IVec3::new(
            back_step.x.round() as i32,
            back_step.y.round() as i32,
            back_step.z.round() as i32,
        );

    // Check if the placement position is empty
    let block_at_placement = get_block_at_position(placement_pos, chunk_manager, chunks_query);
    if block_at_placement == BlockType::Air || block_at_placement == BlockType::Void {
        return Some(placement_pos);
    }

    // If that doesn't work, try the 6 adjacent positions to the solid block
    let adjacent_offsets = [
        IVec3::new(1, 0, 0),  // +X
        IVec3::new(-1, 0, 0), // -X
        IVec3::new(0, 1, 0),  // +Y
        IVec3::new(0, -1, 0), // -Y
        IVec3::new(0, 0, 1),  // +Z
        IVec3::new(0, 0, -1), // -Z
    ];

    for offset in adjacent_offsets {
        let candidate_pos = solid_hit.voxel_position + offset;
        let block_at_candidate = get_block_at_position(candidate_pos, chunk_manager, chunks_query);

        if block_at_candidate == BlockType::Air || block_at_candidate == BlockType::Void {
            return Some(candidate_pos);
        }
    }

    None
}

// Plugin setup
pub fn voxel_raycast_plugin(app: &mut App) {
    app.add_systems(
        Update,
        handle_voxel_interaction.run_if(in_state(GameState::Game)),
    );
}
