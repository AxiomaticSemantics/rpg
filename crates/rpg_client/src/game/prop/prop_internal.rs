use crate::game::{
    assets::RenderResources, metadata::MetadataResources, plugin::GameSessionCleanup,
};

use bevy::{
    asset::Handle,
    ecs::{bundle::Bundle, component::Component, entity::Entity, system::Commands},
    log::debug,
    math::{Quat, Vec3},
    render::mesh::Mesh,
    scene::{Scene, SceneBundle},
    transform::components::Transform,
    utils::default,
};

use util::cleanup::CleanupStrategy;

use std::borrow::Cow;

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

#[derive(Debug, Hash, PartialEq)]
pub(crate) struct PropInfo {
    pub handle: PropHandle,
    pub key: Cow<'static, str>,
}

impl PropInfo {
    pub fn new(key: &str, handle: PropHandle) -> Self {
        Self {
            key: Cow::Owned(key.into()),
            handle,
        }
    }
}

pub(crate) fn spawn(
    commands: &mut Commands,
    metadata: &MetadataResources,
    renderables: &RenderResources,
    key: &str,
    position: Vec3,
    rotation: Option<Quat>,
) -> Entity {
    let prop_info = &renderables.props[key];
    let PropHandle::Scene(handle) = &prop_info.handle else {
        panic!("bad handle");
    };

    debug!("prop pos: {prop_info:?}");

    let prop_meta = &metadata.prop.prop.prop[key];
    let position = if let Some(offset) = prop_meta.offset {
        position + offset
    } else {
        position
    };

    let transform = if let Some(rotation) = rotation {
        Transform::from_translation(position).with_rotation(rotation)
    } else {
        Transform::from_translation(position)
    };

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
