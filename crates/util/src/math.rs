use bevy::{
    ecs::component::Component,
    math::{Vec3, Vec3A},
    prelude::{Deref, DerefMut},
};

#[derive(Component, Deref, DerefMut)]
pub struct AabbComponent(pub Aabb);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aabb {
    pub center: Vec3A,
    pub half_extents: Vec3A,
}

impl Aabb {
    pub fn from_min_max(minimum: Vec3, maximum: Vec3) -> Self {
        let minimum = Vec3A::from(minimum);
        let maximum = Vec3A::from(maximum);
        let center = 0.5 * (maximum + minimum);
        let half_extents = 0.5 * (maximum - minimum);
        Self {
            center,
            half_extents,
        }
    }

    #[inline(always)]
    pub fn min(&self) -> Vec3A {
        self.center - self.half_extents
    }

    #[inline(always)]
    pub fn max(&self) -> Vec3A {
        self.center + self.half_extents
    }
}

pub fn intersect_aabb(lhs: (&Vec3, &Aabb), rhs: (&Vec3, &Aabb)) -> bool {
    let lhs_half_extents: Vec3 = lhs.1.half_extents.into();
    let rhs_half_extents: Vec3 = rhs.1.half_extents.into();

    let lhs = Aabb::from_min_max(*lhs.0 - lhs_half_extents, *lhs.0 + lhs_half_extents);
    let rhs = Aabb::from_min_max(*rhs.0 - rhs_half_extents, *rhs.0 + rhs_half_extents);

    lhs.min().cmple(rhs.max()).all() && lhs.max().cmpge(rhs.min()).all()
}

fn _calculate_normals(indices: &Vec<u32>, vertices: &[[f32; 3]], normals: &mut [[f32; 3]]) {
    let vertex_count = indices.len();

    for i in (0..vertex_count).step_by(3) {
        let v1 = Vec3::from_array(vertices[indices[i + 1] as usize])
            - Vec3::from_array(vertices[indices[i] as usize]);
        let v2 = Vec3::from_array(vertices[indices[i + 2] as usize])
            - Vec3::from_array(vertices[indices[i] as usize]);
        let face_normal = v1.cross(v2).normalize();

        // Add the face normal to the 3 vertex normals that are touching this face
        normals[indices[i] as usize] =
            (Vec3::from_array(normals[indices[i] as usize]) + face_normal).to_array();
        normals[indices[i + 1] as usize] =
            (Vec3::from_array(normals[indices[i + 1] as usize]) + face_normal).to_array();
        normals[indices[i + 2] as usize] =
            (Vec3::from_array(normals[indices[i + 2] as usize]) + face_normal).to_array();
    }

    // Now loop through each vertex vector, and avarage out all the normals stored.
    for normal in &mut normals.iter_mut() {
        *normal = Vec3::from_array(*normal).normalize().to_array();
    }
}

fn _make_indices(indices: &mut Vec<u32>, size: [u32; 2]) {
    for y in 0..size[1] - 1 {
        for x in 0..size[0] - 1 {
            let index = y * size[0] + x;
            indices.push(index + size[0] + 1);
            indices.push(index + 1);
            indices.push(index + size[0]);
            indices.push(index);
            indices.push(index + size[0]);
            indices.push(index + 1);
        }
    }
}
