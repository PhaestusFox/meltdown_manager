use std::ops::{Add, Sub};

use bevy::prelude::*;

use crate::voxels::{ChunkManager, cellular_automata::NextStep, map::CHUNK_SIZE};

#[derive(Component, Deref, Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Reflect)]
#[component(immutable, on_insert = ChunkId::on_insert, on_remove = ChunkId::on_remove, on_add = ChunkId::on_add, on_despawn = ChunkId::on_despawn)]
#[require(Transform, Neighbours)]
pub struct ChunkId(IVec3);

impl std::fmt::Display for ChunkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Chunk({},{},{})", self.x, self.y, self.z))
    }
}

impl ChunkId {
    pub const ZERO: ChunkId = ChunkId(IVec3::ZERO);

    pub fn neighbour(&self, direction: NeighbourDirection) -> ChunkId {
        match direction {
            NeighbourDirection::Up => ChunkId(self.0 + IVec3::Y),
            NeighbourDirection::Down => ChunkId(self.0 - IVec3::Y),
            NeighbourDirection::Left => ChunkId(self.0 - IVec3::X),
            NeighbourDirection::Right => ChunkId(self.0 + IVec3::X),
            NeighbourDirection::Front => ChunkId(self.0 + IVec3::Z),
            NeighbourDirection::Back => ChunkId(self.0 - IVec3::Z),
        }
    }

    pub fn new(x: i32, y: i32, z: i32) -> ChunkId {
        Self(IVec3::new(x, y, z))
    }

