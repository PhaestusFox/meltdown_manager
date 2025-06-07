use bevy::{diagnostic::DiagnosticsStore, ecs::entity::EntityIndexSet, prelude::*, scene::ron::de};

use crate::{
    TARGET_TICKTIME,
    diagnostics::ChunkCount,
    utils::{BlockIter, CoreIter, EdgeIter},
    voxels::{
        ChunkId, Neighbours,
        blocks::Blocks,
        cellular_automata::*,
        map::{CHUNK_SIZE, ChunkData},
    },
};

#[derive(States, Clone, Copy, Debug, Hash, Eq, Default)]
pub enum BatchingStep {
    #[default]
    SetupWorld,
    Pause,
    CalculateBatchs,
    Ready,
    Run,
    Done,
}

impl PartialEq for BatchingStep {
    fn eq(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Resource)]
struct BatchingStrategy {
    batch_size: usize,
    groups: Vec<EntityIndexSet>,
    active_groups: usize,
}

#[derive(Resource, Default)]
struct NextBatch(usize);

impl NextBatch {
    fn reset(&mut self) {
        self.0 = 0;
    }

    fn get(&self) -> usize {
        self.0
    }

    fn take(&mut self) -> usize {
        self.0 += 1;
        self.0 - 1
    }
}

impl BatchingStrategy {
    fn max_batches(&self) -> usize {
        self.groups.len()
    }

    fn reserve(&mut self, size: usize) {
        if self.max_batches() >= size {
            return;
        }
        for _ in self.groups.len()..size {
            self.groups
                .push(EntityIndexSet::with_capacity(self.batch_size));
        }
    }

    fn finish(&mut self) {
        self.groups.iter_mut().for_each(|g| g.clear());
        self.active_groups = 0;
    }

    fn get_batch(&self, batch: usize) -> Option<&EntityIndexSet> {
        if self.active_groups <= batch || self.groups.len() <= batch {
            error!(
                "Batching strategy has only {} active groups, but requested batch {}",
                self.active_groups, batch
            );
            return None;
        }
        Some(&self.groups[batch])
    }

    fn batchs(&self) -> impl Iterator<Item = &EntityIndexSet> {
        self.groups.iter().take(self.active_groups)
    }

    fn len(&self) -> usize {
        self.active_groups
    }

    fn is_empty(&self) -> bool {
        self.active_groups == 0
    }

    fn count(&self) -> usize {
        self.groups
            .iter()
            .take(self.active_groups)
            .map(|g| g.len())
            .sum()
    }

    fn clear(&mut self) {
        for group in &mut self.groups {
            group.clear();
        }
    }

    fn find_batch(&self, entity: Entity) -> Option<usize> {
        let mut found = None;
        self.groups.iter().enumerate().for_each(|(i, g)| {
            if g.contains(&entity) {
                found = Some(i);
            }
        });
        found
    }
}

impl FromWorld for BatchingStrategy {
    fn from_world(world: &mut World) -> Self {
        world.init_resource::<NextBatch>();
        world.init_resource::<Step>();
        BatchingStrategy {
            batch_size: 0,
            groups: Vec::new(),
            active_groups: 0,
        }
    }
}

#[derive(Resource, Default)]
pub struct Step(BatchingStep);

impl Step {
    fn set(&mut self, step: BatchingStep) {
        debug_assert_ne!(self.0, step, "Setting Step to the same value: {:?}", step);
        self.0 = step;
    }

