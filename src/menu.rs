use bevy::{
    app::AppExit,
    color::palettes::css::CRIMSON,
    ecs::spawn::{SpawnIter, SpawnWith},
    prelude::*,
};
use bevy_simple_text_input::{TextInput, TextInputPlaceholder, TextInputValue};

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

use super::GameState;

const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);

#[derive(Resource, Debug, Component, PartialEq, Copy, Clone)]
pub struct MapSize(pub UVec3);

impl Default for MapSize {
    fn default() -> Self {
        MapSize(UVec3::new(15, 3, 15)) // Default map size
    }
}

#[derive(Component)]
struct CurrentMapSizeDisplay;

// Tag component for the input field
#[derive(Component)]
struct MapSizeInputField;

pub fn menu_plugin(app: &mut App) {
    app.init_state::<MenuState>()
        .add_plugins(bevy_simple_text_input::TextInputPlugin)
        .init_resource::<MapSize>()
        .add_systems(OnEnter(GameState::Menu), menu_setup)
        .add_systems(OnEnter(MenuState::Main), main_menu_setup)
        .add_systems(OnEnter(MenuState::Settings), settings_menu_setup)
        .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
        .add_systems(
            OnExit(MenuState::Settings),
            despawn_screen::<OnSettingsMenuScreen>,
        )
        .add_systems(
            Update,
            (
                menu_action,
                button_system,
                update_map_size_display,
                set_map_size_button_action,
            )
                .run_if(in_state(GameState::Menu)),
        );
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    Main,
    Settings,
    #[default]
    Disabled,
}

#[derive(Component)]
struct OnMainMenuScreen;

#[derive(Component)]
struct OnSettingsMenuScreen;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct SelectedOption;

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Settings,
    SetMapSize,
    BackToMainMenu,
    BackToSettings,
    Quit,
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

fn setting_button<T: Resource + Component + PartialEq + Copy>(
    interaction_query: Query<(&Interaction, &T, Entity), (Changed<Interaction>, With<Button>)>,
    selected_query: Single<(Entity, &mut BackgroundColor), With<SelectedOption>>,
    mut commands: Commands,
    mut setting: ResMut<T>,
) {
    let (previous_button, mut previous_button_color) = selected_query.into_inner();
    for (interaction, button_setting, entity) in &interaction_query {
        if *interaction == Interaction::Pressed && *setting != *button_setting {
            *previous_button_color = NORMAL_BUTTON.into();
            commands.entity(previous_button).remove::<SelectedOption>();
            commands.entity(entity).insert(SelectedOption);
            *setting = *button_setting;
        }
    }
}

fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

fn main_menu_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let button_node = Node {
        width: Val::Percent(90.0),
        height: Val::Percent(30.0),
        margin: UiRect::all(Val::Percent(2.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_font = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnMainMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                width: Val::Percent(50.0),
                height: Val::Percent(90.0),
                ..default()
            },
            BackgroundColor(CRIMSON.into()),
            children![
                (
                    Text::new("Meltdown Manager"),
                    TextFont {
                        font_size: 67.0,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        margin: UiRect::all(Val::Percent(5.0)),
                        ..default()
                    },
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Play,
                    children![(
                        Text::new("New Game"),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                    ),]
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Settings,
                    children![(
                        Text::new("Settings"),
                        button_text_font.clone(),
                        TextColor(TEXT_COLOR),
                    ),]
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::Quit,
                    children![(Text::new("Quit"), button_text_font, TextColor(TEXT_COLOR),),]
                ),
            ]
        )],
    ));
}

fn settings_menu_setup(mut commands: Commands, map_size: Res<MapSize>) {
    let button_node = Node {
        width: Val::Percent(90.0),
        height: Val::Percent(30.0),
        margin: UiRect::all(Val::Percent(1.5)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let input_field_node = Node {
        width: Val::Percent(90.0),
        height: Val::Percent(30.0),
        margin: UiRect::all(Val::Percent(1.5)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let text_style = TextFont {
        font_size: 25.0,
        ..default()
    };
    let button_text_style = TextFont {
        font_size: 33.0,
        ..default()
    };

    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        OnSettingsMenuScreen,
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                width: Val::Percent(50.0),
                height: Val::Percent(50.0),
                ..default()
            },
            BackgroundColor(CRIMSON.into()),
            children![
                (
                    Text::new(format!("Current Map Size: {}", map_size.0)),
                    text_style.clone(),
                    TextColor(TEXT_COLOR),
                    CurrentMapSizeDisplay,
                ),
                (
                    Text::new("Use format X,Y,Z with positive integers"),
                    text_style.clone(),
                    TextColor(TEXT_COLOR),
                ),
                (
                    input_field_node,
                    BackgroundColor(NORMAL_BUTTON),
                    TextInput::default(),
                    TextInputPlaceholder {
                        value: "X,Y,Z".to_string(),
                        text_color: Some(Color::srgb(0.5, 0.5, 0.5).into()),
                        ..Default::default()
                    },
                ),
                (
                    Button,
                    button_node.clone(),
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::SetMapSize,
                    children![(
                        Text::new("Set Map"),
                        button_text_style.clone(),
                        TextColor(TEXT_COLOR),
                    ),]
                ),
                (
                    Button,
                    button_node,
                    BackgroundColor(NORMAL_BUTTON),
                    MenuButtonAction::BackToMainMenu,
                    children![(Text::new("Back"), button_text_style, TextColor(TEXT_COLOR),),]
                ),
            ]
        )],
    ));
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.write(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    game_state.set(GameState::Game);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::Settings => menu_state.set(MenuState::Settings),
                MenuButtonAction::BackToMainMenu => menu_state.set(MenuState::Main),
                MenuButtonAction::BackToSettings => {
                    menu_state.set(MenuState::Settings);
                }
                MenuButtonAction::SetMapSize => {
                    //set_map_size_button_action
                }
            }
        }
    }
}

fn update_map_size_display(
    map_size: Res<MapSize>,
    mut query: Query<&mut Text, With<CurrentMapSizeDisplay>>,
) {
    if map_size.is_changed() {
        for mut text in &mut query {
            text.0 = format!("Current Map Size: {}", map_size.0);
        }
    }
}
fn set_map_size_button_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut map_size: ResMut<MapSize>,
    mut text_input_query: Query<(&TextInput, &mut TextInputValue)>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed
            && let MenuButtonAction::SetMapSize = menu_button_action
        {
            if let Ok((_text_input_marker, mut text_input_value)) = text_input_query.single_mut() {
                let input_string = text_input_value.0.clone();

                let parts: Vec<&str> = input_string.split(',').collect();

                if parts.len() == 3 {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        parts[0].trim().parse::<u32>(),
                        parts[1].trim().parse::<u32>(),
                        parts[2].trim().parse::<u32>(),
                    ) {
                        map_size.0 = UVec3::new(x, y, z);
                        info!("Map size updated to: {:?}", map_size.0);
                        text_input_value.0.clear();
                    } else {
                        warn!(
                            "Invalid input: Could not parse to u32. Please use format X,Y,Z (e.g., 10,1,10)"
                        );
                    }
                } else {
                    warn!("Invalid input format. Please use format X,Y,Z (e.g., 10,1,10)");
                }
            } else {
                error!(
                    "Could not find a single TextInput with a TextInputValue component for map size input."
                );
            }
        }
    }
}
