use bevy::{
    diagnostic::{
        Diagnostic, DiagnosticMeasurement, DiagnosticPath, Diagnostics, DiagnosticsStore,
        RegisterDiagnostic,
    },
    ecs::{
        entity::EntityIndexSet,
        spawn::{SpawnIter, SpawnWith},
        system::SystemId,
    },
    platform::collections::HashSet,
    prelude::*,
};

use crate::{
    diagnostics::{DiagnosticSettings, TabButton},
    voxels::{
        ChunkId, NeighbourDirection,
        cellular_automata::{CellData, CellFlags, Cells},
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
    to_update_local: (UpdateMode, UpdateRange),
    to_update_global: (UpdateMode, UpdateRange),
}

impl FromWorld for TabState {
    fn from_world(_: &mut World) -> Self {
        TabState {
            mode: CellMode::OFF,
            to_update_local: (UpdateMode::EveryTick, UpdateRange::All),
            to_update_global: (UpdateMode::EverySecond, UpdateRange::All),
        }
    }
}

fn button_bundle(is_on: bool) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.),
            height: Val::Px(20.),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..Node::DEFAULT
        },
        if is_on {
            BackgroundColor(Color::linear_rgb(0.2, 0.5, 0.2))
        } else {
            BackgroundColor(Color::linear_rgb(0.4, 0.4, 0.4))
        },
    )
}

fn button_test(is_on: bool, grow: f32) -> impl Bundle {
    (
        Node {
            height: Val::Px(20.),
            flex_grow: grow,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..Node::DEFAULT
        },
        if is_on {
            BackgroundColor(Color::linear_rgb(0.2, 0.5, 0.2))
        } else {
            BackgroundColor(Color::linear_rgb(0.4, 0.4, 0.4))
        },
    )
}

