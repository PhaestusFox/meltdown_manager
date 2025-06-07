#![feature(impl_trait_in_assoc_type)]
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::scene::ron::de;
use bevy_console::{AddConsoleCommand, ConsoleConfiguration, ConsolePlugin};
use strum::IntoEnumIterator;

use crate::console::commands::{
    ChunkHighlightCommand, ChunkHighlightState, chunk_highlight_command,
};
use crate::voxels::blocks::Blocks;
use crate::voxels::map::ChunkData;

pub mod voxels;

mod console;
mod diagnostics;
mod player;
mod ui;

const TARGET_TICKTIME: f64 = 100.; // 10 ticks per second

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

    app.insert_resource(Time::<Fixed>::from_duration(
        std::time::Duration::from_millis(TARGET_TICKTIME as u64),
    ));
    app
        // add modifide DefaultPlugin
        .add_plugins((
            default_plugins,
            player::plugin,
            voxels::map::map_plugin,
            // add my diagnostics
            diagnostics::MeltdownDiagnosticsPlugin,
            // only add editor in debug builds
            // editor is not supported on wasm32
        ))
        .add_systems(Startup, (setup, ui::ui::spawn_crosshair))
        .add_console_command::<ChunkHighlightCommand, _>(chunk_highlight_command);

    #[cfg(debug_assertions)]
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(bevy_editor_pls::EditorPlugin::default());

    app.add_plugins(ConsolePlugin);

    // // dont know why some meshes are being detected as empty
    app.add_systems(Update, catch_failed_meshes);
    app.insert_resource(ConsoleConfiguration {
        // override config here
        ..Default::default()
    });
    app.insert_resource(ChunkHighlightState {
        ..Default::default()
    });
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

fn catch_failed_meshes(
    mut meshes: ResMut<phoxels::ChunkMesher>,
    query: Query<(Entity, &ChunkData), (With<ChunkData>, Without<Mesh3d>)>,
) {
    if meshes.is_empty() {
        for (entity, data) in query.iter() {
            let mut not_air = false;
            for block in data.iter() {
                if *block != Blocks::Air {
                    not_air = true;
                }
            }
            if not_air {
                meshes.add_to_queue(entity);
            }
        }
    }
}
