use bevy::prelude::*;

use crate::{GameState, voxels::block::BlockType};

#[derive(Resource, Default)]
pub struct CurrentBlock(pub BlockType);

#[derive(Component)]
struct BlockSelector;

#[derive(Component)]
struct BlockButton {
    block_type: BlockType,
}

#[derive(Component)]
struct CurrentSelectionText;

#[derive(Component)]
struct BlockSelectorScreen;

const SELECTED_BUTTON_BORDER: Color = Color::srgb(1.0, 1.0, 0.0);
const NORMAL_BUTTON_BORDER: Color = Color::srgb(0.3, 0.3, 0.3);
const TEXT_COLOR: Color = Color::srgb(0.1, 0.1, 0.1);

pub fn block_selector_plugin(app: &mut App) {
    app.insert_resource(CurrentBlock(BlockType::default()))
        .add_systems(OnEnter(GameState::Game), setup_block_selector)
        .add_systems(
            Update,
            (handle_keyboard_input, update_button_colors).run_if(in_state(GameState::Game)),
        );
}

fn setup_block_selector(mut commands: Commands) {
    let button_node = Node {
        width: Val::Px(80.0),
        height: Val::Px(80.0),
        margin: UiRect::all(Val::Px(5.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    };

    let button_text_font = TextFont {
        font_size: 12.0,
        ..default()
    };

    let selection_text_font = TextFont {
        font_size: 18.0,
        ..default()
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            ..default()
        },
        BlockSelectorScreen,
        children![(
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(10.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            children![
                (
                    Text::new("Current: Air"),
                    selection_text_font,
                    TextColor(Color::srgb(0.1, 0.1, 0.1)),
                    CurrentSelectionText,
                    Node {
                        margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(0.0), Val::Px(5.0)),
                        ..default()
                    },
                ),
                // Horizontal row of buttons
                (
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    children![
                        // Air
                        (
                            Button,
                            button_node.clone(),
                            BackgroundColor(get_block_color(BlockType::Air)),
                            BorderColor(NORMAL_BUTTON_BORDER),
                            BlockButton {
                                block_type: BlockType::Air
                            },
                            children![(
                                Text::new("Air\n[0]"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),]
                        ),
                        // Copper
                        (
                            Button,
                            button_node.clone(),
                            BackgroundColor(get_block_color(BlockType::Copper)),
                            BorderColor(NORMAL_BUTTON_BORDER),
                            BlockButton {
                                block_type: BlockType::Copper
                            },
                            children![(
                                Text::new("Copper\n[1]"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),]
                        ),
                        // Iron
                        (
                            Button,
                            button_node.clone(),
                            BackgroundColor(get_block_color(BlockType::Iron)),
                            BorderColor(NORMAL_BUTTON_BORDER),
                            BlockButton {
                                block_type: BlockType::Iron
                            },
                            children![(
                                Text::new("Iron\n[2]"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),]
                        ),
                        // Steel
                        (
                            Button,
                            button_node.clone(),
                            BackgroundColor(get_block_color(BlockType::Steel)),
                            BorderColor(NORMAL_BUTTON_BORDER),
                            BlockButton {
                                block_type: BlockType::Steel
                            },
                            children![(
                                Text::new("Steel\n[3]"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),]
                        ),
                        // Uranium
                        (
                            Button,
                            button_node.clone(),
                            BackgroundColor(get_block_color(BlockType::Uranium)),
                            BorderColor(NORMAL_BUTTON_BORDER),
                            BlockButton {
                                block_type: BlockType::Uranium
                            },
                            children![(
                                Text::new("Uranium\n[4]"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),]
                        ),
                        // Water
                        (
                            Button,
                            button_node.clone(),
                            BackgroundColor(get_block_color(BlockType::Water)),
                            BorderColor(NORMAL_BUTTON_BORDER),
                            BlockButton {
                                block_type: BlockType::Water
                            },
                            children![(
                                Text::new("Water\n[5]"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),]
                        ),
                        // Thorium
                        (
                            Button,
                            button_node.clone(),
                            BackgroundColor(get_block_color(BlockType::Thorium)),
                            BorderColor(NORMAL_BUTTON_BORDER),
                            BlockButton {
                                block_type: BlockType::Thorium
                            },
                            children![(
                                Text::new("Thorium\n[6]"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),]
                        ),
                        // Wax
                        (
                            Button,
                            button_node.clone(),
                            BackgroundColor(get_block_color(BlockType::Wax)),
                            BorderColor(NORMAL_BUTTON_BORDER),
                            BlockButton {
                                block_type: BlockType::Wax
                            },
                            children![(
                                Text::new("Wax\n[7]"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),]
                        ),
                        // Rubber
                        (
                            Button,
                            button_node.clone(),
                            BackgroundColor(get_block_color(BlockType::Rubber)),
                            BorderColor(NORMAL_BUTTON_BORDER),
                            BlockButton {
                                block_type: BlockType::Rubber
                            },
                            children![(
                                Text::new("Rubber\n[8]"),
                                button_text_font.clone(),
                                TextColor(TEXT_COLOR),
                            ),]
                        ),
                    ]
                ),
            ]
        )],
    ));
}

fn get_block_color(block_type: BlockType) -> Color {
    match block_type {
        BlockType::Air => Color::srgba(0.8, 0.8, 1.0, 0.3),
        BlockType::Copper => Color::srgb(0.8, 0.4, 0.2),
        BlockType::Iron => Color::srgb(0.6, 0.6, 0.6),
        BlockType::Steel => Color::srgb(0.4, 0.4, 0.5),
        BlockType::Uranium => Color::srgb(0.2, 0.8, 0.2),
        BlockType::Water => Color::srgb(0.2, 0.4, 0.8),
        BlockType::Thorium => Color::srgb(0.6, 0.2, 0.8),
        BlockType::Wax => Color::srgb(0.9, 0.9, 0.6),
        BlockType::Rubber => Color::srgb(0.3, 0.3, 0.3),
        BlockType::Void => Color::srgb(0.1, 0.0, 0.2),
    }
}

fn handle_keyboard_input(
    mut current_block: ResMut<CurrentBlock>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let key_mappings = [
        KeyCode::Digit0,
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];

    for (index, key) in key_mappings.iter().enumerate() {
        if keyboard_input.just_pressed(*key)
            && let Some(block_type) = BlockType::from_repr(index as u8)
        {
            current_block.0 = block_type;
            println!("Selected block: {block_type:?}");
        }
    }
}

fn update_button_colors(
    current_block: Res<CurrentBlock>,
    mut button_query: Query<(&BlockButton, &mut BorderColor)>,
    mut text_query: Query<&mut Text, With<CurrentSelectionText>>,
) {
    for (block_button, mut border_color) in button_query.iter_mut() {
        if block_button.block_type == current_block.0 {
            *border_color = SELECTED_BUTTON_BORDER.into();
        } else {
            *border_color = NORMAL_BUTTON_BORDER.into();
        }
    }

    if let Ok(mut text) = text_query.single_mut() {
        text.0 = format!("Current: {:?}", current_block.0);
    }
}