fn on_open(In(content): In<Entity>, mut commands: Commands, state: Res<TabState>) {
    let buttons = [
        CellMode::OFF,
        CellMode::TEMPERATURE,
        CellMode::PRESURE,
        CellMode::CHARGE,
        CellMode::DUMMY,
        CellMode::PAUSE,
        CellMode::PHASE,
    ];
    let buttons = buttons
        .iter()
        .map(|&mode| {
            let mut string = String::new();
            let on_click = if mode.is_empty() {
                string.push_str("OFF");
                commands.register_system(move |mut tab: ResMut<TabState>| {
                    tab.mode = CellMode::OFF;
                })
            } else {
                bitflags::parser::to_writer(&mode, &mut string).unwrap();
                commands.register_system(move |mut tab: ResMut<TabState>| {
                    tab.mode.toggle(mode);
                })
            };
            (
                button_bundle(mode.is_empty()),
                TabButton::new(on_click),
                Text::new(string),
                mode,
            )
        })
        .collect::<Vec<_>>();

    let modes = [
        UpdateMode::EveryTick,
        UpdateMode::EverySecond,
        UpdateMode::ChunkChange,
        UpdateMode::Manualy,
    ]
    .map(|mode| {
        let mut string = String::new();
        string.push_str(mode.short_name());
        let on_click = commands.register_system(move |mut tab: ResMut<TabState>| {
            tab.to_update_local.0 = mode;
        });
        (
            button_test(state.to_update_local.0 == mode, 1. / 4.),
            TabButton::new(on_click),
            Text::new(mode.short_name()),
        )
    });

    let range = [
        UpdateRange::All,
        UpdateRange::MaxDistance(2),
        UpdateRange::Adjacent,
    ]
    .map(|mode| {
        let mut string = String::new();
        string.push_str(mode.short_name());
        let on_click = commands.register_system(move |mut tab: ResMut<TabState>| {
            tab.to_update_local.1 = mode;
        });
        (
            button_test(state.to_update_local.1 == mode, 1. / 3.),
            TabButton::new(on_click),
            Text::new(mode.short_name()),
        )
    });

    let g_modes = [
        UpdateMode::EverySecond,
        UpdateMode::ChunkChange,
        UpdateMode::Manualy,
    ]
    .map(|mode| {
        let mut string = String::new();
        string.push_str(mode.short_name());
        let on_click = commands.register_system(move |mut tab: ResMut<TabState>| {
            tab.to_update_global.0 = mode;
        });
        (
            button_test(state.to_update_global.0 == mode, 1. / 4.),
            TabButton::new(on_click),
            Text::new(mode.short_name()),
        )
    });

    let g_range = [
        UpdateRange::All,
        UpdateRange::MaxDistance(2),
        UpdateRange::Adjacent,
    ]
    .map(|mode| {
        let mut string = String::new();
        string.push_str(mode.short_name());
        let on_click = commands.register_system(move |mut tab: ResMut<TabState>| {
            tab.to_update_global.1 = mode;
        });
        (
            button_test(state.to_update_global.1 == mode, 1. / 3.),
            TabButton::new(on_click),
            Text::new(mode.short_name()),
        )
    });

    let on_change_local =
        commands.register_system(move |In(value): In<f32>, mut tab: ResMut<TabState>| {
            let UpdateRange::MaxDistance(max) = &mut tab.to_update_local.1 else {
                warn!("Local Range is not MaxDistance, cannot change range");
                return;
            };

            *max = value as u32;
        });
    let on_change_global =
        commands.register_system(move |In(value): In<f32>, mut tab: ResMut<TabState>| {
            let UpdateRange::MaxDistance(max) = &mut tab.to_update_global.1 else {
                warn!("Global Range is not MaxDistance, cannot change range");
                return;
            };

            *max = value as u32;
        });

    let mut tab = commands.entity(content);

    // spawn mode buttons
    tab.insert(Children::spawn(SpawnIter(buttons.into_iter())));

    // spawn range options
    tab.with_child((
        Node {
            height: Val::Px(18.),
            flex_grow: 1.,
            ..Node::DEFAULT
        },
        Text::new("Update Settings:"),
    ));

    let range_bar = matches!(state.to_update_local.1, UpdateRange::MaxDistance(_));
    tab.with_child((
        RangeTab,
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 0.45,
            ..Default::default()
        },
        Children::spawn(SpawnWith(
            move |tab: &mut bevy::ecs::relationship::RelatedSpawner<'_, ChildOf>| {
                tab.spawn((
                    Node {
                        flex_grow: 1.,
                        height: Val::Px(20.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..Node::DEFAULT
                    },
                    Children::spawn((Spawn(Text::new("Mode:")), SpawnIter(modes.into_iter()))),
                ));

                // spawn local update range options
                tab.spawn((
                    Node {
                        flex_grow: 1.,
                        height: Val::Px(20.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..Node::DEFAULT
                    },
                    Children::spawn((Spawn(Text::new("Range:")), SpawnIter(range.into_iter()))),
                ));
                // sapwn range slider
                // visibile only if in range mode
                tab.spawn((
                    if range_bar {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                    Node {
                        height: Val::Px(20.),
                        flex_grow: 1.,
                        ..Default::default()
                    },
                    super::Slider {
                        on_change: on_change_local,
                        min: 2.,
                        max: 25.,
                    },
                    BackgroundColor(Color::linear_rgb(0.1, 0.1, 0.1)),
                ));
            },
        )),
    ));

    let range_bar = matches!(state.to_update_global.1, UpdateRange::MaxDistance(_));

    tab.with_child((
        if state.to_update_local.1 != UpdateRange::All {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        },
        GlobalRangeTab,
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 0.45,
            ..Default::default()
        },
        Children::spawn(SpawnWith(
            move |tab: &mut bevy::ecs::relationship::RelatedSpawner<'_, ChildOf>| {
                tab.spawn((
                    Node {
                        flex_grow: 1.,
                        height: Val::Px(18.),
                        ..Node::DEFAULT
                    },
                    Text::new("Course Settings:"),
                ));
                // spawn global update mode buttons
                tab.spawn((
                    Node {
                        flex_grow: 1.,
                        height: Val::Px(20.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..Node::DEFAULT
                    },
                    Children::spawn((Spawn(Text::new("Mode:")), SpawnIter(g_modes.into_iter()))),
                ));
                // spawn local update range options
                tab.spawn((
                    Node {
                        flex_grow: 1.,
                        height: Val::Px(20.),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        ..Node::DEFAULT
                    },
                    Children::spawn((Spawn(Text::new("Range:")), SpawnIter(g_range.into_iter()))),
                ));
                // sapwn range slider
                // visibile only if in range mode
                tab.spawn((
                    if range_bar {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    },
                    Node {
                        height: Val::Px(20.),
                        flex_grow: 1.,
                        ..Default::default()
                    },
                    super::Slider {
                        on_change: on_change_global,
                        min: 2.,
                        max: 25.,
                    },
                    BackgroundColor(Color::linear_rgb(0.1, 0.1, 0.1)),
                ));
            },
        )),
    ));
}

