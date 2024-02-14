use crate::game::{assets::RenderResources, plugin::GameSessionCleanup};

use bevy::{
    asset::Handle,
    ecs::{bundle::Bundle, component::Component, entity::Entity, system::Commands},
    math::{Quat, Vec3},
    render::mesh::Mesh,
    scene::{Scene, SceneBundle},
    transform::components::Transform,
    utils::default,
};

use util::cleanup::CleanupStrategy;

#[derive(Component)]
pub(crate) struct StaticProp;

#[derive(Component)]
pub(crate) struct DynamicProp;

#[derive(Bundle)]
pub(crate) struct StaticPropBundle {
    pub prop: StaticProp,
    pub scene: SceneBundle,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) enum PropHandle {
    Scene(Handle<Scene>),
    Mesh(Handle<Mesh>),
}

#[derive(Hash, PartialEq)]
pub(crate) struct PropInfo {
    pub handle: PropHandle,
}

impl PropInfo {
    pub fn new(handle: PropHandle) -> Self {
        Self { handle }
    }
}

pub(crate) fn spawn(
    commands: &mut Commands,
    renderables: &RenderResources,
    key: &str,
    position: Vec3,
    rotation: Option<Quat>,
) -> Entity {
    let PropHandle::Scene(handle) = &renderables.props[key].handle else {
        panic!("bad handle");
    };

    // debug!("prop pos: {position}");

    let mut transform = Transform::from_translation(position + Vec3::Y * 0.25);
    if let Some(rotation) = rotation {
        transform.rotation = rotation;
    }

    commands
        .spawn((
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            StaticPropBundle {
                prop: StaticProp,
                scene: SceneBundle {
                    scene: handle.clone_weak(),
                    transform,
                    ..default()
                },
            },
        ))
        .id()
}
