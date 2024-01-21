use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    hierarchy::DespawnRecursiveExt,
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
};

use rpg_util::unit::Unit;

#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct CorpseTimer(pub Timer);

pub(crate) fn upkeep(time: Res<Time>, mut unit_q: Query<&mut Unit, Without<CorpseTimer>>) {
    for mut unit in &mut unit_q {
        unit.stats.apply_regeneration(time.delta_seconds());
    }
}

pub(crate) fn remove_corpses(
    mut commands: Commands,
    time: Res<Time>,
    mut unit_q: Query<(Entity, &mut CorpseTimer), With<Unit>>,
) {
    for (entity, mut timer) in &mut unit_q {
        timer.tick(time.delta());
        if timer.just_finished() {
            // TODO tell the client to despawn the entity
            commands.entity(entity).despawn_recursive();
        }
    }
}
