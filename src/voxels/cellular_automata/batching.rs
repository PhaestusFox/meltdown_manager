use std::ops::{Deref, DerefMut};

use bevy::{log::tracing_subscriber::filter::combinator::Not, prelude::*};

use crate::{
    utils::BlockIter,
    voxels::{
        Chunk, Neighbours,
        cellular_automata::*,
        map::{CHUNK_VOL, ChunkData},
    },
};

#[derive(States, Clone, Copy, Debug, Hash, Eq, Default)]
enum BatchingStep {
    #[default]
    SetupStep,
    CalculatingGroups,
    RunningGroup(usize),
    Done,
}

impl PartialEq for BatchingStep {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Resource)]
struct BatchingStrategy {
    current_step_start: Option<std::time::Instant>,
    last_step_time: f32,
    groups: Vec<Vec<Entity>>,
    active_groups: usize,
}

impl FromWorld for BatchingStrategy {
    fn from_world(_: &mut World) -> Self {
        BatchingStrategy {
            current_step_start: None,
            last_step_time: 0.0,
            groups: Vec::new(),
            active_groups: 0,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<BatchingStrategy>()
        .init_state::<BatchingStep>()
        .register_required_components::<Cells, NextStep>()
        .add_systems(FixedPostUpdate, force_finish)
        .add_systems(FixedPreUpdate, set_prev);
}

pub fn set_prev(
    mut chunks: Query<(&mut Cells, &mut NextStep, Ref<ChunkData>)>,
    mut to_init: Query<(Entity, &ChunkData), Without<NextStep>>,
    mut commands: Commands,
) {
    for (mut chunk, mut next, data) in &mut chunks {
        for (x, y, z) in BlockIter::<30, 30, 30>::new() {
            let block = data
                .get_block(x as u32, y as u32, z as u32)
                .unwrap_or(crate::voxels::blocks::Blocks::Void);
            next.get_by_index_mut(Cells::index(x, y, z)).block = block;
            chunk.get_by_index_mut(Cells::index(x, y, z)).block = block;
        }
        std::mem::swap(chunk.as_mut(), next.deref_mut());
    }
    for (entity, chunk) in &mut to_init {
        // let prev = NextStep(std::mem::replace(chunk.as_mut(), Chunk::empty()));
        let mut next: Chunk<CellData> = Chunk::empty();
        for (x, y, z) in BlockIter::<30, 30, 30>::new() {
            let block = chunk
                .get_block(x as u32, y as u32, z as u32)
                .unwrap_or(crate::voxels::blocks::Blocks::Void);
            next.get_by_index_mut(Cells::index(x, y, z)).block = block;
        }
        commands.entity(entity).insert(NextStep(next));
    }
}

pub fn force_finish(state: Res<BatchingStrategy>, chunks: Query<&mut NextStep>) {}

pub fn step_system(
    max: NonSend<crate::diagnostics::MaxValue>,
    start_state: Query<&Cells>,
    mut new_state: Query<(Entity, &mut NextStep, &Neighbours)>,
) {
    let sender = max.get_sender();
    new_state.par_iter_mut().for_each_init(
        || sender.clone(),
        |max, (center, mut chunk, neighbours)| {
            let Ok(center_pre) = start_state.get(center) else {
                return;
            };
            let mut chunks = [Some(center_pre), None, None, None, None, None, None];
            for (i, n) in neighbours.iter() {
                if let Ok(neighbour) = start_state.get(n) {
                    chunks[i as usize + 1] = Some(neighbour);
                }
            }

            let out =
                super::logic::step(ChunkIter::new(chunk.deref_mut()), ChunkGared::new(chunks));
            let _ = max.send(out);
        },
    );
}
