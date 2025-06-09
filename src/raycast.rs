use bevy::prelude::*;

use crate::{
    GameState,
    hotbar::CurrentBlock,
    player::Player,
    voxels::{
        CHUNK_SIZE, ChunkId, ChunkManager,
        block::BlockType,
        cellular_automata::{CellData, Cells},
    },
};

#[derive(Debug, Clone)]
pub struct RaycastHit {
    pub distance: f32,
    pub voxel_position: IVec3,
    pub cell_data: CellData,
}

#[derive(Resource, Default)]
pub struct DebugUIVisible(pub bool);

#[derive(Component)]
struct DebugUIPanel;

#[derive(Component)]
struct DebugUIContent;

pub fn handle_voxel_interaction(
    camera_query: Query<&Transform, (With<Camera3d>, With<Player>)>,
    mut chunks_query: Query<(&ChunkId, &mut Cells)>,
    input: Res<ButtonInput<MouseButton>>,
    current_block: Res<CurrentBlock>,
    mut debug_ui_visible: ResMut<DebugUIVisible>,
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
        if let Some(solid_hit) =
            raycast_for_solid_block(start_pos, forward, max_distance, &chunks_query)
        {
            println!(
                "Removing block at {:?}: {:?}",
                solid_hit.voxel_position, solid_hit.cell_data
            );

            if set_block_at_position(solid_hit.voxel_position, BlockType::Air, &mut chunks_query) {
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
        if let Some(solid_hit) =
            raycast_for_solid_block(start_pos, forward, max_distance, &chunks_query)
        {
            if let Some(placement_pos) = find_placement_position(forward, &solid_hit, &chunks_query)
            {
                // Choose block type based on key pressed
                let block_type = current_block.0;

                println!(
                    "Placing {:?} block at {:?} (next to {:?})",
                    block_type, placement_pos, solid_hit.voxel_position
                );

                if set_block_at_position(placement_pos, block_type, &mut chunks_query) {
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

    // Middle click to toggle debug UI
    if input.just_pressed(MouseButton::Middle) {
        debug_ui_visible.0 = !debug_ui_visible.0;
        println!("Debug UI toggled: {}", debug_ui_visible.0);
    }
}

#[derive(Resource, Default)]
pub struct AirDebug(pub bool);

pub fn toggle_crosshair(mut crosshair: ResMut<AirDebug>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyT) {
        crosshair.0 = !crosshair.0;
    }
}

// New system to update debug UI content
fn update_debug_ui(
    camera_query: Query<&Transform, (With<Camera3d>, With<Player>)>,
    chunks_query: Query<(&ChunkId, &mut Cells)>,
    debug_ui_visible: Res<DebugUIVisible>,
    mut debug_content_query: Query<&mut Text, With<DebugUIContent>>,
    mut debug_panel_query: Query<&mut Node, With<DebugUIPanel>>,
    air_on: Res<AirDebug>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    // Show/hide the debug panel
    if let Ok(mut style) = debug_panel_query.single_mut() {
        style.display = if debug_ui_visible.0 {
            Display::Flex
        } else {
            Display::None
        };
    }

    // Update content only if visible
    if !debug_ui_visible.0 {
        return;
    }

    let Ok(mut text) = debug_content_query.single_mut() else {
        return;
    };

    // Get camera position and forward direction
    let start_pos = camera_transform.translation;
    let forward = camera_transform.forward().as_vec3();
    let max_distance = 10.0;

    // Perform detailed raycast
    let ray_hits = raycast_all_blocks(start_pos, forward, max_distance, &chunks_query);

    // Format the debug information
    let mut content = String::from("=== RAYCAST DEBUG ===\nToggle Air View with T\n");
    content.push_str(&format!(
        "Origin: ({:.1}, {:.1}, {:.1})\n",
        start_pos.x, start_pos.y, start_pos.z
    ));
    content.push_str(&format!(
        "Direction: ({:.2}, {:.2}, {:.2})\n",
        forward.x, forward.y, forward.z
    ));
    content.push_str(&format!("Max Distance: {:.1}\n", max_distance));
    content.push_str(&format!("Blocks Found: {}\n\n", ray_hits.len()));

    if ray_hits.is_empty() {
        content.push_str("No blocks encountered in ray path.");
    } else if air_on.0 {
        for (i, hit) in ray_hits.iter().enumerate() {
            content.push_str(&format!("--- BLOCK {} ---\n", i + 1));
            content.push_str(&format!("Type: {:?}\n", hit.cell_data.get_block_type()));
            content.push_str(&format!(
                "Position: ({}, {}, {})\n",
                hit.voxel_position.x, hit.voxel_position.y, hit.voxel_position.z
            ));
            content.push_str(&format!("Distance: {:.2}m\n", hit.distance));
            content.push_str(&format!("Temperature: {:.1}K\n", hit.cell_data.tempreture));
            content.push_str(&format!("Energy: {:.2}kJ\n", hit.cell_data.energy));
            content.push_str(&format!("Density: {:.3}kg/m^3\n", hit.cell_data.density));

            if i < ray_hits.len() - 1 {
                content.push_str("\n");
            }
        }
    } else {
        for (i, hit) in ray_hits.iter().enumerate() {
            if hit.cell_data.get_block_type() != BlockType::Air {
                content.push_str(&format!("--- BLOCK {} ---\n", i + 1));
                content.push_str(&format!("Type: {:?}\n", hit.cell_data.get_block_type()));
                content.push_str(&format!(
                    "Position: ({}, {}, {})\n",
                    hit.voxel_position.x, hit.voxel_position.y, hit.voxel_position.z
                ));
                content.push_str(&format!("Distance: {:.2}m\n", hit.distance));
                content.push_str(&format!("Temperature: {:.1}K\n", hit.cell_data.tempreture));
                content.push_str(&format!("Energy: {:.2}kJ\n", hit.cell_data.energy));
                content.push_str(&format!("Density: {:.3}kg/m^3\n", hit.cell_data.density));

                if i < ray_hits.len() - 1 {
                    content.push_str("\n");
                }
            }
        }
    }

    text.0 = content;
}

// New system to setup debug UI
pub fn setup_debug_ui(mut commands: Commands) {
    let panel_style = Node {
        position_type: PositionType::Absolute,
        top: Val::Percent(0.0),
        right: Val::Percent(0.0),
        width: Val::Percent(25.0),
        max_height: Val::Percent(80.0),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Percent(0.1)),
        border: UiRect::all(Val::Percent(0.01)),
        display: Display::Flex,
        overflow: Overflow::scroll_y(),
        ..default()
    };

    let text_font = TextFont {
        font_size: 12.0,
        ..default()
    };

    commands.spawn((
        panel_style,
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        DebugUIPanel,
        children![(
            Text::new("Debug information will appear here..."),
            text_font,
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            DebugUIContent,
        )],
    ));
}

pub fn raycast_all_blocks(
    start_pos: Vec3,
    direction: Vec3,
    max_distance: f32,
    chunks_query: &Query<(&ChunkId, &mut Cells)>,
) -> Vec<RaycastHit> {
    let direction = direction.normalize();
    let step_size = 0.1;
    let max_steps = (max_distance / step_size) as i32;
    let mut hits = Vec::new();
    let mut last_voxel_pos = None;

    for step in 0..max_steps {
        let distance = step as f32 * step_size;
        let world_pos = start_pos + direction * distance;
        let voxel_pos = IVec3::new(
            world_pos.x.floor() as i32,
            world_pos.y.floor() as i32,
            world_pos.z.floor() as i32,
        );

        if Some(voxel_pos) != last_voxel_pos {
            let cell_data = get_block_at_position(voxel_pos, chunks_query);
            let block_type = cell_data.get_block_type();

            if block_type != BlockType::Void {
                hits.push(RaycastHit {
                    distance,
                    voxel_position: voxel_pos,
                    cell_data,
                });
            }

            last_voxel_pos = Some(voxel_pos);
        }
    }

    hits
}

fn get_block_at_position(
    voxel_pos: IVec3,
    chunks_query: &Query<(&ChunkId, &mut Cells)>,
) -> CellData {
    let chunk_id = ChunkId(IVec3::from_array([
        voxel_pos.x.div_euclid(CHUNK_SIZE),
        voxel_pos.y.div_euclid(CHUNK_SIZE),
        voxel_pos.z.div_euclid(CHUNK_SIZE),
    ]));

    let local_pos = IVec3::new(
        voxel_pos.x.rem_euclid(CHUNK_SIZE),
        voxel_pos.y.rem_euclid(CHUNK_SIZE),
        voxel_pos.z.rem_euclid(CHUNK_SIZE),
    );

    for (id, cells) in chunks_query.iter() {
        if *id == chunk_id {
            let cell = cells.get_cell(local_pos.x, local_pos.y, local_pos.z);
            return cell;
        }
    }

    CellData {
        block: BlockType::Void,
        ..Default::default()
    }
}

fn set_block_at_position(
    voxel_pos: IVec3,
    block_type: BlockType,
    chunks_query: &mut Query<(&ChunkId, &mut Cells)>,
) -> bool {
    let chunk_id = ChunkId(IVec3::from_array([
        voxel_pos.x.div_euclid(CHUNK_SIZE),
        voxel_pos.y.div_euclid(CHUNK_SIZE),
        voxel_pos.z.div_euclid(CHUNK_SIZE),
    ]));

    let local_pos = IVec3::new(
        voxel_pos.x.rem_euclid(CHUNK_SIZE),
        voxel_pos.y.rem_euclid(CHUNK_SIZE),
        voxel_pos.z.rem_euclid(CHUNK_SIZE),
    );

    for (id, mut cells) in chunks_query.iter_mut() {
        if *id == chunk_id {
            let CellData {
                block: _,
                energy,
                tempreture,
                density,
                flags,
            } = cells.get_cell(local_pos.x, local_pos.y, local_pos.z);
            cells.set_cell(
                local_pos.x,
                local_pos.y,
                local_pos.z,
                CellData {
                    block: block_type,
                    energy,
                    tempreture,
                    density,
                    flags,
                },
            );
            return true;
        }
    }

    false // Chunk not found
}

pub fn raycast_for_solid_block(
    start_pos: Vec3,
    direction: Vec3,
    max_distance: f32,
    chunks_query: &Query<(&ChunkId, &mut Cells)>,
) -> Option<RaycastHit> {
    let direction = direction.normalize();
    let step_size = 0.1;
    let max_steps = (max_distance / step_size) as i32;

    for step in 0..max_steps {
        let distance = step as f32 * step_size;
        let world_pos = start_pos + direction * distance;
        let voxel_pos = IVec3::new(
            world_pos.x.floor() as i32,
            world_pos.y.floor() as i32,
            world_pos.z.floor() as i32,
        );

        let cell_data = get_block_at_position(voxel_pos, chunks_query);
        let block_type = cell_data.get_block_type();
        // Check if this is a solid block
        if block_type != BlockType::Air && block_type != BlockType::Void {
            return Some(RaycastHit {
                distance,
                voxel_position: voxel_pos,
                cell_data,
            });
        }
    }

    None
}

/// Updated placement function to work with mutable query
pub fn find_placement_position(
    direction: Vec3,
    solid_hit: &RaycastHit,
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
    let block_at_placement = get_block_at_position(placement_pos, chunks_query).get_block_type();
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
        let block_at_candidate =
            get_block_at_position(candidate_pos, chunks_query).get_block_type();

        if block_at_candidate == BlockType::Air || block_at_candidate == BlockType::Void {
            return Some(candidate_pos);
        }
    }

    None
}

pub fn voxel_raycast_plugin(app: &mut App) {
    app.insert_resource(DebugUIVisible::default())
        .insert_resource(AirDebug::default())
        .add_systems(OnEnter(GameState::Game), setup_debug_ui)
        .add_systems(
            Update,
            (handle_voxel_interaction, update_debug_ui, toggle_crosshair)
                .run_if(in_state(GameState::Game)),
        );
}