    pub fn manhattan_distance(self, other: &ChunkId) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs() + (self.z - other.z).abs()) as u32
    }

    fn on_add(mut world: bevy::ecs::world::DeferredWorld, _ctx: bevy::ecs::component::HookContext) {
        world.resource_mut::<crate::diagnostics::ChunkCount>().inc();
    }

    fn on_despawn(
        mut world: bevy::ecs::world::DeferredWorld,
        _ctx: bevy::ecs::component::HookContext,
    ) {
        world.resource_mut::<crate::diagnostics::ChunkCount>().dec();
    }

    fn on_insert(
        mut world: bevy::ecs::world::DeferredWorld,
        ctx: bevy::ecs::component::HookContext,
    ) {
        let id = *world
            .get::<ChunkId>(ctx.entity)
            .expect("This Just got inserted");
        world
            .get_mut::<Transform>(ctx.entity)
            .expect("Required Componet")
            .translation = id.to_translation();

        if world.get::<Name>(ctx.entity).is_none() {
            world
                .commands()
                .entity(ctx.entity)
                .insert(Name::new(format!("{}", id)));
        }

        let neighbours = world
            .get::<Neighbours>(ctx.entity)
            .expect("Required Componet");
        let too_apply = EmptyNeighboursIter::new(neighbours).collect::<Vec<_>>();

        let manager = world.resource::<ChunkManager>();
        let mut can_apply = Vec::with_capacity(too_apply.len());
        let mut recip = Vec::with_capacity(too_apply.len());
        for (apply, direction) in too_apply {
            if let Some(other) = manager.get_chunk(&id.neighbour(direction)) {
                can_apply.push((apply, other));
                recip.push((other, direction.rev()));
            }
        }

        let mut neighbours = world
            .get_mut::<Neighbours>(ctx.entity)
            .expect("Required Componet");
        for (apply, other) in can_apply {
            apply(&mut neighbours, other);
        }

        for (other, direction) in recip {
            if let Some(mut neighbours) = world.get_mut::<Neighbours>(other) {
                match direction {
                    NeighbourDirection::Up => neighbours.up = Some(ctx.entity),
                    NeighbourDirection::Down => neighbours.down = Some(ctx.entity),
                    NeighbourDirection::Left => neighbours.left = Some(ctx.entity),
                    NeighbourDirection::Right => neighbours.right = Some(ctx.entity),
                    NeighbourDirection::Front => neighbours.front = Some(ctx.entity),
                    NeighbourDirection::Back => neighbours.back = Some(ctx.entity),
                }
            } else {
                warn!("Failed to get Neighbours for {other:?} this is probably a bug");
            }
        }

        let mut manager = world.resource_mut::<ChunkManager>();

        if let Some(old) = manager.insert_chunk(id, ctx.entity) {
            if old != ctx.entity {
                warn!(
                    "already used ChunkId({}) on {}: this is probably unitentonal despawing old entity",
                    id.0, ctx.entity
                );
                world.commands().entity(old).despawn();
            } else {
                warn!(
                    "inseted ChunkId({}) onto the same entity: this should not be done",
                    id.0
                )
            }
        }
    }

    fn on_remove(
        mut world: bevy::ecs::world::DeferredWorld,
        ctx: bevy::ecs::component::HookContext,
    ) {
        let id = *world
            .get::<ChunkId>(ctx.entity)
            .expect("This Just about to be removed");
        let mut map = world.resource_mut::<ChunkManager>();
        if let Some(old) = map.remove_chunk(&id) {
            if old != ctx.entity {
                error!(
                    "removed ChunkId from {} but {} has the same id\n*This is a Bug*\n
                Adding {} back to Manager",
                    ctx.entity, old, old
                );
                map.insert_chunk(id, old);
            }
        }
    }

    pub fn from_translation(mut translation: Vec3) -> Self {
        translation /= CHUNK_SIZE as f32;
        ChunkId(translation.floor().as_ivec3())
    }

    pub fn to_translation(self) -> Vec3 {
        (self.0 * CHUNK_SIZE).as_vec3()
    }

    pub fn from_str(str: &str) -> Result<Self, &'static str> {
        let mut s = str.trim();
        if s.is_empty() {
            return Err("ChunkId cannot be empty");
        }
        if s.contains('(') && s.contains(')') {
            let mut iter = s.split('(');
            let _ = iter
                .next()
                .ok_or("Failed to get Preceding ( with trailing )")?;
            s = iter
                .next()
                .ok_or("Failed to find value after (")?
                .split(')')
                .next()
                .ok_or("Failed to find trailing )")?;
        }
        #[allow(unused_assignments)]
        let mut x = None;
        #[allow(unused_assignments)]
        let mut y = None;
        #[allow(unused_assignments)]
        let mut z = None;
        if s.contains(',') {
            let mut split = s.split(',');
            x = split.next();
            y = split.next();
            z = split.next();
        } else if s.contains(|c: char| c.is_whitespace()) {
            let mut split = s.split_whitespace();
            x = split.next();
            y = split.next();
            z = split.next();
        } else {
            let mut change = false;
            let mut split = s.split(move |c: char| {
                if c.is_alphabetic() && !change {
                    false
                } else if (c.is_numeric() || c == '-' || c == ':') && !change {
                    change = true;
                    false
                } else if c.is_alphabetic() && change {
                    change = false;
                    true
                } else {
                    false
                }
            });
            x = split.next();
            y = split.next();
            z = split.next();
        };

        let x = x.ok_or("Failed to find x value")?.trim();
        let y = y.ok_or("Failed to find y value")?.trim();
        let z = z.ok_or("Failed to find z value")?.trim();

        let x = x
            .trim_start_matches(|c: char| !c.is_numeric() && c != '-')
            .trim()
            .parse::<i32>()
            .map_err(|_| "Failed to parse x")?;
        let y = y
            .trim_start_matches(|c: char| !c.is_numeric() && c != '-')
            .trim()
            .parse::<i32>()
            .map_err(|_| "Failed to parse y")?;
        let z = z
            .trim_start_matches(|c: char| !c.is_numeric() && c != '-')
            .trim()
            .parse::<i32>()
            .map_err(|_| "Failed to parse z")?;
        Ok(ChunkId(IVec3::new(x, y, z)))
    }
}

#[derive(Component, Debug, Default)]
pub struct Neighbours {
    up: Option<Entity>,
    down: Option<Entity>,
    left: Option<Entity>,
    right: Option<Entity>,
    front: Option<Entity>,
    back: Option<Entity>,
}

#[derive(Resource)]
pub struct VoidNeighbours(pub [Entity; 6]);

