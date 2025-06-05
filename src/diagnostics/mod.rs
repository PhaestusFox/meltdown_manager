use std::borrow::Cow;

use bevy::{
    ecs::{
        spawn::SpawnIter,
        system::{RunSystemOnce, SystemId},
    },
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
};
#[cfg(not(target_arch = "wasm32"))]
mod automata;
mod chunk;
mod entity;
mod fps;
#[cfg(not(target_arch = "wasm32"))]
pub mod shader;

mod reporting;

pub use chunk::ChunkCount;
pub use reporting::MaxValue;
pub struct MeltdownDiagnosticsPlugin;

impl Plugin for MeltdownDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        // init our settings
        app.init_resource::<DiagnosticSettings>();
        app.add_plugins((fps::plugin, entity::plugin, chunk::plugin))
            .add_systems(Update, (toggle_window, on_click_tap))
            .add_systems(PostStartup, on_init)
            .add_systems(Update, tab_button_system)
            // .add_observer(slider_observer)
            // .add_observer(slider_drop)
            // .add_observer(slider_start)
            // .add_observer(slider_hover)
            // .add_observer(slider_hover_refined);
            .add_systems(Update, slider_not_observer);
        app.init_non_send_resource::<reporting::MaxValue>();

        // web doesnt like my shaders
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(MaterialPlugin::<shader::DebugMaterial>::default())
            .add_plugins(automata::plugin);
    }
}

#[derive(Resource)]
pub struct DiagnosticSettings {
    pub enabled: bool,
    pub pannal_color: Color,
    pub registured_tabs: Vec<DiagnosticTab>,
    pub tab_style: Node,
    pub tab_color: (Color, Color),
}

impl DiagnosticSettings {
    fn register_tab(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        on_open: SystemId<In<Entity>>,
        on_close: SystemId<In<Entity>>,
    ) {
        let name = name.into();
        info!("Registuring Diagnostic Tab {}", name);
        self.registured_tabs.push(DiagnosticTab {
            name,
            on_open,
            on_close,
        });
    }
}

impl Default for DiagnosticSettings {
    fn default() -> Self {
        DiagnosticSettings {
            enabled: false,
            pannal_color: Color::linear_rgb(0.2, 0.2, 0.2),
            registured_tabs: Vec::new(),
            tab_style: Node {
                height: Val::Percent(95.),
                ..Default::default()
            },
            tab_color: (
                Color::linear_rgb(0.4, 0.4, 0.4),
                Color::linear_rgb(0.3, 0.3, 0.3),
            ),
        }
    }
}

#[derive(Component)]
struct DiagnosticsWindow {
    on_enable: SystemId,
    on_disable: SystemId,
    cuttent_tab: Option<usize>,
}

#[derive(Clone)]
pub struct DiagnosticTab {
    name: Cow<'static, str>,
    on_open: SystemId<In<Entity>, ()>,
    on_close: SystemId<In<Entity>, ()>,
}

fn on_init(mut commands: Commands, settings: Res<DiagnosticSettings>) {
    let on_enable = commands.register_system(on_enabled);
    let on_disable = commands.register_system(on_disable);
    let mut tab_style = settings.tab_style.clone();
    tab_style.flex_grow = 0.95 / settings.registured_tabs.len() as f32;
    let tab_color = settings.tab_color.0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                top: Val::Px(20.),
                left: Val::Px(0.),
                width: Val::Percent(25.),
                height: Val::Percent(55.),
                ..Default::default()
            },
            Visibility::Hidden,
            BackgroundColor(settings.pannal_color),
            DiagnosticsWindow {
                on_disable,
                on_enable,
                cuttent_tab: None,
            },
            Name::new("Diagnostics Window"),
        ))
        .with_child((
            Node {
                width: Val::Percent(95.),
                height: Val::Percent(10.),
                margin: UiRect::all(Val::Auto),
                justify_content: JustifyContent::SpaceEvenly,
                ..Default::default()
            },
            Name::new("Tabs"),
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            Children::spawn(SpawnIter(
                settings
                    .registured_tabs
                    .clone()
                    .into_iter()
                    .enumerate()
                    .map(move |(i, tab)| {
                        (
                            Text::new(tab.name.clone()),
                            Name::new(tab.name),
                            Button,
                            TextLayout {
                                justify: JustifyText::Center,
                                ..Default::default()
                            },
                            BackgroundColor(tab_color),
                            tab_style.clone(),
                            DiagnosticTabId(i),
                        )
                    }),
            )),
        ))
        .with_child((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(80.),
                margin: UiRect::all(Val::Auto),
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::Wrap,
                padding: UiRect::all(Val::Percent(2.5)),
                ..Default::default()
            },
            Name::new("Content"),
        ));
}

