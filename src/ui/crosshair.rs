use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Resource, Default)]
pub struct Crosshair(pub bool);

#[derive(Component)]
pub struct CrosshairEntity;

pub struct CrosshairPlugin;

impl Plugin for CrosshairPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Crosshair>()
            .add_systems(Update, (toggle_crosshair, manage_crosshair).chain());
    }
}

pub fn toggle_crosshair(mut crosshair: ResMut<Crosshair>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::KeyC) {
        crosshair.0 = !crosshair.0;
    }
}

pub fn manage_crosshair(
    window_query: Query<&Window, With<PrimaryWindow>>,
    crosshair_on: Res<Crosshair>,
    crosshair_query: Query<Entity, With<CrosshairEntity>>,
    mut commands: Commands,
) {
    let crosshair_exists = !crosshair_query.is_empty();

    if crosshair_on.0 && !crosshair_exists {
        // Spawn crosshair
        spawn_crosshair_ui(&window_query, &mut commands);
    } else if !crosshair_on.0 && crosshair_exists {
        // Despawn crosshair
        for entity in crosshair_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn spawn_crosshair_ui(window_query: &Query<&Window, With<PrimaryWindow>>, commands: &mut Commands) {
    let window_blank = Window {
        ..Default::default()
    };
    let window = window_query.single().unwrap_or(&window_blank);
    let crosshair_size_px = 15.0;
    let crosshair_thickness_px = 2.0;

    let crosshair_size_percent = (crosshair_size_px / window.width()) * 100.0;
    let crosshair_thickness_width_percent = (crosshair_thickness_px / window.width()) * 100.0;
    let crosshair_thickness_height_percent = (crosshair_thickness_px / window.height()) * 100.0;

    let horizontal_center_offset = 50.0 - (crosshair_size_percent / 2.0);
    let vertical_center_offset = 50.0 - (crosshair_size_percent / 2.0);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Relative,
                ..default()
            },
            CrosshairEntity,
        ))
        .with_child((
            Node {
                width: Val::Percent(crosshair_size_percent),
                height: Val::Percent(crosshair_thickness_height_percent),
                position_type: PositionType::Absolute,
                left: Val::Percent(horizontal_center_offset),
                top: Val::Percent(50.0 - (crosshair_thickness_height_percent / 2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(1., 1., 1.)),
        ))
        .with_child((
            Node {
                width: Val::Percent(crosshair_thickness_width_percent),
                height: Val::Percent(crosshair_size_percent),
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0 - (crosshair_thickness_width_percent / 2.0)),
                top: Val::Percent(vertical_center_offset),
                ..default()
            },
            BackgroundColor(Color::srgb(1., 1., 1.)),
        ));
}
