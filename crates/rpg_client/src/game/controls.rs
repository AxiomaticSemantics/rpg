#![allow(clippy::too_many_arguments)]

use super::plugin::GameCamera;

use bevy::{
    ecs::{
        event::EventReader,
        query::With,
        system::Query,
        system::{Res, ResMut, Resource},
    },
    input::{
        gamepad::{
            Gamepad, GamepadAxisChangedEvent, GamepadAxisType, GamepadButton, GamepadButtonType,
        },
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion, MouseWheel},
        ButtonInput,
    },
    log::error,
    math::{primitives::Plane3d, Vec2, Vec3},
    render::camera::Camera,
    transform::components::GlobalTransform,
    window::{PrimaryWindow, Window},
};

#[derive(Resource, Debug, Default)]
pub struct CursorPosition {
    pub screen: Vec2,
    pub ground: Vec3,
    pub body: Vec3,
}

#[derive(Debug, Default)]
pub struct ButtonState {
    pub pressed: bool,
    pub just_pressed: bool,
    pub just_released: bool,
}

#[derive(Resource, Debug, Default)]
pub struct Controls {
    pub mouse_primary: ButtonState,
    pub mouse_secondary: ButtonState,
    pub mouse_wheel_delta: f32,
    pub mouse_motion: Vec2,
    pub gamepad_axis_left: Vec2,
    pub gamepad_axis_right: Vec2,
    pub gamepad_a: ButtonState,
    pub gamepad_b: ButtonState,
    pub gamepad_c: ButtonState,
    pub gamepad_d: ButtonState,
    pub gamepad_lt_a: ButtonState,
    pub gamepad_lt_b: ButtonState,
    pub gamepad_rt_a: ButtonState,
    pub gamepad_rt_b: ButtonState,
    pub escape: ButtonState,
    pub space: ButtonState,
    pub inhibited: bool,
}

impl Controls {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn set_inhibited(&mut self, inhibited: bool) {
        self.inhibited = inhibited;
    }

    pub fn is_inhibited(&self) -> bool {
        self.inhibited
    }
}

