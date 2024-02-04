use bevy::{
    ecs::component::Component,
    math::{
        bounding::{Aabb3d, Bounded3d, IntersectsVolume},
        primitives::Cuboid,
        Quat, Vec3,
    },
    prelude::{Deref, DerefMut},
};

#[derive(Component, Deref, DerefMut)]
pub struct AabbComponent(pub Aabb3d);

pub fn intersect_aabb(lhs: (Vec3, Quat, Aabb3d), rhs: (Vec3, Quat, Aabb3d)) -> bool {
    let lhs_extents: Vec3 = lhs.2.max - lhs.2.min;
    let rhs_extents: Vec3 = rhs.2.max - rhs.2.min;

    let lhs_cuboid = Cuboid::from_size(lhs_extents);
    let rhs_cuboid = Cuboid::from_size(rhs_extents);

    let lhs = lhs_cuboid.aabb_3d(lhs.0, lhs.1);
    let rhs = rhs_cuboid.aabb_3d(rhs.0, rhs.1);

    lhs.intersects(&rhs)
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

/*
    let mut image = images.get_mut(&textures.heightmap).unwrap();
    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        address_mode_u: AddressMode::MirrorRepeat,
        address_mode_v: AddressMode::MirrorRepeat,
        address_mode_w: AddressMode::Repeat,
        ..default()
    });

    let image_size = image.size();
    let size = image_size.x as u32;
    let size_y = size - 1;
    let size_x = size - 1;
    let num_vertices = (size_y * size_x) as usize;
    let num_indices = ((size_y - 1) * (size_x - 1) * 6) as usize;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
    let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

    let mut uv: [f32; 2] = [0., 1.];
    for y in 0..size_y {
        if y % 8 == 0 {
            uv[1] = 1.;
        }

        for x in 0..size_x {
            if x % 8 == 0 {
                uv[0] = 0.;
            }

            let index = y * size_x + x;

            let h = *image.data.get(index as usize * 4).unwrap();
            let h_s = (size_x - 1) as f32 / 2.;

            let pos = Vec3::new(-h_s + x as f32, -16. + (h as f32) / 8., -h_s + y as f32);
            positions.push(pos.to_array());
            normals.push([0., 0., 0.]);
            uvs.push(uv);

            println!("UV: {uv:?}");

            uv[0] += 0.142857142857;
        }
        uv[1] -= 0.142857142857;
    }

    make_indices(&mut indices, [size_x, size_y]);
    calculate_normals(&indices, &positions, &mut normals);

    let mut terrain_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    terrain_mesh.set_indices(Some(Indices::U32(indices)));
    terrain_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    terrain_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    terrain_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    commands.spawn(PbrBundle {
    mesh: meshes.add(terrain_mesh),
    material: materials.add(StandardMaterial {
        base_color_texture: Some(textures.seamless_grass.clone()),
        perceptual_roughness: 0.95,
        ..default()
        }),
        ..default()
        });
        transform: Transform::from_xyz(32., 32., 0.),
        ..default()
    });
*/
