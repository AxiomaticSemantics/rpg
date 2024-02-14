use super::world::RpgWorld;
use crate::game::plugin::GameSessionCleanup;

use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::{debug, info},
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
pub(crate) struct PlayerSpotLight;

#[derive(Component)]
pub(crate) struct EnvironmentDirectionalLight;

#[derive(Resource)]
pub struct Environment;

pub(crate) fn prepare_environment(mut commands: Commands, mut rpg_world: ResMut<RpgWorld>) {
    if rpg_world.env_loaded {
        return;
    }

    let Some(active_zone) = rpg_world.active_zone else {
        return;
    };

    debug!("spawning environment");

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
        PlayerSpotLight,
        SpotLightBundle {
            spot_light: SpotLight {
                range: 20.,
                shadows_enabled: true,
                ..default()
            },
            ..default()
        },
    ));

    // PointLight { intensity: 1000., range: 32., shadows_enabled: true, ..default() }

    let zone_kind = rpg_world.zones[&active_zone].kind;

    if zone_kind == ZoneKind::Overworld || zone_kind == ZoneKind::OverworldTown {
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

    rpg_world.env_loaded = true;
}

pub(crate) fn cleanup(mut commands: Commands) {
    commands.remove_resource::<Environment>();
}

pub(crate) fn day_night_cycle(
    time: Res<Time>,
    mut light_q: Query<(&mut DirectionalLight, &mut Transform), With<EnvironmentDirectionalLight>>,
    mut spot_light_q: Query<&mut SpotLight, With<PlayerSpotLight>>,
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
    if y_sin < 0.0 {
        // night-time
        ambient_light.brightness = 0.02;
        light.illuminance = 0.;
        spot_light.intensity = 2000. + y_sin.abs() * 48000.;
    } else {
        // day-time
        ambient_light.brightness = 0.02 + y_sin * 0.25;
        light.illuminance = 80. + y_sin * 1000.;
        spot_light.intensity = 0. + (1. - y_sin) * 2000.;
    }

    light.illuminance = light.illuminance.clamp(0., 1000.);
    spot_light.intensity = spot_light.intensity.clamp(100., 80000.);
    ambient_light.brightness = ambient_light.brightness.clamp(0.00, 0.00);

    transform.look_at(Vec3::ZERO, Vec3::Y);
    // println!("{rot_x} {}", transform.rotation.xyz());
}