#[derive(Component)]
struct RangeTab;

#[derive(Component)]
struct GlobalRangeTab;

fn on_close(content: In<Entity>, mut commands: Commands) {
    commands.entity(*content).despawn_related::<Children>();
}

use bevy::render::{render_resource::ShaderType, storage::ShaderStorageBuffer};

use phoxels::core::VoxelMaterial;

pub fn plugin(app: &mut App) {
    app.init_non_send_resource::<MaxValue>()
        .add_systems(
            FixedPostUpdate,
            (
                add_diagnostics.run_if(run_add),
                calculate_to_update.run_if(run_update),
                remove_diagnostics.run_if(stop_diagnostics),
            ),
        )
        .add_systems(Update, update_diagnostics)
        .add_systems(FixedPostUpdate, update_max)
        .add_systems(Update, update_tab)
        .init_resource::<ToUpdate>();

    app.add_systems(Startup, reg_tab)
        .init_resource::<TabState>();
}

fn run_add(state: Res<TabState>) -> bool {
    state.mode.bits() != 0
}

fn run_update(state: Res<TabState>) -> bool {
    state
        .mode
        .intersects(CellMode::ALL | CellMode::DUMMY | CellMode::PHASE)
        && !state.mode.intersects(CellMode::PAUSE)
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
                block: crate::voxels::blocks::Blocks::Void,
                energy: FixedNum::ONE,
                presure: FixedNum::ONE,
                charge: FixedNum::ONE,
                flags: CellFlags::empty(),
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
            block: crate::voxels::blocks::Blocks::Void,
            energy: FixedNum::ONE,
            presure: FixedNum::ONE,
            charge: FixedNum::ONE,
            flags: CellFlags::empty(),
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
    state.is_changed() && state.mode.bits() == 0
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
        const ALL = Self::TEMPERATURE.bits() | Self::PRESURE.bits() | Self::CHARGE.bits();
        const PAUSE = 1 << 7;
        const DUMMY = 1 << 8;
        const PHASE = 1 << 9;
    }
}

