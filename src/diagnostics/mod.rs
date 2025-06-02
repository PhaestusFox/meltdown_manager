use std::borrow::Cow;

use bevy::{
    ecs::{
        spawn::SpawnIter,
        system::{RunSystemOnce, SystemId},
    },
    prelude::*,
};
mod entity;
mod fps;
pub mod shader;

pub struct MeltdownDiagnosticsPlugin;

impl Plugin for MeltdownDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        // init our settings
        app.init_resource::<DiagnosticSettings>();
        app.add_plugins((fps::plugin, entity::plugin))
            .add_plugins(MaterialPlugin::<shader::DebugMaterial>::default())
            .add_systems(Update, (toggle_window, on_click_tap))
            .add_systems(PostStartup, on_init);
    }
}

#[derive(Resource)]
pub struct DiagnosticSettings {
    pub enabled: bool,
    pub pannal_color: Color,
    pub registured_tabs: Vec<DiagnosticTab>,
    pub tab_style: Node,
    pub tab_color: (Color, Color),
    pub cell_mode: crate::voxels::cellular_automata::CellMode,
}

impl DiagnosticSettings {
    fn register_tab(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        on_open: SystemId<In<Entity>>,
        on_close: SystemId<In<Entity>>,
    ) {
        let name = name.into();
        info!("Registuring Diagnostic Tap {}", name);
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
            cell_mode: crate::voxels::cellular_automata::CellMode::All,
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
                height: Val::Percent(50.),
                ..Default::default()
            },
            Visibility::Hidden,
            BackgroundColor(settings.pannal_color),
            DiagnosticsWindow {
                on_disable,
                on_enable,
                cuttent_tab: None,
            },
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
                width: Val::Percent(95.),
                height: Val::Percent(80.),
                margin: UiRect::all(Val::Auto),
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
