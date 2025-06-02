use bevy::{
    prelude::*,
    render::{render_resource::ShaderType, storage::ShaderStorageBuffer},
    text::cosmic_text::ttf_parser::loca,
};
use phoxels::core::VoxelMaterial;

pub fn plugin(app: &mut App) {
    app.add_systems(
        FixedPostUpdate,
        (
            add_diagnostics.run_if(run_diagnostics),
            update_diagnostics.run_if(run_diagnostics),
            remove_diagnostics.run_if(stop_diagnostics),
        ),
    )
    .init_non_send_resource::<MaxValue>()
    .add_systems(FixedPostUpdate, update_max.before(update_diagnostics));
}

fn update_max(mut max: NonSendMut<MaxValue>) {
    max.restart();
    max.run();
}

pub struct MaxValue {
    max: CellData,
    channel: std::sync::mpsc::Receiver<CellData>,
    sender: std::sync::mpsc::Sender<CellData>,
}

impl FromWorld for MaxValue {
    fn from_world(_: &mut World) -> Self {
        let (sender, channel) = std::sync::mpsc::channel();
        MaxValue {
            max: CellData {
                temperature: FixedNum::ONE,
                presure: FixedNum::ONE,
                charge: FixedNum::ONE,
            },
            channel,
            sender,
        }
    }
}

impl MaxValue {
    pub fn get_sender(&self) -> std::sync::mpsc::Sender<CellData> {
        self.sender.clone()
    }

    fn get_max(&self) -> CellData {
        self.max
    }

    fn restart(&mut self) {
        self.max = CellData {
            temperature: FixedNum::ONE,
            presure: FixedNum::ONE,
            charge: FixedNum::ONE,
        };
    }

    fn run(&mut self) {
        loop {
            if let Ok(data) = self.channel.try_recv() {
                self.max.max(&data);
            } else {
                println!("Max diagnostics: {:?}", self.max);
                break;
            }
        }
    }
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
    diagnostics::shader::DebugMaterial,
    utils::BlockIter,
    voxels::{CHUNK_ARIA, CHUNK_SIZE, CHUNK_VOL, Chunk, ChunkId, cellular_automata::FixedNum},
};

use super::CellData;

#[derive(Component, Clone, ShaderType, Debug)]
pub struct AutomitaDiagnosticChunk {
    blocks: [Data; CHUNK_VOL / 4],
}

#[derive(Clone, ShaderType, Debug)]
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
    chunks: Query<(&Chunk<CellData>, &MeshMaterial3d<DebugMaterial>), Changed<Chunk<CellData>>>,
    mut materials: ResMut<Assets<DebugMaterial>>,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
    max: NonSend<MaxValue>,
) {
    for (data, MeshMaterial3d(material)) in &chunks {
        let Some(material) = materials.get_mut(material.id()) else {
            warn!("failed to get material");
            continue;
        };
        let Some(buffer) = storage_buffers.get_mut(&material.data) else {
            warn!("failed to get storage buffer");
            continue;
        };
        buffer.set_data(extract_component(data, max.get_max()));
    }
}

pub fn add_diagnostics(
    chunks: Query<
        (Entity, &Chunk<CellData>, &MeshMaterial3d<VoxelMaterial>),
        (Without<MeshMaterial3d<DebugMaterial>>,),
    >,
    mut commands: Commands,
    mut materaials: ResMut<Assets<DebugMaterial>>,
    other: ResMut<Assets<VoxelMaterial>>,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut local: Local<Handle<ShaderStorageBuffer>>,
) {
    for (entity, data, base) in &chunks {
        // let comp = extract_component(data);
        let comp = dummy_diagnostics();
        let mut buffer = ShaderStorageBuffer::default();
        buffer.set_data(comp);
        let base = other
            .get(base.0.id())
            .expect("Voxel shader to be loaded")
            .clone();
        commands
            .entity(entity)
            .insert(MeshMaterial3d(materaials.add(DebugMaterial {
                atlas_shape: base.atlas_shape,
                base_color_texture: base.base_color_texture,
                alpha_mode: base.alpha_mode,
                overrides: base.overrides,
                data: storage_buffers.add(ShaderStorageBuffer::from(dummy_diagnostics())),
            })));
    }
}

const ONE_HUNDRED: FixedNum = FixedNum::lit("100.0");
const ONE: FixedNum = FixedNum::ONE;
const U8: FixedNum = FixedNum::lit("255.0");

fn extract_component(item: &Chunk<CellData>, max: CellData) -> AutomitaDiagnosticChunk {
    let mut chunk = AutomitaDiagnosticChunk {
        blocks: [Data::ZERO; CHUNK_VOL / 4],
        // blocks: 0.,
    };
    for i in (0..CHUNK_VOL).step_by(4) {
        let mut normalized_0 = item.get_by_index(i).normalize(max);
        let mut normalized_1 = item.get_by_index(i + 1).normalize(max);
        let mut normalized_2 = item.get_by_index(i + 2).normalize(max);
        let mut normalized_3 = item.get_by_index(i + 3).normalize(max);
        normalized_0 *= ONE_HUNDRED;
        normalized_1 *= ONE_HUNDRED;
        normalized_2 *= ONE_HUNDRED;
        normalized_3 *= ONE_HUNDRED;
        normalized_0.clamp(ONE, U8);
        normalized_1.clamp(ONE, U8);
        normalized_2.clamp(ONE, U8);
        normalized_3.clamp(ONE, U8);

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
        chunk.blocks[i / 4] = Data::new([
            t0 as u32 | (p0 as u32) << 8 | (c0 as u32) << 16,
            t1 as u32 | (p1 as u32) << 8 | (c1 as u32) << 16,
            t2 as u32 | (p2 as u32) << 8 | (c2 as u32) << 16,
            t3 as u32 | (p3 as u32) << 8 | (c3 as u32) << 16,
        ]);
        // if i > CHUNK_SIZE + CHUNK_ARIA + 1 && i < CHUNK_SIZE + CHUNK_ARIA + 10 {
        //     info!("Diagnostics for chunk: {:?}", item.get_by_index(i));
        //     info!("min: {:?}, max: {:?}", min, max);
        //     info!("range: {:?}", range);
        //     info!("normalized: {:?}", normalized_0);
        //     info!("temperature: {}, presure: {}, charge: {}", t0, p0, c0);
        //     info!("packed: {:032b}", chunk.blocks[i / 4].a);
        // }
    }
    chunk
}

fn remove_diagnostics(
    chunks: Query<Entity, With<MeshMaterial3d<DebugMaterial>>>,
    mut commands: Commands,
) {
    for entity in &chunks {
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<DebugMaterial>>();
    }
}

fn dummy_diagnostics() -> AutomitaDiagnosticChunk {
    let mut chunk = Chunk::<CellData>::solid(CellData::THE_VOID);
    for (x, y, z) in BlockIter::<30, 30, 30>::new() {
        chunk.set_block(
            x,
            y,
            z,
            CellData {
                temperature: FixedNum::from_num(8 * x),
                charge: FixedNum::from_num(8 * y),
                presure: FixedNum::from_num(8 * z),
            },
        );
    }
    extract_component(&chunk, CellData::default())
}
