#![allow(clippy::too_many_arguments)]

use crate::game::{assets::RenderResources, metadata::MetadataResources, plugin::GameState};

use rpg_util::unit::{Corpse, Hero, Unit, Villain};

use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    math::Vec3,
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
    transform::components::Transform,
    utils::Duration,
};

/*
fn spawn_villain(
    commands: &mut Commands,
    next_uid: &mut NextUid,
    origin: &Vec3,
    metadata: &MetadataResources,
    renderables: &RenderResources,
    rng: &mut SharedRng,
) {
    let mut unit = rpg_core::unit::generation::generate(&mut rng.0, &metadata.rpg, next_uid, 1);

    unit.add_default_skills(&metadata.rpg);

    let dir_roll = std::f32::consts::TAU * (0.5 - rng.f32());
    let distance = 14_f32;

    let mut transform = Transform::from_translation(*origin);
    transform.rotate_y(dir_roll);
    transform.translation += transform.forward() * distance;

    actor::spawn_actor(
        commands,
        metadata,
        renderables,
        unit,
        None,
        None,
        Some(transform),
    );
}
*/
