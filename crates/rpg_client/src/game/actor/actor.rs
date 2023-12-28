use super::animation::AnimationState;
use crate::game::{actions::Actions, health_bar::HealthBar};
use audio_manager::plugin::AudioActions;

use bevy::{
    animation::AnimationClip,
    asset::Handle,
    ecs::{bundle::Bundle, component::Component},
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::{Deref, DerefMut},
    render::{mesh::Mesh, primitives::Aabb},
    scene::{Scene, SceneBundle},
};

use rpg_core::{class::Class, villain::VillainId};

#[derive(Default, Debug, Component, Deref, DerefMut)]
pub struct ActorKey(pub &'static str);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActorHandle {
    Mesh(Handle<Mesh>),
    Scene(Handle<Scene>),
}

#[derive(Default, Bundle)]
pub(crate) struct ActorBasicBundle {
    pub actions: Actions,
    pub aabb: Aabb,
    pub animation_state: AnimationState,
    pub actor_key: ActorKey,
    pub audio_action: AudioActions,
    pub health_bar: HealthBar,
}

#[derive(Default, Bundle)]
pub(crate) struct ActorSceneBundle {
    pub basic: ActorBasicBundle,
    pub scene: SceneBundle,
}

#[derive(Default, Bundle)]
pub(crate) struct ActorMeshBundle {
    pub basic: ActorBasicBundle,
    pub mesh: MaterialMeshBundle<StandardMaterial>,
}

#[derive(Hash, PartialEq)]
pub(crate) struct ActorInfo {
    pub animations: Vec<Handle<AnimationClip>>,
    pub actor: ActorHandle,
}

impl ActorInfo {
    pub fn new(animations: Vec<Handle<AnimationClip>>, actor: ActorHandle) -> Self {
        Self { animations, actor }
    }
}

pub fn get_hero_actor_key(class: Class) -> &'static str {
    match class {
        Class::Str => "swordsman",
        Class::Dex => "archer",
        Class::Int => "wizard",
        Class::StrDex => "archer",
        Class::DexInt => "archer",
        Class::IntStr => "wizard",
        Class::StrDexInt => "swordsman",
    }
}

pub fn get_villain_actor_key(id: VillainId) -> &'static str {
    match id {
        VillainId::Wizard => "wizard",
        VillainId::Swordsman => "swordsman",
        VillainId::Archer => "archer",
        VillainId::Cleric => "wizard",
    }
}

// TODO now that I've verified that this works expand upon it further to add a little more variety to our limited skinned meshes
/*
pub(crate) fn _replace_actor_materials(
    actor: Query<Entity, With<Hero>>,
    actor_resources: Res<ActorResources>,
    mut handles: Query<(&Name, &mut Handle<StandardMaterial>)>,
    child_q: Query<&Children>,
) {
    for entity in &actor {
        for child in child_q.iter_descendants(entity) {
            let Ok((name, mut material_handle)) = handles.get_mut(child) else {
                continue;
            };
            if name.as_str() == "Cylinder.003" {
                *material_handle = actor_resources.materials["glow_green"].clone();
            }
        }
    }
}*/

/* fn prepare_actor(
    actor_resources: Res<ActorResources>,
    mut unit_q: Query<(&ActorKey, &Children), With<Unit>>,
    mesh_q: Query<&Handle<Mesh>>,
) {
    for (key, children) in &unit_q {
        for child in children.iter() {
            for mesh in &mesh_q.get(*child) {
                println!("K");
            }
        }
    }
}*/
