use bevy::{
    asset::Handle,
    ecs::{bundle::Bundle, component::Component, entity::Entity, system::Commands},
    hierarchy::BuildChildren,
    math::Vec3,
    pbr::{MaterialMeshBundle, NotShadowCaster, NotShadowReceiver, StandardMaterial},
    render::mesh::Mesh,
    transform::components::Transform,
    utils::default,
};

use super::{assets::RenderResources, plugin::GameSessionCleanup};

use util::cleanup::CleanupStrategy;

#[derive(Debug, PartialEq, Clone, Component)]
pub(crate) struct HealthBar {
    pub min: u32,
    pub max: u32,
    pub curr: u32,
    pub width: f32,
    pub bar_entity: Entity,
}

impl Default for HealthBar {
    fn default() -> Self {
        Self {
            width: 0.8,
            min: 0,
            max: 0,
            curr: 0,
            bar_entity: Entity::PLACEHOLDER,
        }
    }
}

impl HealthBar {
    pub fn new(bar_entity: Entity, width: f32) -> Self {
        Self {
            width,
            min: 0,
            max: 0,
            curr: 0,
            bar_entity,
        }
    }

    pub fn spawn_bars(
        commands: &mut Commands,
        renderables: &RenderResources,
        transform: Transform,
    ) -> Entity {
        commands
            .spawn((
                GameSessionCleanup,
                CleanupStrategy::DespawnRecursive,
                HealthBarParentBundle::new(
                    renderables.meshes["bar_outer"].clone_weak(),
                    renderables.materials["bar_frame"].clone_weak(),
                    transform,
                ),
            ))
            .with_children(|p| {
                p.spawn(HealthBarChildBundle::new(
                    renderables.meshes["bar_inner"].clone_weak(),
                    renderables.materials["bar_fill_hp"].clone_weak(),
                    Transform::from_translation(Vec3::Z * 0.001),
                ));
            })
            .id()
    }
}

#[derive(Bundle)]
pub(crate) struct HealthBarParentBundle {
    pub not_shadow_caster: NotShadowCaster,
    pub not_shadow_receiver: NotShadowReceiver,
    pub bar: HealthBarFrame,
    pub mesh: MaterialMeshBundle<StandardMaterial>,
}

impl HealthBarParentBundle {
    pub(crate) fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
    ) -> Self {
        Self {
            mesh: MaterialMeshBundle {
                mesh,
                material,
                transform,
                ..default()
            },
            not_shadow_caster: NotShadowCaster,
            not_shadow_receiver: NotShadowReceiver,
            bar: HealthBarFrame,
        }
    }
}

#[derive(Bundle)]
pub(crate) struct HealthBarChildBundle {
    pub not_shadow_caster: NotShadowCaster,
    pub not_shadow_receiver: NotShadowReceiver,
    pub rect: HealthBarRect,
    pub mesh: MaterialMeshBundle<StandardMaterial>,
}

impl HealthBarChildBundle {
    pub(crate) fn new(
        mesh: Handle<Mesh>,
        material: Handle<StandardMaterial>,
        transform: Transform,
    ) -> Self {
        Self {
            mesh: MaterialMeshBundle {
                mesh,
                material,
                transform,
                ..default()
            },
            not_shadow_caster: NotShadowCaster,
            not_shadow_receiver: NotShadowReceiver,
            rect: HealthBarRect,
        }
    }
}

#[derive(Component)]
pub(crate) struct HealthBarFrame;

#[derive(Component)]
pub(crate) struct HealthBarRect;
