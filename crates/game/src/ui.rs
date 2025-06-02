use bevy::{prelude::*, window::PrimaryWindow};

pub fn spawn_crosshair(window_query: Query<&Window, With<PrimaryWindow>>, mut commands: Commands) {
    let window = window_query.single().unwrap();
    let crosshair_size = 15.;

    commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Relative,
            ..default()
        },))
        .with_child((
            Node {
                width: Val::Px(crosshair_size),
                height: Val::Px(2.),
                position_type: PositionType::Absolute,
                left: Val::Px(window.width() / 2.0 - crosshair_size + 2.),
                top: Val::Px(window.height() / 2.0 - crosshair_size / 2.0),
                ..default()
            },
            BackgroundColor(Color::srgb(1., 1., 1.)),
        ))
        .with_child((
            Node {
                width: Val::Px(2.),
                height: Val::Px(crosshair_size),
                position_type: PositionType::Absolute,
                left: Val::Px(window.width() / 2.0 - crosshair_size / 2.0),
                top: Val::Px(window.height() / 2.0 - crosshair_size + 2.),
                ..default()
            },
            BackgroundColor(Color::srgb(1., 1., 1.)),
        ));
}