#[derive(Component)]
struct DiagnosticTabId(usize);

fn on_click_tap(
    settings: Res<DiagnosticSettings>,
    mut window: Query<(&mut DiagnosticsWindow, &Children)>,
    tabs: Query<(&Interaction, &DiagnosticTabId), Changed<Interaction>>,
    mut bg: Query<(&DiagnosticTabId, &mut BackgroundColor)>,
    mut commands: Commands,
) {
    let Ok((mut window, children)) = window.single_mut() else {
        error!("Diagnositc Window Not Init");
        return;
    };
    let Some(content) = children.last().cloned() else {
        error!("Diagnostic Window Has not content Aria");
        return;
    };
    for (interaction, DiagnosticTabId(id)) in &tabs {
        if let Interaction::Pressed = *interaction {
            if let Some(old) = window.cuttent_tab {
                commands.run_system_with(settings.registured_tabs[old].on_close, content);
            }
            commands.run_system_with(settings.registured_tabs[*id].on_open, content);
            window.cuttent_tab = Some(*id);
            for (DiagnosticTabId(old), mut bg) in &mut bg {
                if old == id {
                    bg.0 = settings.tab_color.1;
                } else {
                    bg.0 = settings.tab_color.0;
                }
            }
        }
    }
}

fn on_enabled(mut window: Query<&mut Visibility, With<DiagnosticsWindow>>) {
    let Ok(mut visibility) = window.single_mut() else {
        error!("Diagnosics Window Invalid: No Visibility");
        return;
    };
    *visibility = Visibility::Visible;
}

fn on_disable(mut window: Query<&mut Visibility, With<DiagnosticsWindow>>) {
    let Ok(mut visibility) = window.single_mut() else {
        error!("Diagnosics Window Invalid: No Visibility");
        return;
    };
    *visibility = Visibility::Hidden;
}

fn toggle_window(
    inputs: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<DiagnosticSettings>,
    mut commands: Commands,
    window: Query<&DiagnosticsWindow>,
) {
    if !inputs.just_pressed(KeyCode::F3) {
        return;
    }
    settings.enabled = !settings.enabled;
    let Ok(window) = window.single() else {
        commands.queue(|world: &mut World| world.run_system_once(on_init));
        return;
    };
    if settings.enabled {
        commands.run_system(window.on_enable);
    } else {
        commands.run_system(window.on_disable);
    }
}

#[derive(Component)]
#[require(Button)]
struct TabButton(SystemId);

impl TabButton {
    pub fn new(system_id: SystemId) -> Self {
        TabButton(system_id)
    }
}

fn tab_button_system(
    buttons: Query<(&Interaction, &TabButton), Changed<Interaction>>,
    mut commands: Commands,
) {
    for (interaction, TabButton(system_id)) in &buttons {
        if let Interaction::Pressed = *interaction {
            info!("Running System: {:?}", system_id);
            commands.run_system(*system_id);
        }
    }
}

#[derive(Component)]
#[require(ComputedNode)]
#[component(on_add = Slider::on_add)]
struct Slider {
    pub on_change: SystemId<In<f32>>,
    pub min: f32,
    pub max: f32,
}

impl Slider {
    fn on_add(mut world: bevy::ecs::world::DeferredWorld, ctx: bevy::ecs::component::HookContext) {
        world.commands().entity(ctx.entity).with_child((
            Node {
                width: Val::Percent(5.),
                height: Val::Percent(90.),
                left: Val::Px(0.),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgb(0.2, 0.2, 0.2)),
            SliderHandle {
                target: ctx.entity,
                delta: 0.,
                start: Val::Auto,
            },
            ZIndex(1),
        ));
    }

    fn vlaue(&self, precent: f32) -> f32 {
        let range = self.max - self.min;
        self.min + (precent * range)
    }
}

#[derive(Component)]
#[require(Button)]
struct SliderHandle {
    target: Entity,
    start: Val,
    delta: f32,
}

fn slider_start(
    trigger: Trigger<Pointer<DragStart>>,
    mut handle: Query<(&mut SliderHandle, &Node)>,
) {
    let Ok((mut handle, node)) = handle.get_mut(trigger.target) else {
        return;
    };
    // Store the start position of the handle
    handle.start = node.left;
    println!("Slider Start: {:?}", trigger);
}

