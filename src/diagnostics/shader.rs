use std::num::NonZeroU8;

use crate::voxels::block::BlockType;
use bevy::{
    asset::{load_internal_asset, weak_handle},
    prelude::*,
    render::{
        mesh::{MeshVertexAttribute, VertexFormat},
        render_resource::{AsBindGroup, StorageBuffer},
        storage::ShaderStorageBuffer,
    },
};

const SHADER_PATH: &str = "shaders/temperature.wgsl";

pub const BLOCK_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("BlockData", 988540919, VertexFormat::Uint32);

pub struct VoxelShaderPlugin;

impl Plugin for VoxelShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<DebugMaterial>::default());
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct DebugMaterial {
    #[uniform(0)]
    pub atlas_shape: UVec4,
    #[texture(1)]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
    #[uniform(3)]
    pub overrides: [phoxels::prelude::BlockOverrides; 256 / 4],
    #[storage(4, read_only)]
    pub data: Handle<ShaderStorageBuffer>,
    #[uniform(5)]
    pub settings: u32,
}

// impl DebugMaterial {
//     pub fn set_override(&mut self, block: Blocks, override_data: BlockOverride) {
//         let index = block as usize / 4;
//         let offset = (block as usize % 4) as u32;

//         debug_assert!(
//             index < self.overrides.len(),
//             "Block override index out of bounds: {}",
//             index
//         );

//         let mut data = 0;

//         if let Some(stride) = override_data.back {
//             debug_assert!(
//                 stride.get() < 32,
//                 "Back override value out of bounds: {}",
//                 stride.get()
//             );
//             data |= stride.get() as u32; // Store the stride value
//         }
//         data <<= 5;
//         if let Some(stride) = override_data.left {
//             debug_assert!(
//                 stride.get() < 32,
//                 "Left override value out of bounds: {}",
//                 stride.get()
//             );
//             data |= stride.get() as u32; // Store the stride value
//         }
//         data <<= 5;
//         if let Some(stride) = override_data.right {
//             debug_assert!(
//                 stride.get() < 32,
//                 "Left override value out of bounds: {}",
//                 stride.get()
//             );
//             data |= stride.get() as u32; // Store the stride value
//         }
//         data <<= 5;
//         if let Some(stride) = override_data.top {
//             debug_assert!(
//                 stride.get() < 32,
//                 "Left override value out of bounds: {}",
//                 stride.get()
//             );
//             data |= stride.get() as u32; // Store the stride value
//         }
//         data <<= 5;
//         if let Some(stride) = override_data.bottom {
//             debug_assert!(
//                 stride.get() < 32,
//                 "Left override value out of bounds: {}",
//                 stride.get()
//             );
//             data |= stride.get() as u32; // Store the stride value
//         }

//         let overrides = &mut self.overrides[index];
//         match offset {
//             0 => overrides.block_a = data,
//             1 => overrides.block_b = data,
//             2 => overrides.block_c = data,
//             3 => overrides.block_d = data,
//             _ => unreachable!("Offset must be between 0 and 3, got: {}", offset),
//         };
//     }
// }

#[derive(Default)]
pub struct BlockOverride {
    back: Option<NonZeroU8>,
    left: Option<NonZeroU8>,
    right: Option<NonZeroU8>,
    top: Option<NonZeroU8>,
    bottom: Option<NonZeroU8>,
}

impl BlockOverride {
    pub fn back(mut self, stride: u8) -> Self {
        if stride == 0 {
            self.top = None;
        } else {
            debug_assert!(stride < 32, "Stride must be less than 32");
            // SAFETY: We ensure that stride is non-zero and within bounds
            self.back = unsafe { Some(NonZeroU8::new_unchecked(stride)) };
        }
        self
    }
    pub fn left(mut self, stride: u8) -> Self {
        if stride == 0 {
            self.top = None;
        } else {
            debug_assert!(stride < 32, "Stride must be less than 32");
            // SAFETY: We ensure that stride is non-zero and within bounds
            self.left = unsafe { Some(NonZeroU8::new_unchecked(stride)) };
        }
        self
    }
    pub fn right(mut self, stride: u8) -> Self {
        if stride == 0 {
            self.top = None;
        } else {
            debug_assert!(stride < 32, "Stride must be less than 32");
            // SAFETY: We ensure that stride is non-zero and within bounds
            self.right = unsafe { Some(NonZeroU8::new_unchecked(stride)) };
        }
        self
    }
    pub fn top(mut self, stride: u8) -> Self {
        if stride == 0 {
            self.top = None;
        } else {
            debug_assert!(stride < 32, "Stride must be less than 32");
            // SAFETY: We ensure that stride is non-zero and within bounds
            self.top = unsafe { Some(NonZeroU8::new_unchecked(stride)) };
        }
        self
    }
    pub fn bottom(mut self, stride: u8) -> Self {
        if stride == 0 {
            self.top = None;
        } else {
            debug_assert!(stride < 32, "Stride must be less than 32");
            // SAFETY: We ensure that stride is non-zero and within bounds
            self.bottom = unsafe { Some(NonZeroU8::new_unchecked(stride)) };
        }
        self
    }
}

#[derive(Default, Clone, Copy, Debug, bevy::render::render_resource::ShaderType)]
pub struct BlockOverrides {
    block_a: u32,
    block_b: u32,
    block_c: u32,
    block_d: u32,
}

impl Material for DebugMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        SHADER_PATH.into()
        // "shaders/voxel.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        SHADER_PATH.into()
        // "shaders/voxel.wgsl".into()
    }

    // fn prepass_vertex_shader() -> bevy::render::render_resource::ShaderRef {
    //     bevy::render::render_resource::ShaderRef::Default
    // }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[BLOCK_DATA.at_shader_location(0)])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
