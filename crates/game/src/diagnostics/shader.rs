use bevy::{
    pbr::ExtendedMaterial,
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        storage::ShaderStorageBuffer,
    },
};

const SHADER_ASSET_PATH: &str = "shaders/temperature.wgsl";

pub type AutomitaDiagnosticMateraial =
    ExtendedMaterial<phoxels::prelude::VoxelMaterial, AutomitaMateraial>;

#[derive(AsBindGroup, Clone, TypePath, Asset)]
pub struct AutomitaMateraial {
    #[storage(4, read_only)]
    pub data: Handle<ShaderStorageBuffer>,
}

pub fn plugin(app: &mut App) {
    app.add_plugins(MaterialPlugin::<AutomitaDiagnosticMateraial>::default());
}

impl bevy::pbr::MaterialExtension for AutomitaMateraial {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
