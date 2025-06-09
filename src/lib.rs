#![feature(impl_trait_in_assoc_type)]
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy::scene::ron::de;
use bevy_console::{AddConsoleCommand, ConsoleConfiguration, ConsolePlugin};
use strum::IntoEnumIterator;

use crate::hotbar::block_selector_plugin;
use crate::menu::menu_plugin;
use crate::raycast::voxel_raycast_plugin;
use crate::voxels::block::BlockType;
use crate::voxels::map::ChunkData;

pub mod voxels;

pub use utils::BlockIter;

mod console;
mod diagnostics;
mod hotbar;
mod menu;
mod player;
mod raycast;
mod ui;

const TARGET_TICKTIME: f64 = 100.; // 10 ticks per second

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum GameState {
    #[default]
    Menu,
    Game,
}

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
    // #[cfg(debug_assertions)]
    let default_plugins = default_plugins.set(WindowPlugin {
        primary_window: Some(Window {
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..Default::default()
        }),
        ..Default::default()
    });

    let mut app = App::new();
    app.insert_resource(bevy_pkv::PkvStore::new("Phox", "meltdown_manager"));

    app.insert_resource(Time::<Fixed>::from_duration(
        std::time::Duration::from_millis(TARGET_TICKTIME as u64),
    ));
    app.add_systems(Startup, setup);
    app.add_plugins(default_plugins);
    app.init_state::<GameState>();
    app
        // add modifide DefaultPlugin
        .add_plugins((
            menu_plugin,
            player::plugin,
            voxels::map::map_plugin,
            // add my diagnostics
            diagnostics::MeltdownDiagnosticsPlugin,
            // only add editor in debug builds
            // editor is not supported on wasm32
            voxel_raycast_plugin,
        ))
        .add_systems(OnEnter(GameState::Game), ui::ui::spawn_crosshair);
    app.add_plugins(block_selector_plugin);
    // #[cfg(debug_assertions)]
    // #[cfg(not(target_arch = "wasm32"))]
    // app.add_plugins(bevy_editor_pls::EditorPlugin::default());

    app.add_plugins(console::plugin);

    // // dont know why some meshes are being detected as empty
    app.add_systems(Update, catch_failed_meshes);
    app.insert_resource(ConsoleConfiguration {
        // override config here
        ..Default::default()
    });
    app.run();
}

fn setup(mut commands: Commands) {
    let angle_rad = -45.0f32.to_radians();
    let cos_angle = angle_rad.cos();
    let sin_angle = angle_rad.sin();
    commands.spawn((
        Name::new("Player"),
        Camera3d::default(),
        player::Player { speed: 30. },
        IsDefaultUiCamera,
        Transform::from_matrix(Mat4 {
            x_axis: Vec4::from_array([1.0, 0.0, 0.0, 0.0]),
            y_axis: Vec4::from_array([0.0, cos_angle, sin_angle, 0.0]),
            z_axis: Vec4::from_array([0.0, -sin_angle, cos_angle, 0.0]),
            w_axis: Vec4::from_array([50.0, 25.0, 50.0, 1.0]),
        }),
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
                if *block != BlockType::Air {
                    not_air = true;
                }
            }
            if not_air {
                meshes.add_to_queue(entity);
            }
        }
    }
}