pub fn update_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut window_q: Query<&mut Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    gamepad_input: Res<ButtonInput<GamepadButton>>,
    mut gamepad_axis: EventReader<GamepadAxisChangedEvent>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut controls: ResMut<Controls>,
    mut cursor_position: ResMut<CursorPosition>,
) {
    for axis_event in gamepad_axis.read() {
        match axis_event.axis_type {
            GamepadAxisType::LeftStickX => controls.gamepad_axis_left.x = axis_event.value,
            GamepadAxisType::LeftStickY => controls.gamepad_axis_left.y = axis_event.value,
            GamepadAxisType::RightStickX => controls.gamepad_axis_right.x = axis_event.value,
            GamepadAxisType::RightStickY => controls.gamepad_axis_right.y = axis_event.value,
            _ => {}
        }
    }

    let mut window = window_q.single_mut();

    if controls.gamepad_axis_left != Vec2::ZERO {
        let new_cursor_position = Vec2::new(
            cursor_position.screen.x + 25. * controls.gamepad_axis_left.x,
            cursor_position.screen.y - 25. * controls.gamepad_axis_left.y,
        );
        window.set_cursor_position(Some(new_cursor_position));
    }

    let button_south = GamepadButton::new(Gamepad::new(0), GamepadButtonType::South);
    let button_east = GamepadButton::new(Gamepad::new(0), GamepadButtonType::East);
    let button_north = GamepadButton::new(Gamepad::new(0), GamepadButtonType::North);
    let button_west = GamepadButton::new(Gamepad::new(0), GamepadButtonType::West);
    let button_lt_a = GamepadButton::new(Gamepad::new(0), GamepadButtonType::LeftTrigger);
    let button_lt_b = GamepadButton::new(Gamepad::new(0), GamepadButtonType::LeftTrigger2);
    let button_rt_a = GamepadButton::new(Gamepad::new(0), GamepadButtonType::RightTrigger);
    let button_rt_b = GamepadButton::new(Gamepad::new(0), GamepadButtonType::RightTrigger2);

    controls.gamepad_a.pressed = gamepad_input.pressed(button_south);
    controls.gamepad_a.just_pressed = gamepad_input.just_pressed(button_south);
    controls.gamepad_a.just_released = gamepad_input.just_released(button_south);

    controls.gamepad_b.pressed = gamepad_input.pressed(button_east);
    controls.gamepad_b.just_pressed = gamepad_input.just_pressed(button_east);
    controls.gamepad_b.just_released = gamepad_input.just_released(button_east);

    controls.gamepad_c.pressed = gamepad_input.pressed(button_north);
    controls.gamepad_c.just_pressed = gamepad_input.just_pressed(button_north);
    controls.gamepad_c.just_released = gamepad_input.just_released(button_north);

    controls.gamepad_d.pressed = gamepad_input.pressed(button_west);
    controls.gamepad_d.just_pressed = gamepad_input.just_pressed(button_west);
    controls.gamepad_d.just_released = gamepad_input.just_released(button_west);

    controls.gamepad_lt_a.pressed = gamepad_input.pressed(button_lt_a);
    controls.gamepad_lt_a.just_pressed = gamepad_input.just_pressed(button_lt_a);
    controls.gamepad_lt_a.just_released = gamepad_input.just_released(button_lt_a);

    controls.gamepad_lt_b.pressed = gamepad_input.pressed(button_lt_b);
    controls.gamepad_lt_b.just_pressed = gamepad_input.just_pressed(button_lt_b);
    controls.gamepad_lt_b.just_released = gamepad_input.just_released(button_lt_b);

    controls.gamepad_rt_a.pressed = gamepad_input.pressed(button_rt_a);
    controls.gamepad_rt_a.just_pressed = gamepad_input.just_pressed(button_rt_a);
    controls.gamepad_rt_a.just_released = gamepad_input.just_released(button_rt_a);

    controls.gamepad_rt_b.pressed = gamepad_input.pressed(button_rt_b);
    controls.gamepad_rt_b.just_pressed = gamepad_input.just_pressed(button_rt_b);
    controls.gamepad_rt_b.just_released = gamepad_input.just_released(button_rt_b);

    controls.escape.pressed = keyboard_input.pressed(KeyCode::Escape);
    controls.escape.just_pressed = keyboard_input.just_pressed(KeyCode::Escape);
    controls.escape.just_released = keyboard_input.just_released(KeyCode::Escape);

    controls.space.pressed = keyboard_input.pressed(KeyCode::Space);
    controls.space.just_pressed = keyboard_input.just_pressed(KeyCode::Space);
    controls.space.just_released = keyboard_input.just_released(KeyCode::Space);

    controls.mouse_wheel_delta = mouse_wheel.read().fold(0., |sum, v| sum + v.y);
    controls.mouse_motion = mouse_motion.read().fold(Vec2::ZERO, |sum, v| sum + v.delta);

    controls.mouse_primary.pressed = mouse_input.pressed(MouseButton::Left);
    controls.mouse_primary.just_pressed = mouse_input.just_pressed(MouseButton::Left);
    controls.mouse_primary.just_released = mouse_input.just_released(MouseButton::Left);

    controls.mouse_secondary.pressed = mouse_input.pressed(MouseButton::Right);
    controls.mouse_secondary.just_pressed = mouse_input.just_pressed(MouseButton::Right);
    controls.mouse_secondary.just_released = mouse_input.just_released(MouseButton::Right);

    let (camera, transform) = camera_q.single();

    let Some(position) = window.cursor_position() else {
        return;
    };

    let Some(ray) = camera.viewport_to_world(transform, position) else {
        error!("could not convert viewport position to world");
        return;
    };

    let Some(ground_distance) = ray.intersect_plane(Vec3::ZERO, Plane3d::new(Vec3::Y)) else {
        error!("could not determine ground distance");
        return;
    };
    let ground_point = ray.get_point(ground_distance);

    let Some(body_distance) = ray.intersect_plane(Vec3::new(0., 1.2, 0.), Plane3d::new(Vec3::Y))
    else {
        error!("could not determine body distance");
        return;
    };
    let body_point = ray.get_point(body_distance);

    cursor_position.screen = position;
    cursor_position.ground = ground_point;
    cursor_position.body = body_point;
}