    fn get(&self) -> BatchingStep {
        self.0
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<BatchingStrategy>()
        .register_required_components::<Cells, NextStep>() // make sure Cells always has NextStep
        // If we are out of time for a tick, we force finish it in one frame --- makes Step = Done
        .add_systems(FixedFirst, force_finish.run_if(in_step(BatchingStep::Run)))
        // If Step = Run, we run a batch of the simulation --- Sets Step = Done if all batches are finished
        // This must be in Update to maximize performance -- Because it will run with most of the rest of the game
        .add_systems(Update, run_batch.run_if(in_step(BatchingStep::Run)))
        // If Step = Ready, we check if we need to update the batching groups
        // for now we do this every second
        // Update batching groups sets Step = Ready
        // Important: only huristic system should schedule the RecalculateBatchs step
        // ^ Thecnically any system that only runs if Step = Ready can schedule it
        .add_systems(
            PostUpdate,
            (
                batching_huristinc.run_if(in_step(BatchingStep::Ready)),
                update_batching.run_if(in_step(BatchingStep::CalculateBatchs)),
            )
                .chain(),
        )
        // If Step = Ready by time we get to last, set Step = Run --- this is the start of the tick
        // would be better if could be before update to avoid a wasted frame
        // ^ can't think of a way to do this that wouldn't clober Step before other systems get to do there checks
        .add_systems(Last, start_tick.run_if(in_step(BatchingStep::Ready)));

    // wait for world to finish generating chunks
    app.add_systems(
        Update,
        start_ticking.run_if(in_step(BatchingStep::SetupWorld)),
    );
    // If tick it finished, we update the state of the world --- makes Step = Ready
    app.configure_sets(
        Update,
        (ApplyStep::PreApply, ApplyStep::Apply, ApplyStep::PostApply)
            .after(run_batch)
            .chain()
            .run_if(in_step(BatchingStep::Done)),
    );

    app.add_systems(Update, set_prev.in_set(ApplyStep::Apply));

    app.add_systems(
        Update,
        apply_physics
            .in_set(ApplyStep::PostApply)
            .run_if(logic::is_step(logic::StepMode::PHYSICS)),
    );

    app.add_systems(Update, toggle_pause);
    app.add_systems(Update, update_meshs.after(ApplyStep::PostApply));

    // this is what increments the target tick once per 1/10th of a second
    // should run try run before we start a tick
    app.add_systems(
        FixedPostUpdate,
        inc_target.run_if(not(in_step(BatchingStep::Pause))),
    );
}

fn set_prev(
    mut chunks: Query<(Entity, &mut Cells, &mut NextStep)>,
    mut state: ResMut<Step>,
    mut batch: ResMut<NextBatch>,
    strategy: Res<BatchingStrategy>,
) {
    for (entity, mut chunk, mut next) in &mut chunks {
        if !next.has_run {
            error!("NextStep for entity {entity:?} has not run, but we are setting it as previous. This is a bug.\n
            it should have run as part of batch: {:?}\n", strategy.find_batch(entity));
        }
        assert!(next.has_run);
        next.has_run = false;
        if next.chunk.is_solid() != chunk.is_solid() {
            next.chunk.set_not_solid();
        }
        if chunk.is_solid() && chunk.get_block(0, 0, 0).get_block() == Blocks::Air {
            continue;
        }
        std::mem::swap(chunk.bypass_change_detection(), &mut next.chunk);
    }
    batch.reset();
    state.set(BatchingStep::Ready);
}

fn update_batching(
    mut strategy: ResMut<BatchingStrategy>,
    query: Query<Entity, With<Cells>>,
    diagnostics: Res<DiagnosticsStore>,
    mut state: ResMut<Step>,
    chunk_count: Res<crate::diagnostics::ChunkCount>,
) {
    println!("calculating batching groups");

    let frame_time = if let Some(b) =
        diagnostics.get(&bevy::diagnostic::FrameTimeDiagnosticsPlugin::FRAME_TIME)
    {
        b.average().unwrap_or(TARGET_TICKTIME)
    } else {
        TARGET_TICKTIME
    };

    // set min batch size to 10
    let target = (chunk_count.get() / 10).max(5);

    // info!("Frame time: {:.04?}ms", frame_time);
    let batches = ((TARGET_TICKTIME / frame_time) as usize).clamp(4, target) - 1;
    // info!("Targeting {} batches for this tick", batches);
    // info!(
    //     "Targeting {} batches for this tick",
    //     (TARGET_TICKTIME / frame_time)
    // );
    strategy.reserve(batches);
    strategy.active_groups = batches;
    let mut total_entitys = 0;

    strategy.clear();

    for (i, entity) in query.iter().enumerate() {
        total_entitys += 1;
        strategy.groups[i % batches].insert(entity);
    }
    strategy.batch_size = total_entitys / batches;
    // info!(
    //     "set batches to {} with {} entities per batch",
    //     batches, strategy.batch_size
    // );
    debug_assert_eq!(
        chunk_count.get(),
        total_entitys,
        "Chunk count does not number of entities found to tick"
    );
    state.set(BatchingStep::Ready);
}

fn force_finish(
    strategy: Res<BatchingStrategy>,
    start_state: Query<&Cells>,
    mut new_state: Query<(Entity, &ChunkId, &mut NextStep, &Neighbours)>,
    mut next_state: ResMut<Step>,
    mut next_batch: ResMut<NextBatch>,
    tick: Res<VoxelTick>,
) {
    for finish in strategy.batchs().skip(next_batch.get()) {
        next_batch.take();
        new_state.par_iter_many_unique_mut(finish).for_each(
            |(center, id, mut chunk, neighbours)| {
                let Ok(center_pre) = start_state.get(center) else {
                    return;
                };
                let block_o = center_pre.get_block(0, 0, 0);
                if block_o.get_block() == Blocks::Air && center_pre.is_solid() {
                    chunk.has_run = true;
                    return; // skip chunks filled with air
                }
                let mut chunks = [Some(center_pre), None, None, None, None, None, None];
                for (i, n) in neighbours.iter() {
                    if let Ok(neighbour) = start_state.get(n) {
                        chunks[i as usize + 1] = Some(neighbour);
                    }
                }
                #[cfg(debug_assertions)]
                {
                    super::step(
                        ChunkIter::new(&mut chunk.chunk),
                        ChunkGared::new(chunks, *id),
                        tick.get(),
                    );
                }
                #[cfg(not(debug_assertions))]
                {
                    super::step(
                        ChunkIter::new(&mut chunk.chunk),
                        ChunkGared::new(chunks),
                        tick.get(),
                    );
                }

                chunk.has_run = true;
            },
        );
    }

    next_state.set(BatchingStep::Done);
}

fn run_batch(
    strategy: Res<BatchingStrategy>,
    mut state: ResMut<Step>,
    max: NonSend<crate::diagnostics::MaxValue>,
    start_state: Query<&Cells>,
    mut new_state: Query<(Entity, &ChunkId, &mut NextStep, &Neighbours), With<Cells>>,
    mut next_batch: ResMut<NextBatch>,
    tick: Res<VoxelTick>,
) {
    if strategy.is_empty() {
        error!("Batching strategy is empty, but we are in the run step. This is a bug.");
        state.set(BatchingStep::CalculateBatchs);
        return;
    }
    let sender = max.get_sender();
    let current = next_batch.take();
    trace!("Running batch {current} of {}", strategy.len());
    let Some(batch) = strategy.get_batch(current) else {
        state.set(BatchingStep::CalculateBatchs);
        return;
    };
    new_state.par_iter_many_unique_mut(batch).for_each_init(
        || sender.clone(),
        |max, (center, id, mut chunk, neighbours)| {
            let Ok(center_pre) = start_state.get(center) else {
                warn!("Failed to get chunk {id:?} for batching, skipping");
                return;
            };
            let block_o = center_pre.get_block(0, 0, 0);
            if block_o.get_block() == Blocks::Air && center_pre.is_solid() {
                #[cfg(debug_assertions)]
                if id.y != 1 {
                    info!("Skipping chunk {:?} filled with air", id);
                }
                chunk.has_run = true;
                return; // skip chunks filled with air
            }
            let mut chunks = [Some(center_pre), None, None, None, None, None, None];
            for (i, n) in neighbours.iter() {
                if let Ok(neighbour) = start_state.get(n) {
                    chunks[i as usize + 1] = Some(neighbour);
                }
            }
            debug_assert!(!chunk.has_run);
            #[cfg(debug_assertions)]
            {
                let out = super::logic::step_diag(
                    ChunkIter::new(&mut chunk.chunk),
                    ChunkGared::new(chunks, *id),
                    tick.get(),
                );
                let _ = max.send(out);
            }
            #[cfg(not(debug_assertions))]
            super::step(
                ChunkIter::new(&mut chunk.chunk),
                ChunkGared::new(chunks),
                tick.get(),
            );
            chunk.has_run = true;
        },
    );

    if next_batch.get() >= strategy.len() {
        state.set(BatchingStep::Done);
    }
}

fn start_ticking(
    mut state: ResMut<Step>,
    generating_chunks: Res<phoxels::ChunkGenerator<Blocks>>,
    chunk_count: Res<ChunkCount>,
    mut chunk_manager: ResMut<crate::voxels::ChunkManager>,
) {
    if generating_chunks.is_empty() {
        println!(
            "all chunks({}) are generated, starting simulation",
            chunk_count.get()
        );
        state.set(BatchingStep::CalculateBatchs);
        chunk_manager.update_chunk_order(); // run this just before we start to get all chunks in known order
    }
}

fn toggle_pause(
    mut state: ResMut<Step>,
    input: Res<ButtonInput<KeyCode>>,
    mut local: Local<BatchingStep>,
) {
    if input.just_pressed(KeyCode::F10) {
        if state.get() == BatchingStep::Pause {
            info!("Resuming batching");
            state.set(*local);
        } else {
            info!("Pausing batching");
            *local = state.get();
            state.set(BatchingStep::Pause);
        }
    }
}

/// This system is used to determine if we need to recalculate the batching groups.
fn batching_huristinc(
    _strategy: Res<BatchingStrategy>,
    mut state: ResMut<Step>,
    time: Res<Time<Real>>,
    mut last: Local<u32>,
) {
    if *last != time.elapsed_secs() as u32 {
        info!("Triggering batching recalculation");
        *last = time.elapsed_secs() as u32;
        state.set(BatchingStep::CalculateBatchs);
    }
}

fn start_tick(
    mut step: ResMut<Step>,
    time: Res<Time<Real>>,
    mut local: Local<(u8, u32)>,
    mut tick: ResMut<VoxelTick>,
    target: Res<TargetTick>,
) {
    debug_assert!(
        target.get() >= tick.get(),
        "Target tick is in the past: {} < {}",
        target.get(),
        tick.get()
    );
    if tick.get() >= target.get() {
        return;
    }
    if time.delta_secs_f64() > TARGET_TICKTIME {
        warn!(
            "Tick time is too high: {}s, skipping tick",
            time.delta_secs()
        );
        return;
    }
    local.0 += 1;
    if local.1 != time.elapsed_secs() as u32 {
        local.1 = time.elapsed_secs() as u32;
        info!("Ticks this sec: {}", local.0);
        local.0 = 0;
    }
    tick.inc();
    step.set(BatchingStep::Run);
}

fn in_step(step: BatchingStep) -> impl Fn(Res<Step>) -> bool {
    move |s: Res<Step>| s.0 == step
}

pub fn can_modify_next_step(s: Res<Step>) -> bool {
    s.0 == BatchingStep::Done
}

pub fn can_modify_world(s: Res<Step>) -> bool {
    s.0 == BatchingStep::Ready
}

pub fn can_fuck_with_next_step(s: Res<Step>) -> bool {
    s.0 == BatchingStep::Pause
}

// struct NextStepRead<'w, 's> {
//     query: Query<'w, 's, (&'w ChunkId, &'w Cells, &'w NextStep)>,
// }

// fn test_system(query: Query<(&ChunkId, &Cells, &NextStep)>) {
//     for i in query.iter() {}
// }

// unsafe impl<'w, 's> bevy::ecs::system::SystemParam for NextStepRead<'w, 's> {
//     type State = QueryState<(ChunkId, Cells, NextStep)>;

//    // type Item<'world, 'state> = (&'world ChunkId, &'world Cells, &'world NextStep);

//     fn init_state(
//         world: &mut World,
//         system_meta: &mut bevy::ecs::system::SystemMeta,
//     ) -> Self::State {
//         unsafe {
//             system_meta
//                 .component_access_set_mut()
//                 .add_unfiltered_resource_read(world.register_resource::<Step>());
//         }
//         QueryState::new(world)
//     }

//     unsafe fn get_param<'world, 'state>(
//         state: &'state mut Self::State,
//         system_meta: &bevy::ecs::system::SystemMeta,
//         world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
//         change_tick: bevy::ecs::component::Tick,
//     ) -> Self::Item<'world, 'state> {
//         todo!()
//     }

//     unsafe fn validate_param(
//         state: &Self::State,
//         system_meta: &bevy::ecs::system::SystemMeta,
//         world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell,
//     ) -> std::result::Result<(), bevy::ecs::system::SystemParamValidationError> {
//         unsafe {
//             let Some(step) = world.get_resource::<Step>() else {
//                 return Err(bevy::ecs::system::SystemParamValidationError::invalid(
//                     "Failed to get Step resource",
//                 ));
//             };
//             if step.0 != BatchingStep::Done {
//                 return Err(bevy::ecs::system::SystemParamValidationError::new(
//                     true,
//                     "Cant access next if step is not done",
//                     "BatchingStep != Done",
//                 ));
//             }
//         };
//         Ok(())
//     }
// }

#[derive(SystemSet, Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum ApplyStep {
    PreApply,
    Apply,
    PostApply,
}

fn apply_physics(
    mut chunks: Query<Option<&mut Cells>, With<NextStep>>,
    neighbours: Query<(Entity, &ChunkId, &Neighbours)>,
    void_chunks: Res<VoidNeighbours>,
    chunk_manager: Res<crate::voxels::ChunkManager>,
) {
    let mut chunk_sets = Vec::new();
    // let (send, read) = std::sync::mpsc::channel();
    // find all changes that want to be applied
    for entity in chunk_manager.iter() {
        let Ok((_, id, neighbours)) = neighbours.get(entity) else {
            warn!("Failed to get neighbours for chunk {entity:?}, skipping");
            continue;
        };
        let mut c_entitys = [
            entity,
            void_chunks.0[0],
            void_chunks.0[1],
            void_chunks.0[2],
            void_chunks.0[3],
            void_chunks.0[4],
            void_chunks.0[5],
        ];
        for (d, n) in neighbours.iter() {
            c_entitys[d as usize + 1] = n;
        }

        let Ok(chunks) = chunks.get_many(c_entitys) else {
            warn!("Failed to get chunk {entity:?} for physics application, skipping");
            continue;
        };

        #[cfg(debug_assertions)]
        let garde = ChunkGared::new(chunks, *id);
        #[cfg(not(debug_assertions))]
        let garde = ChunkGared::new(chunks);

        let mut to_core = bevy::platform::collections::HashMap::new();
        let mut to_edge = bevy::platform::collections::HashMap::new();
        for cell in CoreIter::new() {
            let Some(block) = garde.get(cell) else {
                warn!("Failed to get block at {cell:?} for chunk {entity:?}, skipping");
                continue;
            };
            let (target, direction) = match block.flags.intersection(CellFlags::MOVE_ALL) {
                // CellFlags::MOVE_LEFT => (cell.left(), CellFlags::MOVE_RIGHT),
                CellFlags::MOVE_RIGHT => (cell.right(), CellFlags::MOVE_LEFT),
                CellFlags::MOVE_FORWARD => (cell.forward(), CellFlags::MOVE_BACK),
                // CellFlags::MOVE_BACK => (cell.backward(), CellFlags::MOVE_FORWARD),
                // CellFlags::MOVE_DOWN => (cell.down(), CellFlags::MOVE_UP),
                CellFlags::MOVE_UP => (cell.up(), CellFlags::MOVE_DOWN),
                _ => continue, // no physics to apply
            };
            let Some(other) = garde.get(target) else {
                warn!(
                    "Failed to get {cell:?} {:?}: {target:?} for chunk {entity:?}, skipping",
                    block.flags
                );
                continue; // target block is out of bounds
            };
            let mut other = other.flags;
            other.remove(CellFlags::IS_GAS | CellFlags::IS_LIQUID);
            if other.bits() != direction.bits() {
                continue; // target block is not trying to swap with this block
            }
            let (a, b) = CellId::order(cell, target);
            to_core.insert(a, b);
        }
        for cell in EdgeIter::new() {
            let Some(block) = garde.get(cell) else {
                warn!("Failed to get block at {cell:?} for chunk {entity:?}, skipping");
                continue;
            };
            let (target, direction) = match block.flags.intersection(CellFlags::MOVE_ALL) {
                // CellFlags::MOVE_LEFT => (cell.left(), CellFlags::MOVE_RIGHT),
                CellFlags::MOVE_RIGHT => (cell.right(), CellFlags::MOVE_LEFT),
                CellFlags::MOVE_FORWARD => (cell.forward(), CellFlags::MOVE_BACK),
                // CellFlags::MOVE_BACK => (cell.backward(), CellFlags::MOVE_FORWARD),
                // CellFlags::MOVE_DOWN => (cell.down(), CellFlags::MOVE_UP),
                CellFlags::MOVE_UP => (cell.up(), CellFlags::MOVE_DOWN),
                _ => continue, // no physics to apply
            };
            let Some(other) = garde.get(target) else {
                warn!(
                    "Failed to get {cell:?} {:?}: {target:?} for chunk {entity:?}, skipping",
                    block.flags
                );
                continue; // target block is out of bounds
            };
            let mut other = other.flags;
            other.remove(CellFlags::IS_GAS | CellFlags::IS_LIQUID);
            if other.bits() != direction.bits() {
                continue; // target block is not trying to swap with this block
            }
            let (a, b) = CellId::order(cell, target);
            to_edge.insert(a, b);
        }
        if to_core.is_empty() && to_edge.is_empty() {
            continue; // no physics to apply
        }
        let to_core = if to_core.is_empty() {
            None
        } else {
            Some(to_core)
        };
        let to_edge = if to_edge.is_empty() {
            None
        } else {
            Some(to_edge)
        };
        chunk_sets.push((entity, (to_core, to_edge)));
    }
    if chunk_sets.is_empty() {
        return;
    }
    // println!("Applying physics to {} chunks", chunk_sets.len());
    let mut applied = bevy::platform::collections::HashSet::new();
    for (entity, (to_core, to_edge)) in chunk_sets {
        applied.clear();
        if let Ok((_, _, neighbours)) = neighbours.get(entity) {
            let mut chunk_entitys = [
                entity,
                void_chunks.0[0],
                void_chunks.0[1],
                void_chunks.0[2],
                void_chunks.0[3],
                void_chunks.0[4],
                void_chunks.0[5],
            ];
            for (n, e) in neighbours.iter() {
                chunk_entitys[n as usize + 1] = e;
            }
            let Ok(chunks) = chunks.get_many_mut(chunk_entitys) else {
                warn!(
                    "Failed to get chunks for entity {entity:?}; Added Void chunks to allow edge chunks to work"
                );
                continue;
            };

            let mut garde = MutChunkGared::new(chunks);
            let iter = match (to_core, to_edge) {
                (None, None) => continue,
                (None, Some(e)) => e,
                (Some(c), None) => c,
                (Some(c), Some(e)) => {
                    for (a, b) in c {
                        if applied.contains(&a) || applied.contains(&b) {
                            continue;
                        }
                        applied.insert(a);
                        applied.insert(b);
                        garde.swap(a, b);
                    }
                    e
                }
            };

            for (a, b) in iter {
                if applied.contains(&a) || applied.contains(&b) {
                    println!("Skipping already applied physics for {a:?} and {b:?}");
                    continue;
                }
                applied.insert(a);
                applied.insert(b);
                garde.swap(a, b);
            }
        }
    }
}

fn update_meshs(
    mut query: Query<(Entity, &Cells, &mut ChunkData), Changed<Cells>>,
    mut mesher: ResMut<phoxels::ChunkMesher>,
) {
    for (entity, cells, mut data) in &mut query {
        if cells.is_solid() && cells.get_block(0, 0, 0).get_block() == Blocks::Air {
            continue; // skip chunks filled with air
        }
        for (i, block) in cells.blocks().enumerate() {
            data.set_block(
                i as u32 % CHUNK_SIZE as u32,
                i as u32 / (CHUNK_SIZE * CHUNK_SIZE) as u32,
                (i as u32 / CHUNK_SIZE as u32) % CHUNK_SIZE as u32,
                block.get_block(),
            );
        }
        mesher.add_to_queue(entity);
    }
}

fn inc_target(mut target: ResMut<TargetTick>) {
    target.inc();
}
