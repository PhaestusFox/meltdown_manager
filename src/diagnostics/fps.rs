use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use super::DiagnosticSettings;

pub fn plugin(app: &mut App) {
    app.add_plugins((bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),))
        .add_systems(Startup, reg_tab)
        .add_systems(Update, (update_fps, update_frame_time));
}

fn reg_tab(mut settings: ResMut<DiagnosticSettings>, mut commands: Commands) {
    let on_open = commands.register_system(on_open);
    let on_close = commands.register_system(on_close);

    settings.register_tab("FPS", on_open, on_close);
}

fn on_open(In(content): In<Entity>, mut commands: Commands) {
    commands.entity(content).with_children(|p| {
        p.spawn((Text::new("FPS"), FPSText));
        p.spawn((Text::new("FrameTime"), FrameTimeText));
    });
}

#[derive(Component)]
struct FPSText;
#[derive(Component)]
struct FrameTimeText;

fn on_close(content: In<Entity>, mut commands: Commands) {
    commands.entity(*content).despawn_related::<Children>();
}

fn update_fps(mut text: Query<&mut Text, With<FPSText>>, diagnostics: Res<DiagnosticsStore>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let Some(frame_time) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) else {
        error!("FPS not in DiagnosticsStore");
        return;
    };
    if let Some(avr) = frame_time.average() {
        text.0 = format!("FPS: {:.02?}", avr);
    } else {
        text.0 = String::from("FPS: N/A");
    }
}

fn update_frame_time(
    mut text: Query<&mut Text, With<FrameTimeText>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let Some(frame_time) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME) else {
        error!("Frame Time not in DiagnosticsStore");
        return;
    };
    if let Some(avr) = frame_time.value() {
        text.0 = format!("Frame Time: {:.04?}", avr);
    } else {
        text.0 = String::from("Frame Time: N/A");
    }
}