impl FromWorld for VoidNeighbours {
    fn from_world(world: &mut World) -> Self {
        VoidNeighbours([
            world.spawn(NextStep::default()).id(),
            world.spawn(NextStep::default()).id(),
            world.spawn(NextStep::default()).id(),
            world.spawn(NextStep::default()).id(),
            world.spawn(NextStep::default()).id(),
            world.spawn(NextStep::default()).id(),
        ])
    }
}

struct EmptyNeighboursIter<'a> {
    neighbours: &'a Neighbours,
    index: usize,
}

pub struct NeighboursIter<'a> {
    neighbours: &'a Neighbours,
    index: usize,
}

impl Iterator for NeighboursIter<'_> {
    type Item = (NeighbourDirection, Entity);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < 6 {
            let idx = self.index;
            self.index += 1;
            let out = match idx {
                0 => self.neighbours.up(),
                1 => self.neighbours.down(),
                2 => self.neighbours.left(),
                3 => self.neighbours.right(),
                4 => self.neighbours.front(),
                5 => self.neighbours.back(),
                _ => None,
            };
            if let Some(out) = out {
                return Some((NeighbourDirection::from_index(idx), out));
            }
        }
        None
    }
}

impl<'a> EmptyNeighboursIter<'a> {
    fn new(neighbours: &'a Neighbours) -> Self {
        Self {
            neighbours,
            index: 0,
        }
    }
}

impl<'a> Iterator for EmptyNeighboursIter<'a> {
    type Item = (fn(&mut Neighbours, Entity), NeighbourDirection);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= 6 {
            return None;
        }
        let idx = self.index;
        let entry = match idx {
            0 => self.neighbours.up.is_none(),
            1 => self.neighbours.down.is_none(),
            2 => self.neighbours.left.is_none(),
            3 => self.neighbours.right.is_none(),
            4 => self.neighbours.front.is_none(),
            5 => self.neighbours.back.is_none(),
            _ => unreachable!(),
        };
        self.index += 1;
        if entry {
            let f: fn(&mut Neighbours, Entity) = match idx {
                0 => |n, e| n.up = Some(e),
                1 => |n, e| n.down = Some(e),
                2 => |n, e| n.left = Some(e),
                3 => |n, e| n.right = Some(e),
                4 => |n, e| n.front = Some(e),
                5 => |n, e| n.back = Some(e),
                _ => unreachable!(),
            };
            let id = NeighbourDirection::from_index(idx);
            Some((f, id))
        } else {
            self.next()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NeighbourDirection {
    Up,
    Down,
    Left,
    Right,
    Front,
    Back,
}

impl NeighbourDirection {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Up,
            1 => Self::Down,
            2 => Self::Left,
            3 => Self::Right,
            4 => Self::Front,
            5 => Self::Back,
            _ => {
                #[cfg(debug_assertions)]
                unreachable!(); // this should never happen, but if it does, panic in debug mode
                #[allow(unreachable_code)]
                Self::Up // default to up if in release mode
            }
        }
    }

    pub fn rev(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Front => Self::Back,
            Self::Back => Self::Front,
        }
    }
}

impl Neighbours {
    pub fn up(&self) -> Option<Entity> {
        self.up
    }
    pub fn down(&self) -> Option<Entity> {
        self.down
    }
    pub fn left(&self) -> Option<Entity> {
        self.left
    }
    pub fn right(&self) -> Option<Entity> {
        self.right
    }
    pub fn front(&self) -> Option<Entity> {
        self.front
    }
    pub fn back(&self) -> Option<Entity> {
        self.back
    }

    pub fn iter(&self) -> NeighboursIter {
        NeighboursIter {
            neighbours: self,
            index: 0,
        }
    }
}

impl Add for ChunkId {
    type Output = ChunkId;

    fn add(self, other: ChunkId) -> Self::Output {
        ChunkId(self.0 + other.0)
    }
}

impl Sub for ChunkId {
    type Output = ChunkId;

    fn sub(self, other: ChunkId) -> Self::Output {
        ChunkId(self.0 - other.0)
    }
}

impl ChunkId {
    pub fn min(self, other: ChunkId) -> ChunkId {
        ChunkId(IVec3::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        ))
    }
    pub fn max(self, other: ChunkId) -> ChunkId {
        ChunkId(IVec3::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        ))
    }
}
