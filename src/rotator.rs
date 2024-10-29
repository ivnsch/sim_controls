use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

/// Based on Valorant's default sensitivity, not entirely sure why it is exactly 1.0 / 180.0,
/// but I'm guessing it is a misunderstanding between degrees/radians and then sticking with
/// it because it felt nice.
#[allow(unused)]
pub const RADIANS_PER_DOT: f32 = 1.0 / 180.0;

#[allow(unused)]
#[derive(Component)]
pub struct MouseController {
    pub mouse_key_cursor_grab: MouseButton,
    pub rot_pars: RotPars,
}

impl Default for MouseController {
    fn default() -> Self {
        Self {
            mouse_key_cursor_grab: MouseButton::Left,
            rot_pars: RotPars { sensitivity: 0.01 },
        }
    }
}

pub struct RotPars {
    pub sensitivity: f32,
}

#[allow(unused)]
#[derive(Component, Debug)]
pub struct Rotator {
    pub key_y: KeyCode,
    pub key_z: KeyCode,
    pub key_x: KeyCode,
    pub key_i: KeyCode,
    pub key_o: KeyCode,
    pub key_p: KeyCode,
    pub key_shift_left: KeyCode,
    pub key_shift_right: KeyCode,
}

impl Default for Rotator {
    fn default() -> Self {
        Self {
            key_y: KeyCode::KeyY,
            key_z: KeyCode::KeyZ,
            key_x: KeyCode::KeyX,
            key_i: KeyCode::KeyI,
            key_o: KeyCode::KeyO,
            key_p: KeyCode::KeyP,
            key_shift_left: KeyCode::ShiftLeft,
            key_shift_right: KeyCode::ShiftRight,
        }
    }
}

#[allow(clippy::too_many_arguments, unused)]
fn run_camera_rotator(
    key_input: Res<ButtonInput<KeyCode>>,
    mut camera: Query<(&mut Transform, &mut Rotator), With<Camera>>,
) {
    if let Ok((mut transform, mut rotator)) = camera.get_single_mut() {
        rotate(
            key_input,
            &rotator,
            &mut transform,
            rotator.key_x,
            rotator.key_y,
            rotator.key_z,
        );
    }
}

pub fn rotate(
    key_input: Res<ButtonInput<KeyCode>>,
    rotator: &Rotator,
    transform: &mut Transform,
    x_key: KeyCode,
    y_key: KeyCode,
    z_key: KeyCode,
) {
    let mut rotation = 0.03;
    if key_input.pressed(rotator.key_shift_left) || key_input.pressed(rotator.key_shift_right) {
        rotation = -rotation;
    }

    if key_input.pressed(y_key) {
        transform.rotate_around(
            Vec3::ZERO,
            Quat::from_euler(EulerRot::XYZ, 0.0, rotation, 0.0),
        );
    }
    if key_input.pressed(z_key) {
        transform.rotate_around(
            Vec3::ZERO,
            Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, rotation),
        );
    }
    if key_input.pressed(x_key) {
        transform.rotate_around(
            Vec3::ZERO,
            Quat::from_euler(EulerRot::XYZ, rotation, 0.0, 0.0),
        );
    }
}

pub fn handle_mouse(
    windows: &mut Query<&mut Window>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_cursor_grab: Local<bool>, // whether there's an active grab (user pressing mouse/trackpad)
    transform: &mut Transform,
    controller: &mut MouseController,
) {
    // get current gesture (just started pressing or released)
    let grab: Option<CursorGrabInput> =
        cursor_grab_update(mouse_button_input, controller.mouse_key_cursor_grab);

    // determine whether currently there's an active grab
    let update_status = match &grab {
        // just did something: map to status and save
        Some(grab) => {
            let status = match grab {
                CursorGrabInput::JustPressed => CursorGrabStatus::Active,
                CursorGrabInput::JustReleased => CursorGrabStatus::Inactive,
            };
            // save current state (no-op if user just didn't do anything)
            *mouse_cursor_grab = match &status {
                CursorGrabStatus::Active => true,
                CursorGrabStatus::Inactive => false,
            };
            status
        }
        // didn't do anything: use current state
        None => match *mouse_cursor_grab {
            true => CursorGrabStatus::Active,
            false => CursorGrabStatus::Inactive,
        },
    };

    // if there was a gesture, do cursor and window updates
    if let Some(input) = &grab {
        update_cursor_and_window_for_grab_input(windows, &mut mouse_events, input);
    };

    // rotate mouse during active grab
    match &update_status {
        CursorGrabStatus::Active => {
            update_rotation_with_mouse(&mut mouse_events, transform, &mut controller.rot_pars)
        }
        CursorGrabStatus::Inactive => {}
    };
}

/// updates cursor visibility and window focus for a given grab input
pub fn update_cursor_and_window_for_grab_input(
    windows: &mut Query<&mut Window>,
    mouse_events: &mut EventReader<MouseMotion>,
    input: &CursorGrabInput,
) {
    match input {
        CursorGrabInput::JustPressed => {
            for mut window in windows {
                if !window.focused {
                    continue;
                }
                window.cursor.grab_mode = CursorGrabMode::Locked;
                window.cursor.visible = false;
            }
        }
        CursorGrabInput::JustReleased => {
            for mut window in windows {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            }
            mouse_events.clear()
        }
    }
}

fn update_rotation_with_mouse(
    mouse_events: &mut EventReader<MouseMotion>,
    transform: &mut Transform,
    rot_pars: &mut RotPars,
) {
    let mut mouse_delta = Vec2::ZERO;

    for mouse_event in mouse_events.read() {
        mouse_delta += mouse_event.delta * rot_pars.sensitivity;

        transform.rotate_around(
            Vec3::ZERO,
            Quat::from_euler(EulerRot::XYZ, mouse_delta.y, 0., 0.),
        );
        transform.rotate_around(
            Vec3::ZERO,
            Quat::from_euler(EulerRot::XYZ, 0., mouse_delta.x, 0.),
        );
    }
}

pub fn cursor_grab_update(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    button: MouseButton,
) -> Option<CursorGrabInput> {
    // Important: don't change ordering, sometimes pressed-release is delivered at the same time,
    // which we have to map to release, so we ask for release first
    if mouse_button_input.just_released(button) {
        return Some(CursorGrabInput::JustReleased);
    } else if mouse_button_input.just_pressed(button) {
        return Some(CursorGrabInput::JustPressed);
    }
    None
}

#[derive(Debug)]
pub enum CursorGrabInput {
    JustPressed,
    JustReleased,
}

#[derive(Debug)]
pub enum CursorGrabStatus {
    Active,
    Inactive,
}
