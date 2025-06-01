use bevy::{
    diagnostic::{DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use super::DiagnosticSettings;

pub fn plugin(app: &mut App) {
    app.add_plugins((bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),))
        .add_systems(Startup, reg_tab)
        .add_systems(Update, (update_count));
}

fn reg_tab(mut settings: ResMut<DiagnosticSettings>, mut commands: Commands) {
    let on_open = commands.register_system(on_open);
    let on_close = commands.register_system(on_close);

    settings.register_tab("Entity", on_open, on_close);
}

fn on_open(In(content): In<Entity>, mut commands: Commands) {
    commands.entity(content).with_children(|p| {
        p.spawn((Text::new("Count"), CountText));
    });
}

#[derive(Component)]
struct CountText;

fn on_close(content: In<Entity>, mut commands: Commands) {
    commands.entity(*content).despawn_related::<Children>();
}

fn update_count(mut text: Query<&mut Text, With<CountText>>, diagnostics: Res<DiagnosticsStore>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let Some(frame_time) = diagnostics.get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT) else {
        error!("Entity count not in DiagnosticsStore");
        return;
    };
    if let Some(avr) = frame_time.value() {
        text.0 = format!("Count: {:.00?}", avr);
    } else {
        text.0 = String::from("Count: N/A");
    }
}
