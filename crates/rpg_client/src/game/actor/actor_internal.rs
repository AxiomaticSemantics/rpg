use super::{
    animation::AnimationState,
    player::{Player, PlayerBundle},
};
use crate::game::{
    assets::RenderResources, health_bar::HealthBar, passive_tree::PassiveTree,
    plugin::GameSessionCleanup,
};

use bevy::{
    animation::AnimationClip,
    asset::Handle,
    ecs::{bundle::Bundle, component::Component, entity::Entity, system::Commands},
    log::info,
    math::{bounding::Aabb3d, Vec3},
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::{Deref, DerefMut},
    render::mesh::Mesh,
    scene::{Scene, SceneBundle},
    transform::components::Transform,
    utils::default,
};

use audio_manager::plugin::AudioActions;
use rpg_core::{
    class::Class,
    passive_tree::PassiveSkillGraph,
    storage::UnitStorage as RpgUnitStorage,
    unit::{Unit as RpgUnit, UnitKind},
    villain::VillainId,
};
use rpg_util::{
    item::UnitStorage,
    skill::{SkillSlots, Skills},
    unit::{Hero, Unit, UnitBundle, Villain, VillainBundle},
};
use util::{cleanup::CleanupStrategy, math::AabbComponent};

#[derive(Default, Debug, Component, Deref, DerefMut)]
pub struct ActorKey(pub &'static str);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ActorHandle {
    Mesh(Handle<Mesh>),
    Scene(Handle<Scene>),
}

#[derive(Bundle)]
pub(crate) struct ActorBasicBundle {
    pub aabb: AabbComponent,
    pub animation_state: AnimationState,
    pub actor_key: ActorKey,
    pub audio_action: AudioActions,
    pub health_bar: HealthBar,
}

#[derive(Bundle)]
pub(crate) struct ActorSceneBundle {
    pub basic: ActorBasicBundle,
    pub scene: SceneBundle,
}

#[derive(Bundle)]
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
    entity: Entity,
    commands: &mut Commands,
    renderables: &RenderResources,
    transform: Transform,
    unit: RpgUnit,
    skills: Skills,
    skill_slots: SkillSlots,
    storage: Option<RpgUnitStorage>,
    passive_tree: Option<PassiveSkillGraph>,
) {
    let aabb = AabbComponent(Aabb3d {
        min: Vec3::new(-0.3, 0., -0.25),
        max: Vec3::new(0.3, 1.8, 0.25),
    });

    let bar = HealthBar::spawn_bars(commands, renderables, Transform::default());

    let actor_key = match unit.kind {
        UnitKind::Hero => get_hero_actor_key(unit.class),
        UnitKind::Villain => get_villain_actor_key(unit.info.villain().id),
    };

    let actor_basic_bundle = ActorBasicBundle {
        health_bar: HealthBar::new(bar, 0.8),
        actor_key: ActorKey(actor_key),
        aabb,
        audio_action: AudioActions::default(),
        animation_state: AnimationState::default(),
    };

    match unit.kind {
        UnitKind::Hero => {
            let actor = renderables.actors[actor_key].actor.clone();
            let actor_hero_bundle = (
                GameSessionCleanup,
                CleanupStrategy::DespawnRecursive,
                UnitBundle::new(Unit(unit), skills, skill_slots),
                UnitStorage(storage.unwrap()),
                PassiveTree(passive_tree.unwrap()),
                //AabbGizmo::default(),
            );

            if entity == Entity::PLACEHOLDER {
                match actor {
                    ActorHandle::Mesh(handle) => {
                        commands.spawn((
                            actor_hero_bundle,
                            Hero,
                            ActorMeshBundle {
                                basic: actor_basic_bundle,
                                mesh: MaterialMeshBundle {
                                    mesh: handle,
                                    transform,
                                    ..default()
                                },
                            },
                        ));
                    }
                    ActorHandle::Scene(handle) => {
                        commands.spawn((
                            actor_hero_bundle,
                            Hero,
                            ActorSceneBundle {
                                basic: actor_basic_bundle,
                                scene: SceneBundle {
                                    scene: handle,
                                    transform,
                                    ..default()
                                },
                            },
                        ));
                    }
                }
            } else {
                match actor {
                    ActorHandle::Mesh(handle) => {
                        commands.entity(entity).insert((
                            actor_hero_bundle,
                            PlayerBundle {
                                player: Player,
                                hero: Hero,
                            },
                            ActorMeshBundle {
                                basic: actor_basic_bundle,
                                mesh: MaterialMeshBundle {
                                    mesh: handle,
                                    transform,
                                    ..default()
                                },
                            },
                        ));
                    }
                    ActorHandle::Scene(handle) => {
                        commands.entity(entity).insert((
                            actor_hero_bundle,
                            PlayerBundle {
                                player: Player,
                                hero: Hero,
                            },
                            ActorSceneBundle {
                                basic: actor_basic_bundle,
                                scene: SceneBundle {
                                    scene: handle,
                                    transform,
                                    ..default()
                                },
                            },
                        ));
                    }
                }
            };
        }
        UnitKind::Villain => {
            let actor = renderables.actors[actor_key].actor.clone();

            let actor_villain_bundle = (
                GameSessionCleanup,
                CleanupStrategy::DespawnRecursive,
                VillainBundle {
                    villain: Villain,
                    unit: UnitBundle::new(Unit(unit), skills, skill_slots),
                },
            );

            match actor {
                ActorHandle::Mesh(handle) => {
                    commands.spawn((
                        actor_villain_bundle,
                        ActorMeshBundle {
                            basic: actor_basic_bundle,
                            mesh: MaterialMeshBundle {
                                transform,
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
                                transform,
                                ..default()
                            },
                        },
                    ));
                }
            };
        }
    }
}

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
