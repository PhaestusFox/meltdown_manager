#![feature(impl_trait_in_assoc_type)]
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

pub mod voxels;

mod ui;

mod diagnostics;
mod player;

pub fn run_game() {
    // this is what the template was doing
    let default_plugins = DefaultPlugins
        .set(AssetPlugin {
            // Wasm builds will check for meta files (that don't exist) if this isn't set.
            // This causes errors and even panics in web builds on itch.
            // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
            meta_check: AssetMetaCheck::Never,
            ..default()
        })
        // use nearest to get crisp pixel art
        .set(ImagePlugin::default_nearest());

    // if in debug build uncap framerate to make it easier to know if we have frame budgit
    #[cfg(debug_assertions)]
    let default_plugins = default_plugins.set(WindowPlugin {
        primary_window: Some(Window {
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
        }),
        ..Default::default()
    });

    let mut app = App::new();
    app
        // add modifide DefaultPlugin
        .add_plugins(default_plugins)
        .add_plugins((player::plugin, voxels::plugin))
        .add_systems(Startup, setup)
        .add_systems(Startup, ui::spawn_crosshair);

    // add my diagnostics
    app.add_plugins(diagnostics::MeltdownDiagnosticsPlugin);
    // only add editor in debug builds
    #[cfg(debug_assertions)]
    app.add_plugins(bevy_editor_pls::EditorPlugin::default());

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        Camera3d::default(),
        player::Player { speed: 100. },
        IsDefaultUiCamera,
    ));
}

mod utils;