impl Clone for CellMode {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for CellMode {}

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

fn update_tab(
    state: Res<TabState>,
    mut modes: Query<(&mut BackgroundColor, &CellMode)>,
    local: Single<&Children, With<RangeTab>>,
    mut global: Single<(&mut Visibility, &Children), With<GlobalRangeTab>>,
    children: Query<&Children>,
    mut buttons: Query<
        (&mut BackgroundColor, &mut Visibility),
        (Without<CellMode>, Without<GlobalRangeTab>),
    >,
) {
    if !state.is_changed() {
        return;
    }
    for (mut button, mode) in &mut modes {
        if state.mode.intersects(*mode) || mode.is_empty() && state.mode.is_empty() {
            button.0 = Color::linear_rgb(0.2, 0.5, 0.2);
        } else {
            button.0 = Color::linear_rgb(0.4, 0.4, 0.4);
        }
    }
    if state.to_update_local.1 != UpdateRange::All {
        *global.0 = Visibility::Inherited;
    } else {
        *global.0 = Visibility::Hidden;
    }
    for i in 0..3 {
        match i {
            0 => {
                // clear buttons color
                let g_children = children.get(global.1[1]).unwrap();
                let children = children.get(local[0]).unwrap();
                for child in g_children.iter().skip(1).chain(children.iter().skip(1)) {
                    if let Ok((mut color, _)) = buttons.get_mut(child) {
                        color.0 = Color::linear_rgb(0.4, 0.4, 0.4);
                    }
                }
                if let Some(child) = g_children.get(state.to_update_global.0.index()) {
                    if let Ok((mut color, _)) = buttons.get_mut(*child) {
                        color.0 = Color::linear_rgb(0.2, 0.5, 0.2);
                    } else {
                        warn!("Failed to get button color for global mode");
                    }
                }
                if let Some(child) = children.get(state.to_update_local.0.index() + 1) {
                    if let Ok((mut color, _)) = buttons.get_mut(*child) {
                        color.0 = Color::linear_rgb(0.2, 0.5, 0.2);
                    } else {
                        warn!("Failed to get button color for local mode");
                    }
                }
            }
            1 => {
                // clear buttons color
                let g_children = children.get(global.1[2]).unwrap();
                let children = children.get(local[1]).unwrap();
                for child in g_children.iter().skip(1).chain(children.iter().skip(1)) {
                    if let Ok((mut color, _)) = buttons.get_mut(child) {
                        color.0 = Color::linear_rgb(0.4, 0.4, 0.4);
                    }
                }
                if let Some(child) = g_children.get(state.to_update_global.1.index() + 1) {
                    if let Ok((mut color, _)) = buttons.get_mut(*child) {
                        color.0 = Color::linear_rgb(0.2, 0.5, 0.2);
                    } else {
                        warn!("Failed to get button color for global range");
                    }
                }
                if let Some(child) = children.get(state.to_update_local.1.index() + 1) {
                    if let Ok((mut color, _)) = buttons.get_mut(*child) {
                        color.0 = Color::linear_rgb(0.2, 0.5, 0.2);
                    } else {
                        warn!("Failed to get button color for local range");
                    }
                }
            }
            2 => {
                // clear buttons color
                if let Ok((_, mut vis)) = buttons.get_mut(global.1[3]) {
                    *vis = if let UpdateRange::MaxDistance(_) = state.to_update_global.1 {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                }
                if let Ok((_, mut vis)) = buttons.get_mut(local[2]) {
                    *vis = if let UpdateRange::MaxDistance(_) = state.to_update_local.1 {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                }
            }
            _ => unreachable!(),
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

#[derive(Resource, Default)]
struct ToUpdate {
    entitys: EntityIndexSet,
    per_frame: usize,
}

impl ToUpdate {
    fn insert(&mut self, entity: Entity) {
        self.entitys.insert(entity);
    }

    fn get_frame(&mut self) -> Option<Vec<Entity>> {
        if self.entitys.is_empty() {
            return None;
        }
        let mut result = Vec::with_capacity(self.per_frame);
        for _ in 0..self.per_frame.min(self.entitys.len()) {
            result.push(self.entitys.pop().expect("not Empty"));
        }
        Some(result)
    }
    fn set_frames(&mut self, frames: u32) {
        if frames == 0 {
            warn!("Frames set to 0, setting to 1");
            self.per_frame = 1;
            return;
        }
        self.per_frame = self.entitys.len() / frames as usize;
    }

    fn clear(&mut self) {
        self.entitys.clear();
    }

    fn sort(&mut self, chunks: Query<(Entity, &ChunkId)>, location: ChunkId) {
        self.entitys.sort_by(|a, b| {
            let a_id = chunks.get(*a).map_or(ChunkId::ZERO, |(_, id)| *id);
            let b_id = chunks.get(*b).map_or(ChunkId::ZERO, |(_, id)| *id);
            b_id.manhattan_distance(&location)
                .cmp(&a_id.manhattan_distance(&location))
        });
    }
}

fn calculate_to_update(
    chunks: Query<(Entity, &ChunkId)>,
    player: Single<&Transform, With<crate::player::Player>>,
    mut last: Local<(ChunkId, u32)>,
    time: Res<Time>,
    input: Res<ButtonInput<KeyCode>>,
    state: Res<TabState>,
    mut to_update: ResMut<ToUpdate>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let location = ChunkId::from_translation(player.translation);
    to_update.clear();
    let fps = diagnostics
        .get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FPS)
        .map(|f| f.average().map(|v| v as u32).unwrap_or(30))
        .unwrap_or(30);
    if fps < 15 {
        warn!("FPS is too low: {fps} skipping adding chunks to update");
        return;
    }

    if state.mode.intersects(CellMode::PAUSE) {
        return;
    }
    let is_seconed = {
        let l = last.1;
        let t = time.elapsed_secs() as u32;
        last.1 = t;
        l != t
    };
    match state.to_update_local.0 {
        UpdateMode::EveryTick => {}
        UpdateMode::EverySecond => {
            if !is_seconed {
                return;
            }
        }
        UpdateMode::ChunkChange => {
            if last.0 == location {
                return;
            }
            last.0 = location;
        }
        UpdateMode::Manualy => {
            if !input.just_pressed(KeyCode::F5) {
                return;
            }
        }
    }

    match state.to_update_local.1 {
        UpdateRange::All => {
            for (id, ..) in &chunks {
                to_update.insert(id);
            }
        }
        UpdateRange::MaxDistance(distance) => {
            for (e, id, ..) in &chunks {
                if location.manhattan_distance(id) <= distance {
                    to_update.insert(e);
                }
            }
        }
        UpdateRange::Adjacent => {
            for (e, id, ..) in &chunks {
                if location.manhattan_distance(id) <= 1 {
                    to_update.insert(e);
                }
            }
        }
    }

    if state.to_update_local.1 != UpdateRange::All {
        let update_global = match state.to_update_global.0 {
            UpdateMode::EveryTick => {
                warn!(
                    r#"    Global Update::EveryTick is redundant
    it is only checked after Local::Mode has passed
    AnyRange here would by necessary be Eq or more inclusive than Local::EveryTick"#
                );
                false
            }
            UpdateMode::EverySecond => is_seconed,
            UpdateMode::ChunkChange => last.0 != location,
            UpdateMode::Manualy => input.just_pressed(KeyCode::F5),
        };

        if update_global {
            match state.to_update_global.1 {
                UpdateRange::All => {
                    for (e, ..) in &chunks {
                        to_update.insert(e);
                    }
                }
                UpdateRange::MaxDistance(distance) => {
                    for (e, id, ..) in &chunks {
                        if location.manhattan_distance(id) <= distance {
                            to_update.insert(e);
                        }
                    }
                }
                UpdateRange::Adjacent => {
                    for (e, id, ..) in &chunks {
                        if location.manhattan_distance(id) <= 1 {
                            to_update.insert(e);
                        }
                    }
                }
            }
        }
    }

    // works out the expected number of frames per tick; we spread the updating over 66% of the frames
    to_update.set_frames(fps / 15);

    to_update.sort(chunks, location);
}

fn update_diagnostics(
    chunks: Query<(&Cells, &MeshMaterial3d<DebugMaterial>)>,
    mut materials: ResMut<Assets<DebugMaterial>>,
    mut storage_buffers: ResMut<Assets<ShaderStorageBuffer>>,
    state: Res<TabState>,
    max: NonSend<MaxValue>,
    mut to_update: ResMut<ToUpdate>,
) {
    let Some(to_update) = to_update.get_frame() else {
        return;
    };
    for e in to_update {
        let Ok((data, material)) = chunks.get(e) else {
            warn!("failed to get chunk data");
            continue;
        };
        let Some(material) = materials.get_mut(material.id()) else {
            warn!("failed to get material");
            continue;
        };
        let Some(buffer) = storage_buffers.get_mut(&material.data) else {
            warn!("failed to get storage buffer");
            continue;
        };
        material.settings = state.mode.bits();
        if state.mode.intersects(CellMode::DUMMY) {
            if state.is_changed() {
                buffer.set_data(dummy_diagnostics());
            }
            continue;
        }
        buffer.set_data(extract_component(data, max.get_max()));
    }
}

fn add_diagnostics(
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
                settings: state.mode.bits(),
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
        let item0 = item.get_by_index(i);
        let item1 = item.get_by_index(i + 1);
        let item2 = item.get_by_index(i + 2);
        let item3 = item.get_by_index(i + 3);

        let mut tt0 = item0.temperature() / FixedNum::from_num(2);
        let mut tt1 = item1.temperature() / FixedNum::from_num(2);
        let mut tt2 = item2.temperature() / FixedNum::from_num(2);
        let mut tt3 = item3.temperature() / FixedNum::from_num(2);
        let mut normalized_0 = item0.normalize(max);
        let mut normalized_1 = item1.normalize(max);
        let mut normalized_2 = item2.normalize(max);
        let mut normalized_3 = item3.normalize(max);
        normalized_0 *= ONE_HUNDRED;
        normalized_1 *= ONE_HUNDRED;
        normalized_2 *= ONE_HUNDRED;
        normalized_3 *= ONE_HUNDRED;
        normalized_0.clamp(ONE, U8);
        tt0 = tt0.clamp(FixedNum::ZERO, U8);
        normalized_1.clamp(ONE, U8);
        tt1 = tt1.clamp(FixedNum::ZERO, U8);
        normalized_2.clamp(ONE, U8);
        tt2 = tt2.clamp(FixedNum::ZERO, U8);
        normalized_3.clamp(ONE, U8);
        tt3 = tt3.clamp(FixedNum::ZERO, U8);

        let t0: u8 = tt0.to_num();
        let p0: u8 = normalized_0.presure.to_num();
        let c0: u8 = normalized_0.charge.to_num();
        let t1: u8 = tt1.to_num();
        let p1: u8 = normalized_1.presure.to_num();
        let c1: u8 = normalized_1.charge.to_num();
        let t2: u8 = tt2.to_num();
        let p2: u8 = normalized_2.presure.to_num();
        let c2: u8 = normalized_2.charge.to_num();
        let t3: u8 = tt3.to_num();
        let p3: u8 = normalized_3.presure.to_num();
        let c3: u8 = normalized_3.charge.to_num();
        chunk.blocks[i / 4] = Data::new([
            t0 as u32 | (p0 as u32) << 8 | (c0 as u32) << 16 | (item0.flags.bits() as u32) << 24,
            t1 as u32 | (p1 as u32) << 8 | (c1 as u32) << 16 | (item1.flags.bits() as u32) << 24,
            t2 as u32 | (p2 as u32) << 8 | (c2 as u32) << 16 | (item2.flags.bits() as u32) << 24,
            t3 as u32 | (p3 as u32) << 8 | (c3 as u32) << 16 | (item3.flags.bits() as u32) << 24,
        ]);
    }
    chunk
}

fn remove_diagnostics(
    chunks: Query<Entity, With<MeshMaterial3d<DebugMaterial>>>,
    mut force_update: Query<
        &mut MeshMaterial3d<VoxelMaterial>,
        With<MeshMaterial3d<DebugMaterial>>,
    >,
    mut commands: Commands,
) {
    for entity in &chunks {
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<DebugMaterial>>();
    }
    for mut force in &mut force_update {
        force.set_changed();
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
                block: crate::voxels::blocks::Blocks::Void,
                energy: FixedNum::from_num(8 * x),
                charge: FixedNum::from_num(8 * y),
                presure: FixedNum::from_num(8 * z),
                flags: CellFlags::empty(),
            },
        );
    }
    extract_component(&chunk, CellData::all(U8))
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum UpdateMode {
    EveryTick,
    EverySecond,
    ChunkChange,
    Manualy,
}

impl UpdateMode {
    fn index(&self) -> usize {
        match self {
            UpdateMode::EveryTick => 0,
            UpdateMode::EverySecond => 1,
            UpdateMode::ChunkChange => 2,
            UpdateMode::Manualy => 3,
        }
    }

    fn short_name(&self) -> &'static str {
        match self {
            UpdateMode::EveryTick => "Tick",
            UpdateMode::EverySecond => "Second",
            UpdateMode::ChunkChange => "Move",
            UpdateMode::Manualy => "Manual",
        }
    }
}

#[derive(Debug, Eq, Clone, Copy)]
enum UpdateRange {
    All,
    MaxDistance(u32),
    Adjacent,
}

impl PartialEq for UpdateRange {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl UpdateRange {
    fn short_name(&self) -> &'static str {
        match self {
            UpdateRange::All => "All",
            UpdateRange::MaxDistance(_) => "Distance",
            UpdateRange::Adjacent => "Current",
        }
    }

    fn index(&self) -> usize {
        match self {
            UpdateRange::All => 0,
            UpdateRange::MaxDistance(_) => 1,
            UpdateRange::Adjacent => 2,
        }
    }
}
