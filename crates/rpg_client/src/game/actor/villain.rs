#![allow(clippy::too_many_arguments)]

use crate::game::{
    actions::{Action, ActionData, Actions, AttackData},
    assets::RenderResources,
    metadata::MetadataResources,
    plugin::GameState,
};

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
pub fn spawner(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<VillainSpawner>,
    mut state: ResMut<GameState>,
    mut random: ResMut<SharedRng>,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
    hero_q: Query<(&Unit, &Transform), With<Hero>>,
) {
    // Eventually resurrection or continuation of villain actions may be desired, keep this check
    // here for now
    let (hero_unit, hero_transform) = hero_q.single();
    if !hero_unit.is_alive() {
        return;
    }

    spawner.timer.tick(time.delta());
    if spawner.timer.finished() || state.session_stats.spawned == 0 {
        let frequency = spawner.frequency * 0.995;
        if state.session_stats.spawned != 0 {
            spawner.update_frequency(frequency);
        }

        spawn_villain(
            &mut commands,
            &mut state.next_uid,
            &hero_transform.translation,
            &metadata,
            &renderables,
            &mut random,
        );

        state.session_stats.spawned += 1;
    }
}

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
