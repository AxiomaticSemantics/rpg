use bevy::{
    math::{Quat, Vec3, Vec4},
    render::color::Color,
};

pub fn to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

pub fn to_vec3(vec: Vec<f32>) -> Vec3 {
    Vec3::from_slice(vec.as_slice())
}

pub fn to_vec4(vec: Vec<f32>) -> Vec4 {
    Vec4::from_slice(vec.as_slice())
}

pub fn to_color_rgb(vec: Vec<f32>) -> Option<Color> {
    if vec.len() == 3 {
        Some(Color::rgb(vec[0], vec[1], vec[2]))
    } else {
        None
    }
}

pub fn to_color_rgba(vec: Vec<f32>) -> Option<Color> {
    if vec.len() == 4 {
        Some(Color::rgba(vec[0], vec[1], vec[2], vec[3]))
    } else {
        None
    }
}
