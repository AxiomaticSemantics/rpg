use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query},
    },
    hierarchy::DespawnRecursiveExt,
};

/// A marker `Component` to signify how an entity should be despawned.
#[derive(Clone, Copy, Component)]
pub enum CleanupStrategy {
    Despawn,
    DespawnRecursive,
}

pub fn cleanup<C: Component>(
    mut commands: Commands,
    query: Query<(Entity, &CleanupStrategy), (With<C>, With<CleanupStrategy>)>,
) {
    println!("cleanup for {}", std::any::type_name::<C>());

    for (e, strategy) in &query {
        match strategy {
            CleanupStrategy::Despawn => commands.entity(e).despawn(),
            CleanupStrategy::DespawnRecursive => commands.entity(e).despawn_recursive(),
        }
    }
}
