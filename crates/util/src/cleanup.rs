use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query},
    },
    hierarchy::DespawnRecursiveExt,
    log::debug,
};

/// A marker `Component` to signify how an entity should be despawned
#[derive(Clone, Copy, Component)]
pub enum CleanupStrategy {
    Despawn,
    DespawnRecursive,
}

/// A generic cleanup system used to reduce boilerplating
pub fn cleanup<C: Component>(
    mut commands: Commands,
    query: Query<(Entity, &CleanupStrategy), (With<C>, With<CleanupStrategy>)>,
) {
    debug!("running cleanup for {}", std::any::type_name::<C>());

    for (e, strategy) in &query {
        match strategy {
            CleanupStrategy::Despawn => commands.entity(e).despawn(),
            CleanupStrategy::DespawnRecursive => commands.entity(e).despawn_recursive(),
        }
    }
}
