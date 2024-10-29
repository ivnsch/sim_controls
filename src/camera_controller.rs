//! A freecam-style camera controller plugin.
//! To use in your own application:
//! - Copy the code for the [`CameraControllerPlugin`] and add the plugin to your App.
//! - Attach the [`CameraController`] component to an entity with a [`Camera3dBundle`].

use crate::rotator::{
    cursor_grab_update, update_cursor_and_window_for_grab_input, CursorGrabInput, CursorGrabStatus,
};
use bevy::{input::mouse::MouseMotion, prelude::*};

pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                run_camera_controller,
                // if testing this, disable the mouse_handler mouse_handler as both use the same mouse event
                // mouse_handler
            ),
        );
    }
}

#[derive(Component)]
pub struct CameraController {
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    mouse_key_cursor_zoom: MouseButton,
    zoom_sensitivity: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            key_forward: KeyCode::KeyW,
            key_back: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_up: KeyCode::KeyE,
            key_down: KeyCode::KeyQ,
            mouse_key_cursor_zoom: MouseButton::Left,
            zoom_sensitivity: 0.3,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn run_camera_controller(
    key_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    if let Ok((mut transform, mut controller)) = query.get_single_mut() {
        handle_keyboard(&key_input, &mut transform, &mut controller);
    }
}

const TRANSLATION_SPEED: f32 = 0.8; // base speed (unscaled by TRANSLATION_SCALING_FACTOR)
const TRANSLATION_SCALING_FACTOR: f32 = 0.05; // lower to slow down more quickly
const TRANSLATION_MIN_UPDATE: f32 = 0.3; // lower speed boundary

fn handle_keyboard(
    key_input: &Res<ButtonInput<KeyCode>>,
    transform: &mut Transform,
    controller: &mut CameraController,
) {
    if key_input.pressed(controller.key_forward) {
        update_with_scaling(&mut transform.translation.z, false);
    }
    if key_input.pressed(controller.key_back) {
        update_with_scaling(&mut transform.translation.z, true)
    }
    if key_input.pressed(controller.key_right) {
        update_with_scaling(&mut transform.translation.x, true);
    }
    if key_input.pressed(controller.key_left) {
        update_with_scaling(&mut transform.translation.x, false);
    }
    if key_input.pressed(controller.key_up) {
        update_with_scaling(&mut transform.translation.y, true);
    }
    if key_input.pressed(controller.key_down) {
        update_with_scaling(&mut transform.translation.y, false);
    }
}

/// move faster when far away
/// assumes object being looked at centered at 0,0,0
/// (in user experience sense, in that we move faster because we're traveling through assumed empty/mostly empty space)
fn update_with_scaling(dim: &mut f32, add: bool) {
    *dim += (*dim * TRANSLATION_SCALING_FACTOR * TRANSLATION_SPEED).max(TRANSLATION_MIN_UPDATE)
        * if add { 1. } else { -1. };
}

/// TODO re-implement using MouseWheel
#[allow(dead_code)]
fn mouse_handler(
    mut windows: Query<&mut Window>,
    mut mouse_events: EventReader<MouseMotion>,
    mut mouse_cursor_grab: Local<bool>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut camera: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    if let Ok((mut transform, controller)) = camera.get_single_mut() {
        // get current gesture (just started pressing or released)
        let grab = cursor_grab_update(mouse_button_input, controller.mouse_key_cursor_zoom);

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
            update_cursor_and_window_for_grab_input(&mut windows, &mut mouse_events, input);
        };

        // zoom during active grab
        match &update_status {
            CursorGrabStatus::Active => update_zoom_with_mouse(
                &mut mouse_events,
                &mut transform,
                controller.zoom_sensitivity,
            ),
            CursorGrabStatus::Inactive => {}
        };
    }
}

fn update_zoom_with_mouse(
    mouse_events: &mut EventReader<MouseMotion>,
    transform: &mut Transform,
    sensitivity: f32,
) {
    let mut mouse_delta = Vec2::ZERO;
    for mouse_event in mouse_events.read() {
        mouse_delta += mouse_event.delta;
        if mouse_delta != Vec2::ZERO {
            transform.translation.z -= mouse_event.delta.x + mouse_event.delta.y * sensitivity;
        }
    }
}
