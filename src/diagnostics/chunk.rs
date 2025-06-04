use bevy::{
    diagnostic::{
        Diagnostic, DiagnosticMeasurement, DiagnosticPath, Diagnostics, DiagnosticsStore,
        RegisterDiagnostic,
    },
    ecs::system::SystemId,
    prelude::*,
};

use crate::{
    diagnostics::{DiagnosticSettings, TabButton},
    voxels::{ChunkId, NeighbourDirection},
};

const CHUNK_COUNT: DiagnosticPath = DiagnosticPath::const_new("Chunk count");

#[derive(Resource, Default)]
pub struct ChunkCount(usize);

impl ChunkCount {
    pub fn inc(&mut self) {
        self.0 += 1;
    }

    pub fn dec(&mut self) {
        if self.0 > 0 {
            self.0 -= 1;
        } else {
            error!("Attempting to decrement chunk count below zero");
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, reg_tab)
        .register_diagnostic(
            Diagnostic::new(CHUNK_COUNT)
                .with_max_history_length(0)
                .with_smoothing_factor(0.),
        )
        .add_systems(Update, (mark_unknown, update_count, update_text))
        .init_resource::<TabState>()
        .init_resource::<ChunkCount>();
}

fn reg_tab(mut settings: ResMut<DiagnosticSettings>, mut commands: Commands) {
    let on_open = commands.register_system(on_open);
    let on_close = commands.register_system(on_close);

    settings.register_tab("Chunk", on_open, on_close);
}

#[derive(Resource)]
struct TabState {
    valadate_neighbour_id: SystemId,
    valadation_state: ValidationState,
}

impl FromWorld for TabState {
    fn from_world(world: &mut World) -> Self {
        let id = world.register_system(validate_neighbours);
        TabState {
            valadate_neighbour_id: id,
            valadation_state: ValidationState::Unknown,
        }
    }
}

#[derive(Debug)]
enum ValidationState {
    Valid,
    Unknown,
    Invalid,
}

fn on_open(In(content): In<Entity>, mut commands: Commands, state: Res<TabState>) {
    commands.entity(content).with_children(|p| {
        p.spawn((Text::new("Count"), CountText));
        p.spawn((
            Text::new("Neighbours:"),
            Node {
                width: Val::Percent(100.),
                height: Val::Px(50.),
                ..Default::default()
            },
            children!(
                (
                    Node {
                        width: Val::Percent(90.),
                        ..Default::default()
                    },
                    BackgroundColor(Color::linear_rgb(0.8, 0.8, 0.8)),
                    Text::new("Valadatre"),
                    TabButton(state.valadate_neighbour_id),
                ),
                (Text::new("Unknown"), ValadatreText)
            ),
        ));
    });
}

fn on_close(content: In<Entity>, mut commands: Commands) {
    commands.entity(*content).despawn_related::<Children>();
}

fn update_count(
    mut text: Single<&mut Text, With<CountText>>,
    mut diagnostics: Diagnostics,
    count: Res<ChunkCount>,
) {
    if !count.is_changed() {
        return;
    }
    diagnostics.add_measurement(&CHUNK_COUNT, || count.0 as f64);
    text.0 = format!("Count: {}", count.0);
}
fn update_text(mut text: Query<&mut Text, With<ValadatreText>>, state: Res<TabState>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    match state.valadation_state {
        ValidationState::Valid => text.0 = String::from("Valid"),
        ValidationState::Unknown => text.0 = String::from("Unknown"),
        ValidationState::Invalid => text.0 = String::from("Invalid"),
    }
}

#[derive(Component)]
struct CountText;

#[derive(Component)]
struct ValadatreText;

fn validate_neighbours(
    mut state: ResMut<TabState>,
    neighbors: Query<(Entity, &crate::voxels::Neighbours)>,
) {
    let mut is_valid = true;
    for (entity, neighbour) in neighbors.iter() {
        for (n, e) in neighbour.iter() {
            if e == entity {
                continue; // Skip self-references
            }
            if let Ok((_, o)) = neighbors.get(e) {
                if match n.rev() {
                    NeighbourDirection::Up => o.up(),
                    NeighbourDirection::Down => o.down(),
                    NeighbourDirection::Left => o.left(),
                    NeighbourDirection::Right => o.right(),
                    NeighbourDirection::Front => o.front(),
                    NeighbourDirection::Back => o.back(),
                } != Some(entity)
                {
                    is_valid = false;
                    error!(
                        "Invalid neighbour for entity {:?}: {:?} -> {:?}",
                        entity, n, e
                    );
                    break;
                }
            } else {
                error!("Entity {:?} has invalid neighbour: {:?}", entity, e);
                is_valid = false;
                break;
            }
        }
    }
    if is_valid {
        state.valadation_state = ValidationState::Valid;
    } else {
        state.valadation_state = ValidationState::Invalid;
    };
}

fn mark_unknown(mut state: ResMut<TabState>, change: Query<(), Changed<ChunkId>>) {
    if change.iter().next().is_some() {
        state.valadation_state = ValidationState::Unknown;
    }
}