fn slider_observer(
    tringger: Trigger<Pointer<Drag>>,
    mut handle: Query<(&SliderHandle, &mut Node)>,
    data: Query<&ComputedNode>,
) {
    let Ok((handle, mut node)) = handle.get_mut(tringger.target) else {
        return;
    };
    let Ok(slider) = data.get(handle.target) else {
        error!("Slider: No Slider Found for Handle {:?}", handle.target);
        return;
    };
    match handle.start {
        Val::Px(start) => {
            let Ok(data) = data.get(handle.target) else {
                error!(
                    "Slider: No ComputedNode Found for Handle {:?}",
                    handle.target
                );
                return;
            };
            node.left =
                Val::Px((start + tringger.distance.x).clamp(0., slider.size.x - data.size.x));
        }
        Val::Percent(start) => {
            let moved = tringger.distance.x / slider.size.x;
            println!(
                "{} / {} = Moved: {:?}",
                tringger.distance.x, slider.size.x, moved
            );
            node.left = Val::Percent((start + moved).clamp(0., 95.));
        }
        i => {
            warn!(
                "Slider Handle Start Position is {:?}, this is not supported",
                i
            );
        }
    }
}

fn slider_drop(
    trigger: Trigger<Pointer<DragEnd>>,
    handel: Query<(&Node, &SliderHandle)>,
    slider: Query<(&Slider, &ComputedNode)>,
    mut commands: Commands,
) {
    let Ok((node, handle)) = handel.get(trigger.target) else {
        return;
    };
    let Ok((slider, data)) = slider.get(handle.target) else {
        error!("Slider: No Slider Found for Handle {:?}", handle.target);
        return;
    };
    let p = match node.left {
        Val::Percent(p) => {
            p * 1.05 // mult by 1.05 to account for the width of the handle
        }
        Val::Px(w) => {
            let p = w / data.size.x;
            p * 1.05 // mult by 1.05 to account for the width of the handle
        }
        i => {
            warn!(
                "Slider Handle Left Position is {:?}, this is not supported",
                i
            );
            return;
        }
    };
    commands.run_system_with(slider.on_change, slider.vlaue(p));
}

fn slider_not_observer(
    mouse_movement: Res<AccumulatedMouseMotion>,
    mut handle: Query<(&mut Node, &mut SliderHandle, &Interaction)>,
    slider: Query<(&Slider, &ComputedNode)>,
    input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    for (mut node, mut handle, interaction) in &mut handle {
        let Ok((slider, space)) = slider.get(handle.target) else {
            error!("Slider: No Slider Found for Handle {:?}", handle.target);
            continue;
        };
        if input.just_pressed(MouseButton::Left) && *interaction == Interaction::Pressed {
            handle.start = node.left;
            handle.delta = 0.;
            println!("clicked");
        }
        if input.pressed(MouseButton::Left) {
            match handle.start {
                Val::Percent(p) => {
                    handle.delta += mouse_movement.delta.x;
                    node.left =
                        Val::Percent((p + (handle.delta / space.size.x) * 50.).clamp(0., 95.));
                }
                Val::Px(start) => {
                    handle.delta += mouse_movement.delta.x;
                    node.left = Val::Px((start + handle.delta).clamp(0., space.size.x * 95.));
                }
                _ => {
                    continue; // means not targeted
                }
            }
        }
        if input.just_released(MouseButton::Left) && handle.start != Val::Auto {
            let p = match node.left {
                Val::Percent(p) => p * 105., // mult by 1.05 to account for the width of the handle
                Val::Px(w) => {
                    let p = w / space.size.x;
                    p * 105. // mult by 1.05 to account for the width of the handle
                }
                i => {
                    warn!(
                        "Slider Handle Left Position is {:?}, this is not supported",
                        i
                    );
                    continue; // means not targeted
                }
            };
            handle.start = Val::Auto;
            commands.run_system_with(slider.on_change, slider.vlaue(p));
        }
    }
}

fn slider_hover(
    trigger: Trigger<Pointer<Over>>,
    handle: Query<&BackgroundColor, With<SliderHandle>>,
) {
    let Ok(node) = handle.get(trigger.target) else {
        return;
    };
    println!("Slider Hover: {:?}", node);
}

fn slider_hover_refined(
    trigger: Trigger<Pointer<Over>, SliderHandle>,
    handle: Query<&BackgroundColor, With<SliderHandle>>,
) {
    let Ok(node) = handle.get(trigger.target) else {
        return;
    };
    println!("Refined Slider Hover: {:?}", node);
}
