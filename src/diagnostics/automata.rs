use bevy::{
    diagnostic::{
        Diagnostic, DiagnosticMeasurement, DiagnosticPath, Diagnostics, DiagnosticsStore,
        RegisterDiagnostic,
    },
    ecs::{spawn::SpawnIter, system::SystemId},
    prelude::*,
};

use crate::{
    diagnostics::{DiagnosticSettings, TabButton},
    voxels::{
        ChunkId, NeighbourDirection,
        cellular_automata::{CellData, Cells},
    },
};

fn reg_tab(mut settings: ResMut<DiagnosticSettings>, mut commands: Commands) {
    let on_open = commands.register_system(on_open);
    let on_close = commands.register_system(on_close);

    settings.register_tab("Auto", on_open, on_close);
}

#[derive(Resource)]
struct TabState {
    mode: CellMode,
}

impl FromWorld for TabState {
    fn from_world(world: &mut World) -> Self {
        TabState {
            mode: CellMode::OFF,
        }
    }
}

#[derive(Debug)]
enum ValidationState {
    Valid,
    Unknown,
    Invalid,
}
fn button_node() -> Node {
    Node {
        width: Val::Percent(100.),
        height: Val::Px(20.),
        flex_direction: FlexDirection::Row,
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..Node::DEFAULT
    }
}
fn on_open(In(content): In<Entity>, mut commands: Commands, state: Res<TabState>) {
    let buttons = [
        (
            TabButton::new(commands.register_system(|mut tab: ResMut<TabState>| {
                tab.mode ^= CellMode::TEMPERATURE;
            })),
            Text::new("Temp"),
            CellMode::TEMPERATURE,
        ),
        (
            TabButton::new(commands.register_system(|mut tab: ResMut<TabState>| {
                tab.mode ^= CellMode::PRESURE;
            })),
            Text::new("Pres"),
            CellMode::PRESURE,
        ),
        (
            TabButton::new(commands.register_system(|mut tab: ResMut<TabState>| {
                tab.mode ^= CellMode::DUMMY;
            })),
            Text::new("Dummy"),
            CellMode::DUMMY,
        ),
        (
            TabButton::new(commands.register_system(|mut tab: ResMut<TabState>| {
                tab.mode ^= CellMode::CHARGE;
            })),
            Text::new("Charge"),
            CellMode::CHARGE,
        ),
        (
            TabButton::new(commands.register_system(|mut tab: ResMut<TabState>| {
                tab.mode = CellMode::OFF;
            })),
            Text::new("Off"),
            CellMode::OFF,
        ),
    ];
    commands.entity(content).insert(Children::spawn(SpawnIter(
        buttons.into_iter().map(|c| (c, button_node())),
    )));
}

fn on_close(content: In<Entity>, mut commands: Commands) {
    commands.entity(*content).despawn_related::<Children>();
}

////
use bevy::{
    prelude::*,
    render::{render_resource::ShaderType, storage::ShaderStorageBuffer},
};
use phoxels::core::VoxelMaterial;

pub fn plugin(app: &mut App) {
    app.init_non_send_resource::<MaxValue>()
        .add_systems(
            FixedPostUpdate,
            (
                add_diagnostics.run_if(not_mode(CellMode::OFF)),
                update_diagnostics.run_if(any(CellMode::ALL - CellMode::PAUSE)),
                remove_diagnostics.run_if(stop_diagnostics),
            ),
        )
        .add_systems(FixedPostUpdate, update_max.before(update_diagnostics))
        .add_systems(Update, update_tab);

    app.add_systems(Startup, reg_tab)
        .init_resource::<TabState>();
}

fn mode(mode: CellMode) -> impl Fn(Res<TabState>) -> bool {
    move |state: Res<TabState>| state.mode == mode
}

fn not_mode(mode: CellMode) -> impl Fn(Res<TabState>) -> bool {
    move |state: Res<TabState>| state.mode != mode
}

fn any(modes: CellMode) -> impl Fn(Res<TabState>) -> bool {
    move |state: Res<TabState>| state.mode.intersects(modes)
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
                break;
            }
        }
    }
}

fn stop_diagnostics(state: Res<TabState>) -> bool {
    state.is_changed() && state.mode == CellMode::OFF
}

bitflags::bitflags! {
    /// Cell mode for diagnostics
    ///
    /// - `Off`: Diagnostics are off
    /// - `Temperature`: Show temperature diagnostics
    /// - `Presure`: Show pressure diagnostics
    /// - `Charge`: Show charge diagnostics
    /// - `All`: Show all diagnostics
    /// - `Pause`: Pause diagnostics
    /// - `Dummy`: Dummy mode for testing
    struct CellMode: u32 {
        const OFF = 0;
        const TEMPERATURE = 1 << 0;
        const PRESURE = 1 << 1;
        const CHARGE = 1 << 2;
        const ALL = Self::TEMPERATURE.bits | Self::PRESURE.bits | Self::CHARGE.bits;
        const PAUSE = 1 << 7;
        const DUMMY = 1 << (7 + 1);
    }
}

impl Component for CellMode {
    const STORAGE_TYPE: bevy::ecs::component::StorageType =
        bevy::ecs::component::StorageType::Table;

    type Mutability = bevy::ecs::component::Immutable;
}

use crate::{
    diagnostics::shader::DebugMaterial,
    utils::BlockIter,
    voxels::{
        cellular_automata::FixedNum,
        map::{CHUNK_ARIA, CHUNK_SIZE, CHUNK_VOL},
    },
};

fn update_tab(state: Res<TabState>, mut buttons: Query<(&mut TextColor, &CellMode)>) {
    if !state.is_changed() {
        return;
    }
    for (mut button, mode) in &mut buttons {
        if state.mode & *mode == *mode {
            button.0 = Color::linear_rgb(0., 1., 0.);
        } else {
            button.0 = Color::WHITE;
        }
    }
}

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
    chunks: Query<(&Cells, &MeshMaterial3d<DebugMaterial>), Changed<Cells>>,
    mut materials: ResMut<Assets<DebugMaterial>>,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
    state: Res<TabState>,
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
        material.settings = state.mode.bits;
        if state.mode == CellMode::DUMMY {
            buffer.set_data(dummy_diagnostics());
            continue;
        }
        buffer.set_data(extract_component(data, max.get_max()));
    }
}

pub fn add_diagnostics(
    chunks: Query<
        (Entity, &Cells, &MeshMaterial3d<VoxelMaterial>),
        (Without<MeshMaterial3d<DebugMaterial>>,),
    >,
    mut commands: Commands,
    mut materaials: ResMut<Assets<DebugMaterial>>,
    other: ResMut<Assets<VoxelMaterial>>,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
    mut local: Local<Handle<ShaderStorageBuffer>>,
    state: Res<TabState>,
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
                settings: state.mode.bits,
            })));
    }
}

const ONE_HUNDRED: FixedNum = FixedNum::lit("100.0");
const ONE: FixedNum = FixedNum::ONE;
const U8: FixedNum = FixedNum::lit("255.0");

fn extract_component(item: &Cells, max: CellData) -> AutomitaDiagnosticChunk {
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
    let mut chunk = Cells::solid(CellData::THE_VOID);
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
