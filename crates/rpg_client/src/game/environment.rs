use super::world::zone::Zone;
use crate::game::plugin::GameSessionCleanup;

use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    math::{Quat, Vec3},
    pbr::{
        AmbientLight, CascadeShadowConfigBuilder, DirectionalLight, DirectionalLightBundle,
        SpotLight, SpotLightBundle,
    },
    render::color::Color,
    time::Time,
    transform::components::Transform,
    utils::default,
};

use rpg_world::zone::Kind as ZoneKind;

use util::cleanup::CleanupStrategy;

#[derive(Component)]
pub(crate) struct EnvironmentDirectionalLight;

#[derive(Resource)]
pub struct Environment;

pub(crate) fn setup(mut commands: Commands, zone: Res<Zone>) {
    println!("environment::setup");

    let cascade_shadow_config = CascadeShadowConfigBuilder {
        first_cascade_far_bound: 32.,
        minimum_distance: 0.1,
        maximum_distance: 64.,
        ..default()
    }
    .build();

    // lighting

    // spotlight
    commands.spawn((
        GameSessionCleanup,
        CleanupStrategy::Despawn,
        SpotLightBundle {
            spot_light: SpotLight {
                range: 20.,
                //outer_angle: 0.6,
                //inner_angle: 0.3,
                shadows_enabled: true,
                ..default()
            },
            ..default()
        },
    ));

    // PointLight { intensity: 1000., range: 32., shadows_enabled: true, ..default() }

    if zone.zone.kind == ZoneKind::Overworld {
        commands.spawn((
            GameSessionCleanup,
            CleanupStrategy::Despawn,
            EnvironmentDirectionalLight,
            DirectionalLightBundle {
                transform: Transform::from_translation(Vec3::Y),
                directional_light: DirectionalLight {
                    color: Color::rgb(0.99, 0.99, 0.97),
                    shadows_enabled: true,
                    ..default()
                },
                cascade_shadow_config,
                ..default()
            },
        ));
    }

    println!("game environment spawn complete");
}

pub(crate) fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Environment>();
}

pub(crate) fn day_night_cycle(
    time: Res<Time>,
    mut light_q: Query<(&mut DirectionalLight, &mut Transform), With<EnvironmentDirectionalLight>>,
    mut spot_light_q: Query<&mut SpotLight>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    // Hacky trash code, but it's just to show the cycle is there, not for accuracy
    let day_length = 2. * 60.; // FIXME hard-coded for now
    let dt = time.delta_seconds();
    let rot = std::f32::consts::TAU * (dt / day_length);
    let (mut light, mut transform) = light_q.single_mut();
    let mut spot_light = spot_light_q.single_mut();

    transform.translate_around(Vec3::ZERO, Quat::from_rotation_z(rot));

    let back = transform.back();
    let y_sin = back.y.sin();

    // println!("y_sin {y_sin}");
    if y_sin <= 0.0 {
        ambient_light.brightness = 0.02;
        light.illuminance = 800. + (1. - y_sin.abs() * 600.);
        spot_light.intensity = 200. + y_sin.abs() * 4800.;
    } else {
        ambient_light.brightness = 0.02 + y_sin * 0.25;
        light.illuminance = 800. + y_sin * 24000.;
        spot_light.intensity = 200. + (1. - y_sin) * 200.;
    }

    light.illuminance = light.illuminance.clamp(600., 10000.);
    spot_light.intensity = spot_light.intensity.clamp(1., 800.);
    ambient_light.brightness = ambient_light.brightness.clamp(0.02, 0.2);

    transform.look_at(Vec3::ZERO, Vec3::Y);
    // println!("{rot_x} {}", transform.rotation.xyz());
}
