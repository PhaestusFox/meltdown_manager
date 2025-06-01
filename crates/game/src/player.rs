use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (
            player_look,
            player_move,
            focus_events,
            toggle_grab,
            change_speed,
        ),
    );
    app.add_observer(apply_grab);
}

#[derive(Component)]
pub struct Player {
    pub speed: f32,
}

fn player_look(
    // should only be on player you control so use single so save unwrapping
    // if no player is found this system will not run
    // if more then one player is found the system will also not run
    mut player: Single<&mut Transform, With<Player>>,
    //can use raw events
    // mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mouse_movement: Res<AccumulatedMouseMotion>,
    // use window to check if we should let the player look or not
    window: Single<&Window, With<PrimaryWindow>>,
    inputs: Res<ButtonInput<KeyCode>>,
) {
    // if using MouseMotion events need to accumulate them
    // let delta = mouse_motion.read().map(|e| e.delta).sum();

    // if window is not focused don't let player look
    if !window.focused {
        return;
    }

    if inputs.pressed(KeyCode::F10) {
        println!("Mouse movement: {:?}", mouse_movement.delta);
    }

    // change to use 100. divided by min width and hight, this will make the game feel the same even on different resolutions
    let sensitivity = 1. / window.width().min(window.height());

    //get angles as euler angles because they are more natural then Quats, don't need role
    let (mut yaw, mut pitch, _) = player.rotation.to_euler(EulerRot::YXZ);
    // subtract y movement for pitch - up/down
    pitch -= mouse_movement.delta.y * sensitivity;

    // subtract x movement for yaw - left/right
    yaw -= mouse_movement.delta.x * sensitivity;

    // stops you looking past straight up, it will flickering as the value becomes negative
    pitch = pitch.clamp(-1.57, 1.57);

    // recalculate the Quat from the yaw and pitch, yaw first or we end up with unintended role
    player.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.);
}

#[derive(Event, Deref)]
struct GrabEvent(bool);

fn apply_grab(
    // tells bevy what event to watch for with this observer
    grab: Trigger<GrabEvent>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if **grab {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked
    } else {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
}

fn focus_events(mut events: EventReader<bevy::window::WindowFocused>, mut commands: Commands) {
    if let Some(event) = events.read().last() {
        commands.trigger(GrabEvent(event.focused));
    }
}

fn toggle_grab(
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut commands: Commands,
    inputs: Res<ButtonInput<KeyCode>>,
) {
    if !inputs.just_pressed(KeyCode::Escape) {
        return;
    }
    window.focused = !window.focused;
    commands.trigger(GrabEvent(window.focused));
}

fn player_move(
    // need access to player
    mut player: Single<(&mut Transform, &Player)>,
    // need access to keyboard inputs
    inputs: Res<ButtonInput<KeyCode>>,
    // need delta time to update position consistently even during lag or non 60 fps
    time: Res<Time>,
) {
    let mut delta = Vec3::ZERO;
    if inputs.pressed(KeyCode::KeyA) {
        delta.x -= 1.;
    }
    if inputs.pressed(KeyCode::KeyD) {
        delta.x += 1.;
    }
    if inputs.pressed(KeyCode::KeyW) {
        delta.z += 1.;
    }
    if inputs.pressed(KeyCode::KeyS) {
        delta.z -= 1.;
    }
    if inputs.pressed(KeyCode::Space) {
        delta.y += 1.;
    }
    if inputs.pressed(KeyCode::ControlLeft) {
        delta.y -= 1.;
    }

    let (ref mut player, Player { speed }) = *player;

    let forward = player.forward().as_vec3() * delta.z;
    let left = player.right().as_vec3() * delta.x;
    let mut to_move = forward + left;
    to_move.y = 0.;
    to_move = to_move.normalize_or_zero();
    to_move.y = delta.y;
    if inputs.pressed(KeyCode::ShiftLeft) {
        to_move *= 2.
    }
    player.translation += to_move.normalize_or_zero() * time.delta_secs() * speed;
}

fn change_speed(mut player: Single<&mut Player>, inputs: Res<ButtonInput<KeyCode>>) {
    let mut power = 1.;
    if inputs.pressed(KeyCode::ShiftLeft) {
        power = 5.;
    }
    if inputs.just_pressed(KeyCode::AltLeft) {
        power *= 10.;
    }

    if inputs.just_pressed(KeyCode::NumpadAdd) {
        player.speed += 10. * power;
    }
    if inputs.just_pressed(KeyCode::NumpadSubtract) {
        player.speed -= 10. * power;
        player.speed = player.speed.clamp(10., f32::INFINITY);
    }
    if inputs.just_pressed(KeyCode::NumpadMultiply) {
        player.speed = 50. * power;
    }
}
