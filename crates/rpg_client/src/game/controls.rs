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
        keyboard::KeyCode,
        mouse::{MouseButton, MouseMotion, MouseWheel},
        ButtonInput,
    },
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
    pub just_pressed: bool,
    pub pressed: bool,
    pub just_released: bool,
}

#[derive(Resource, Debug, Default)]
pub struct Controls {
    pub mouse_primary: ButtonState,
    pub mouse_secondary: ButtonState,
    pub mouse_wheel_delta: f32,
    pub mouse_motion: Vec2,
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
    window_q: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut controls: ResMut<Controls>,
    mut cursor_position: ResMut<CursorPosition>,
) {
    controls.escape.pressed = keyboard_input.pressed(KeyCode::Escape);
    controls.escape.just_pressed = keyboard_input.just_pressed(KeyCode::Escape);
    controls.escape.just_released = keyboard_input.just_released(KeyCode::Escape);

    controls.space.pressed = keyboard_input.pressed(KeyCode::Space);
    controls.space.just_pressed = keyboard_input.just_pressed(KeyCode::Space);
    controls.space.just_released = keyboard_input.just_released(KeyCode::Space);

    let value = mouse_wheel.read().fold(0., |sum, v| sum + v.y);
    controls.mouse_wheel_delta = value;

    controls.mouse_motion = mouse_motion.read().fold(Vec2::ZERO, |sum, v| sum + v.delta);

    controls.mouse_primary.pressed = mouse_input.pressed(MouseButton::Left);
    controls.mouse_primary.just_pressed = mouse_input.just_pressed(MouseButton::Left);
    controls.mouse_primary.just_released = mouse_input.just_released(MouseButton::Left);

    controls.mouse_secondary.pressed = mouse_input.pressed(MouseButton::Right);
    controls.mouse_secondary.just_pressed = mouse_input.just_pressed(MouseButton::Right);
    controls.mouse_secondary.just_released = mouse_input.just_released(MouseButton::Right);

    let (camera, transform) = camera_q.single();
    let window = window_q.single();

    let Some(position) = window.cursor_position() else {
        return;
    };
    let Some(ray) = camera.viewport_to_world(transform, position) else {
        println!("could not convert viewport position to world");
        return;
    };

    let Some(ground_distance) = ray.intersect_plane(Vec3::new(0., 0., 0.), Plane3d::new(Vec3::Y))
    else {
        println!("could not determine ground distance");
        return;
    };
    let ground_point = ray.get_point(ground_distance);

    let Some(body_distance) = ray.intersect_plane(Vec3::new(0., 1.2, 0.), Plane3d::new(Vec3::Y))
    else {
        println!("could not determine body distance");
        return;
    };
    let body_point = ray.get_point(body_distance);

    cursor_position.screen = position;
    cursor_position.ground = ground_point;
    cursor_position.body = body_point;
}
