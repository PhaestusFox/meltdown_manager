use bevy::{
    prelude::*,
    render::{
        render_resource::ShaderType,
        storage::ShaderStorageBuffer,
    },
};
use phoxels::core::VoxelMaterial;

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (
            add_diagnostics.run_if(run_diagnostics),
            update_diagnostics.run_if(run_diagnostics),
            remove_diagnostics.run_if(stop_diagnostics),
        ),
    );
}

fn run_diagnostics(settings: Res<crate::diagnostics::DiagnosticSettings>) -> bool {
    settings.enabled && settings.cell_mode != CellMode::Off && settings.cell_mode != CellMode::Pause
}

fn stop_diagnostics(settings: Res<crate::diagnostics::DiagnosticSettings>) -> bool {
    settings.is_changed() && !settings.enabled
}

#[derive(PartialEq, Eq)]
pub enum CellMode {
    Off = 0,
    Temperature = 1 << 0,
    Presure = 1 << 1,
    Charge = 1 << 2,
    All = 7,
    Pause = 8,
}

use crate::{
    diagnostics::shader::AutomitaDiagnosticMateraial,
    diagnostics::shader::AutomitaMateraial,
    voxels::{CHUNK_VOL, Chunk},
};

use super::CellData;

#[derive(Component, Clone, ShaderType)]
pub struct AutomitaDiagnosticChunk {
    blocks: [Data; CHUNK_VOL / 4],
}

#[derive(Clone, ShaderType)]
struct Data {
    a: u32,
    b: u32,
    c: u32,
    d: u32,
}

impl Data {
    const ZERO: Data = Data {
        a: 0,
        b: 0,
        c: 0,
        d: 0,
    };

    fn new(data: [u32; 4]) -> Data {
        Data {
            a: data[0],
            b: data[1],
            c: data[2],
            d: data[3],
        }
    }
}

pub fn update_diagnostics(
    chunks: Query<
        (
            &Chunk<CellData>,
            &MeshMaterial3d<AutomitaDiagnosticMateraial>,
        ),
        Changed<Chunk<CellData>>,
    >,
    materials: Res<Assets<AutomitaDiagnosticMateraial>>,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (data, MeshMaterial3d(material)) in &chunks {
        let Some(material) = materials.get(material.id()) else {
            continue;
        };
        let Some(buffer) = storage_buffers.get_mut(material.extension.data.id()) else {
            warn!("failed to get buffer");
            continue;
        };
        buffer.set_data(extract_component(data));
    }
}

pub fn add_diagnostics(
    chunks: Query<
        (Entity, &Chunk<CellData>, &MeshMaterial3d<VoxelMaterial>),
        (Without<MeshMaterial3d<AutomitaDiagnosticMateraial>>,),
    >,
    mut commands: Commands,
    mut materaials: ResMut<Assets<AutomitaDiagnosticMateraial>>,
    other: ResMut<Assets<VoxelMaterial>>,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
) {
    for (entity, data, base) in &chunks {
        let comp = extract_component(data);
        let mut buffer = ShaderStorageBuffer::default();
        buffer.set_data(comp);
        commands.entity(entity).insert(MeshMaterial3d(
            materaials.add(AutomitaDiagnosticMateraial {
                base: other
                    .get(base.0.id())
                    .expect("Voxel shader to be loaded")
                    .clone(),
                extension: AutomitaMateraial {
                    data: storage_buffers.add(buffer),
                },
            }),
        ));
    }
}

fn extract_component(item: &Chunk<CellData>) -> AutomitaDiagnosticChunk {
    let mut chunk = AutomitaDiagnosticChunk {
        blocks: [Data::ZERO; CHUNK_VOL / 4],
        // blocks: 0.,
    };
    let mut min = CellData::MAX;
    let mut max = CellData::MIN;
    for cell in item.blocks() {
        min.min(&cell);
        max.max(&cell);
    }
    let range = max - min;
    if range.any_zero() {
        return chunk;
    }
    for i in (0..CHUNK_VOL).step_by(4) {
        let mut normalized_0 = (item.get_by_index(i) - min) / range;
        let mut normalized_1 = (item.get_by_index(i + 1) - min) / range;
        let mut normalized_2 = (item.get_by_index(i + 2) - min) / range;
        let mut normalized_3 = (item.get_by_index(i + 3) - min) / range;
        normalized_0 *= 255;
        normalized_1 *= 255;
        normalized_2 *= 255;
        normalized_3 *= 255;
        let t0: u8 = normalized_0.temperature.to_num();
        let p0: u8 = normalized_0.presure.to_num();
        let c0: u8 = normalized_0.charge.to_num();
        let t1: u8 = normalized_1.temperature.to_num();
        let p1: u8 = normalized_1.presure.to_num();
        let c1: u8 = normalized_1.charge.to_num();
        let t2: u8 = normalized_2.temperature.to_num();
        let p2: u8 = normalized_2.presure.to_num();
        let c2: u8 = normalized_2.charge.to_num();
        let t3: u8 = normalized_3.temperature.to_num();
        let p3: u8 = normalized_3.presure.to_num();
        let c3: u8 = normalized_3.charge.to_num();
        chunk.blocks[i] = Data::new([
            t0 as u32 | (p0 as u32) << 8 | (c0 as u32) << 16,
            t1 as u32 | (p1 as u32) << 8 | (c1 as u32) << 16,
            t2 as u32 | (p2 as u32) << 8 | (c2 as u32) << 16,
            t3 as u32 | (p3 as u32) << 8 | (c3 as u32) << 16,
        ])
    }
    chunk
}

fn remove_diagnostics(
    chunks: Query<Entity, With<MeshMaterial3d<AutomitaDiagnosticMateraial>>>,
    mut commands: Commands,
) {
    for entity in &chunks {
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<AutomitaDiagnosticMateraial>>();
    }
}
