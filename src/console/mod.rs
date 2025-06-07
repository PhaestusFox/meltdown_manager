use bevy::prelude::*;
use bevy_console::*;

pub(crate) mod commands;

pub fn plugin(app: &mut App) {
    app.add_plugins(bevy_console::ConsolePlugin);
    app.add_console_command::<commands::ChunkHighlightCommand, _>(
        commands::chunk_highlight_command,
    )
    .add_console_command::<commands::NeighborsCommand, _>(commands::chunk_neighbore_command)
    .add_console_command::<commands::RedrawCommand, _>(commands::chunk_redraw_command);

    commands::init(app);
}

#[derive(Component)]
#[component(on_add = AxisPointer::on_add, on_remove = AxisPointer::on_remove)]
/// Pointer component that holds the entities for the x, y, and z axes.
struct AxisPointer {
    x: Entity,
    y: Entity,
    z: Entity,
}

impl AxisPointer {
    pub fn new() -> Self {
        Self {
            x: Entity::PLACEHOLDER,
            y: Entity::PLACEHOLDER,
            z: Entity::PLACEHOLDER,
        }
    }
    fn on_add(mut world: bevy::ecs::world::DeferredWorld, ctx: bevy::ecs::component::HookContext) {
        let mut xg = GizmoAsset::new();
        let mut yg = GizmoAsset::new();
        let mut zg = GizmoAsset::new();

        xg.arrow(Vec3::ZERO, Vec3::X * 15., Color::linear_rgb(1., 0., 0.));
        yg.arrow(Vec3::ZERO, Vec3::Y * 15., Color::linear_rgb(0., 1., 0.));
        zg.arrow(Vec3::ZERO, Vec3::Z * 15., Color::linear_rgb(0., 0., 1.));

        let mut assets = world.resource_mut::<Assets<GizmoAsset>>();

        let xg = assets.add(xg);
        let yg = assets.add(yg);
        let zg = assets.add(zg);

        let mut commands: Commands = world.commands();

        let xe = commands
            .spawn((
                Gizmo {
                    handle: xg,
                    ..Default::default()
                },
                Transform::IDENTITY,
                ChildOf(ctx.entity),
            ))
            .id();

        let ye = commands
            .spawn((
                Gizmo {
                    handle: yg,
                    ..Default::default()
                },
                Transform::IDENTITY,
                ChildOf(ctx.entity),
            ))
            .id();
        let ze = commands
            .spawn((
                Gizmo {
                    handle: zg,
                    ..Default::default()
                },
                Transform::IDENTITY,
                ChildOf(ctx.entity),
            ))
            .id();

        let mut p = world
            .get_mut::<AxisPointer>(ctx.entity)
            .expect("About to add Pointer");
        p.x = xe;
        p.y = ye;
        p.z = ze;
    }
    fn on_remove(
        mut world: bevy::ecs::world::DeferredWorld,
        ctx: bevy::ecs::component::HookContext,
    ) {
        let p = world
            .get::<AxisPointer>(ctx.entity)
            .expect("About to remove Pointer");
        let x = p.x;
        let y = p.y;
        let z = p.z;
        let mut commands = world.commands();
        commands.entity(x).despawn();
        commands.entity(y).despawn();
        commands.entity(z).despawn();
    }
}
