use super::{
    animation::AnimationState,
    player::{Player, PlayerBundle},
    unit::{Hero, ThinkTimer, Unit, UnitBundle, Villain, VillainBundle},
};
use crate::game::{
    actions::Actions, assets::RenderResources, health_bar::HealthBar, item::UnitStorage,
    metadata::MetadataResources, passive_tree::PassiveTree, plugin::GameSessionCleanup,
};

use audio_manager::plugin::AudioActions;

use bevy::{
    animation::AnimationClip,
    asset::Handle,
    ecs::{bundle::Bundle, component::Component, system::Commands},
    math::Vec3,
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::{Deref, DerefMut},
    render::{mesh::Mesh, primitives::Aabb},
    scene::{Scene, SceneBundle},
    time::{Timer, TimerMode},
    transform::components::Transform,
    utils::default,
};

use rpg_core::{
    class::Class,
    passive_tree::PassiveSkillGraph,
    storage::UnitStorage as RpgUnitStorage,
    unit::{Unit as RpgUnit, UnitKind},
    villain::VillainId,
};
use util::cleanup::CleanupStrategy;

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
        Class::Str | Class::StrDexInt => "swordsman",
        Class::Dex | Class::StrDex | Class::DexInt => "archer",
        Class::Int | Class::IntStr => "wizard",
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

pub(crate) fn spawn_actor(
    commands: &mut Commands,
    metadata: &MetadataResources,
    renderables: &RenderResources,
    unit: RpgUnit,
    storage: Option<RpgUnitStorage>,
    passive_tree: Option<PassiveSkillGraph>,
    transform: Option<Transform>,
) {
    let body_aabb = Aabb::from_min_max(Vec3::new(-0.3, 0., -0.25), Vec3::new(0.3, 1.8, 0.25));

    let bar = HealthBar::spawn_bars(commands, renderables, Transform::default());

    let actor_key = match unit.kind {
        UnitKind::Hero => get_hero_actor_key(unit.class),
        UnitKind::Villain => get_villain_actor_key(unit.info.villain().id),
    };

    let actor_basic_bundle = ActorBasicBundle {
        health_bar: HealthBar::new(bar, 0.8),
        actor_key: ActorKey(actor_key),
        aabb: body_aabb,
        ..default()
    };

    match unit.kind {
        UnitKind::Hero => {
            let actor = renderables.actors[actor_key].actor.clone();
            let actor_player_bundle = (
                GameSessionCleanup,
                CleanupStrategy::DespawnRecursive,
                PlayerBundle {
                    player: Player,
                    hero: Hero,
                },
                UnitBundle::new(Unit(unit)),
                UnitStorage(storage.unwrap()),
                PassiveTree(passive_tree.unwrap()),
                //AabbGizmo::default(),
            );

            match actor {
                ActorHandle::Mesh(handle) => {
                    commands.spawn((
                        actor_player_bundle,
                        ActorMeshBundle {
                            basic: actor_basic_bundle,
                            mesh: MaterialMeshBundle {
                                mesh: handle,
                                ..default()
                            },
                        },
                    ));
                }
                ActorHandle::Scene(handle) => {
                    commands.spawn((
                        actor_player_bundle,
                        ActorSceneBundle {
                            basic: actor_basic_bundle,
                            scene: SceneBundle {
                                scene: handle,
                                ..default()
                            },
                        },
                    ));
                }
            };
        }
        UnitKind::Villain => {
            let unit_info = unit.info.villain();
            let villain_info = &metadata.rpg.unit.villains.villains[&unit_info.id];

            let actor = renderables.actors[actor_key].actor.clone();

            let think_timer = ThinkTimer(Timer::from_seconds(
                villain_info.think_cooldown,
                TimerMode::Repeating,
            ));

            let actor_villain_bundle = (
                GameSessionCleanup,
                CleanupStrategy::DespawnRecursive,
                VillainBundle {
                    villain: Villain {
                        look_target: None,
                        moving: false,
                    },
                    think_timer,
                },
                UnitBundle::new(Unit(unit)),
            );

            match actor {
                ActorHandle::Mesh(handle) => {
                    commands.spawn((
                        actor_villain_bundle,
                        ActorMeshBundle {
                            basic: actor_basic_bundle,
                            mesh: MaterialMeshBundle {
                                mesh: handle,
                                ..default()
                            },
                        },
                    ));
                }
                ActorHandle::Scene(handle) => {
                    commands.spawn((
                        actor_villain_bundle,
                        ActorSceneBundle {
                            basic: actor_basic_bundle,
                            scene: SceneBundle {
                                scene: handle,
                                transform: transform.unwrap(),
                                ..default()
                            },
                        },
                    ));
                }
            };
        }
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
