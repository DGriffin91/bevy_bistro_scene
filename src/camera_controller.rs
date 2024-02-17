// Copied from https://github.com/DGriffin91/bevy_basic_camera

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};

/// Provides basic movement functionality to the attached camera
#[derive(Component, Clone)]
pub struct CameraController {
    pub enabled: bool,
    pub initialized: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub mouse_key_enable_mouse: MouseButton,
    pub keyboard_key_enable_mouse: KeyCode,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
    pub orbit_focus: Vec3,
    pub orbit_mode: bool,
    pub scroll_wheel_speed: f32,
    pub lock_y: bool,
}

impl CameraController {
    pub fn print_controls(self) -> Self {
        println!(
            "
===============================
======= Camera Controls =======
===============================
    {:?} - Forward
    {:?} - Backward
    {:?} - Left
    {:?} - Right
    {:?} - Up
    {:?} - Down
    {:?} - Run
    {:?}/{:?} - EnableMouse
",
            self.key_forward,
            self.key_back,
            self.key_left,
            self.key_right,
            self.key_up,
            self.key_down,
            self.key_run,
            self.mouse_key_enable_mouse,
            self.keyboard_key_enable_mouse,
        );
        self
    }
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 0.25,
            key_forward: KeyCode::KeyW,
            key_back: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_up: KeyCode::KeyE,
            key_down: KeyCode::KeyQ,
            key_run: KeyCode::ShiftLeft,
            mouse_key_enable_mouse: MouseButton::Left,
            keyboard_key_enable_mouse: KeyCode::KeyM,
            walk_speed: 5.0,
            run_speed: 15.0,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
            orbit_focus: Vec3::ZERO,
            orbit_mode: false,
            scroll_wheel_speed: 0.1,
            lock_y: false,
        }
    }
}

pub fn camera_controller(
    time: Res<Time>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut scroll_evr: EventReader<MouseWheel>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut move_toggled: Local<bool>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    let dt = time.delta_seconds();

    if let Ok((mut transform, mut options)) = query.get_single_mut() {
        if !options.initialized {
            let (_roll, yaw, pitch) = transform.rotation.to_euler(EulerRot::ZYX);
            options.yaw = yaw;
            options.pitch = pitch;
            options.initialized = true;
        }
        if !options.enabled {
            return;
        }

        let mut scroll_distance = 0.0;

        // Handle scroll input
        for ev in scroll_evr.read() {
            match ev.unit {
                MouseScrollUnit::Line => {
                    scroll_distance = ev.y;
                }
                MouseScrollUnit::Pixel => (),
            }
        }

        // Handle key input
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(options.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(options.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(options.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(options.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(options.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(options.key_down) {
            axis_input.y -= 1.0;
        }
        if key_input.just_pressed(options.keyboard_key_enable_mouse) {
            *move_toggled = !*move_toggled;
        }

        // Apply movement update
        if axis_input != Vec3::ZERO {
            let max_speed = if key_input.pressed(options.key_run) {
                options.run_speed
            } else {
                options.walk_speed
            };
            options.velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = options.friction.clamp(0.0, 1.0);
            options.velocity *= 1.0 - friction;
            if options.velocity.length_squared() < 1e-6 {
                options.velocity = Vec3::ZERO;
            }
        }
        let forward = transform.forward();
        let right = transform.right();
        let mut translation_delta = options.velocity.x * dt * *right
            + options.velocity.y * dt * Vec3::Y
            + options.velocity.z * dt * *forward;
        let mut scroll_translation = Vec3::ZERO;
        if options.orbit_mode && options.scroll_wheel_speed > 0.0 {
            scroll_translation = scroll_distance
                * transform.translation.distance(options.orbit_focus)
                * options.scroll_wheel_speed
                * *forward;
        }
        if options.lock_y {
            translation_delta *= Vec3::new(1.0, 0.0, 1.0);
        }
        transform.translation += translation_delta + scroll_translation;
        options.orbit_focus += translation_delta;

        // Handle mouse input
        let mut mouse_delta = Vec2::ZERO;
        if mouse_button_input.pressed(options.mouse_key_enable_mouse) || *move_toggled {
            for mouse_event in mouse_events.read() {
                mouse_delta += mouse_event.delta;
            }
        } else {
            mouse_events.clear();
        }

        if mouse_delta != Vec2::ZERO {
            let sensitivity = if options.orbit_mode {
                options.sensitivity * 2.0
            } else {
                options.sensitivity
            };
            let (pitch, yaw) = (
                (options.pitch - mouse_delta.y * 0.5 * sensitivity * dt).clamp(
                    -0.99 * std::f32::consts::FRAC_PI_2,
                    0.99 * std::f32::consts::FRAC_PI_2,
                ),
                options.yaw - mouse_delta.x * sensitivity * dt,
            );

            // Apply look update
            transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
            options.pitch = pitch;
            options.yaw = yaw;

            if options.orbit_mode {
                let rot_matrix = Mat3::from_quat(transform.rotation);
                transform.translation = options.orbit_focus
                    + rot_matrix.mul_vec3(Vec3::new(
                        0.0,
                        0.0,
                        options.orbit_focus.distance(transform.translation),
                    ));
            }
        }
    }
}

/// Simple flying camera plugin.
/// In order to function, the [`CameraController`] component should be attached to the camera entity.
#[derive(Default)]
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_controller);
    }
}
