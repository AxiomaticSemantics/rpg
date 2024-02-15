#![allow(clippy::too_many_arguments)]
use super::{
    assets::RenderResources,
    plugin::GameSessionCleanup,
    prop::{PropHandle, PropInfo},
};

use rpg_core::{
    skill::{
        skill_tables::SkillTableEntry, AreaInstance, DirectInstance, OrbitData, ProjectileInstance,
        ProjectileShape, SkillId, SkillInfo, SkillInstance, SkillTarget, TimerDescriptor,
    },
    uid::{InstanceUid, Uid},
};
use rpg_util::skill::*;

use util::{cleanup::CleanupStrategy, math::AabbComponent};

use bevy::{
    asset::{Assets, Handle},
    ecs::system::Commands,
    gizmos::aabb::ShowAabbGizmo,
    log::debug,
    math::{
        bounding::Aabb3d,
        primitives::{Circle, Sphere},
        Vec3,
    },
    pbr::{MaterialMeshBundle, PbrBundle, StandardMaterial},
    render::{mesh::Mesh, prelude::SpatialBundle},
    scene::SceneBundle,
    time::{Timer, TimerMode},
    transform::components::Transform,
    utils::default,
};

use std::borrow::Cow;

pub(crate) fn prepare_skill(
    instance_uid: InstanceUid,
    owner: Uid,
    target: &SkillTarget,
    renderables: &mut RenderResources,
    meshes: &mut Assets<Mesh>,
    skill_info: &SkillTableEntry,
    skill_id: SkillId,
) -> (
    Aabb3d,
    Transform,
    SkillUse,
    Option<PropHandle>,
    Option<Handle<StandardMaterial>>,
    Option<SkillTimer>,
) {
    // debug!("{:?}", &skill_info.origin);

    /* FIXME
    let effects = skill
        .effects
        .iter()
        .map(|e| {
            let data = match e {
                EffectInfo::Pierce(_) => EffectData::Pierce(PierceData::default()),
                EffectInfo::Chain(_) => EffectData::Chain(ChainData::default()),
                EffectInfo::Split(_) => EffectData::Split(SplitData::default()),
                EffectInfo::Dot(_) => EffectData::Dot(DotData::default()),
                EffectInfo::Knockback(_) => EffectData::Knockback(KnockbackData::default()),
            };

            EffectInstance::new(e.clone(), data)
        })
        .collect();
    */

    let timer = if let Some(timer) = &skill_info.timer {
        match timer {
            TimerDescriptor::Duration(duration) => Some(SkillTimer::Duration(Timer::from_seconds(
                *duration,
                TimerMode::Once,
            ))),
            TimerDescriptor::Tickable(tickable) => Some(SkillTimer::Tickable(Tickable {
                timer: Timer::from_seconds(tickable.duration, TimerMode::Once),
                ticker: Timer::from_seconds(tickable.frequency, TimerMode::Repeating),
                can_damage: true,
            })),
        }
    } else {
        None
    };

    let (aabb, skill_use, transform, mesh, material) = match &skill_info.info {
        SkillInfo::Direct(_) => {
            let aabb = renderables.aabbs["direct_attack"];
            let SkillInfo::Direct(info) = &skill_info.info else {
                panic!("Expected direct attack")
            };

            let instance = SkillInstance::Direct(DirectInstance {
                info: info.clone(),
                frame: 0,
            });

            let skill_transform =
                Transform::from_translation(target.origin).looking_at(target.target, Vec3::Y);

            (aabb, instance, skill_transform, None, None)
        }
        SkillInfo::Projectile(info) => {
            //println!("spawn {speed} {duration} {size}");

            let (handle, aabb) = if info.shape == ProjectileShape::Box {
                let handle = renderables.props["bolt_01"].handle.clone();
                let aabb = renderables.aabbs["bolt_01"];

                (handle, aabb)
            } else {
                let radius = info.size as f32 / 100. / 2.;
                let key = format!("orb_radius_{}", info.size);

                let (handle, aabb) = if renderables.props.contains_key(key.as_str()) {
                    let handle = renderables.props[key.as_str()].handle.clone();
                    let aabb = renderables.aabbs[key.as_str()];

                    (handle, aabb)
                } else {
                    let mesh = Mesh::try_from(Sphere {
                        radius,
                        ..default()
                    })
                    .unwrap();

                    let handle = meshes.add(mesh);

                    let aabb = Aabb3d {
                        min: Vec3::splat(-radius),
                        max: Vec3::splat(radius),
                    };

                    renderables
                        .aabbs
                        .insert(Cow::Owned(key.as_str().into()), aabb);

                    let handle = PropHandle::Mesh(handle);
                    renderables
                        .props
                        .insert(key.into(), PropInfo::new(handle.clone()));

                    (handle, aabb)
                };

                (handle, aabb)
            };

            let spawn_transform = if info.orbit.is_some() {
                Transform::from_translation(target.origin)
            } else if info.aerial.is_some() {
                Transform::from_translation(target.origin).looking_at(target.target, Vec3::Y)
            } else {
                Transform::from_translation(target.origin).looking_at(target.target, Vec3::Y)
            };

            let instance_info = SkillInstance::Projectile(ProjectileInstance {
                info: info.clone(),
                orbit: if info.orbit.is_some() {
                    Some(OrbitData {
                        origin: spawn_transform.translation,
                    })
                } else {
                    None
                },
            });

            (
                aabb,
                instance_info,
                spawn_transform,
                Some(handle),
                Some(renderables.materials["lava"].clone_weak()),
            )
        }
        SkillInfo::Area(info) => {
            let skill_instance = SkillInstance::Area(AreaInstance { info: info.clone() });

            let transform =
                Transform::from_translation(target.origin).looking_to(Vec3::NEG_Y, Vec3::Y);

            let radius = info.radius as f32 / 100.;
            let key = format!("area_radius_{}", info.radius);

            // If the required resources are cached use the
            let (mesh_handle, aabb) = if renderables.meshes.contains_key(key.as_str()) {
                // The required resources are cached, return them
                (
                    renderables.meshes[key.as_str()].clone_weak(),
                    renderables.aabbs[key.as_str()],
                )
            } else {
                // The required resource is not cached, add it

                // 2d shapes are on the XY plane
                let aabb = Aabb3d {
                    min: Vec3::new(-radius, -radius, 0.0),
                    max: Vec3::new(radius, radius, 0.5),
                };

                let handle = meshes.add(Circle::new(radius));
                let weak = handle.clone_weak();
                renderables
                    .meshes
                    .insert(Cow::Owned(key.as_str().into()), handle);
                renderables
                    .aabbs
                    .insert(Cow::Owned(key.as_str().into()), aabb);

                (weak, aabb)
            };

            let material = renderables.materials["aura_red"].clone_weak();

            (
                aabb,
                skill_instance,
                transform,
                Some(PropHandle::Mesh(mesh_handle)),
                Some(material),
            )
        }
    };

    let instance = SkillUse::new(
        instance_uid,
        owner,
        skill_id,
        skill_info.base_damage.clone(),
        skill_use,
        vec![], // FIXME effects,
    );

    (aabb, transform, instance, mesh, material, timer)
}

pub(crate) fn spawn_instance(
    commands: &mut Commands,
    aabb: Aabb3d,
    transform: Transform,
    skill_use_instance: SkillUse,
    prop: Option<PropHandle>,
    material: Option<Handle<StandardMaterial>>,
    timer: Option<SkillTimer>,
) {
    // FIXME skill timer needs to be passed in here
    let skill_use = SkillUseBundle::new(skill_use_instance);

    let common_bundle = (
        GameSessionCleanup,
        CleanupStrategy::DespawnRecursive,
        AabbComponent(aabb),
        skill_use,
    );

    let entity = match &common_bundle.3.skill.instance {
        SkillInstance::Direct(_) => {
            // debug!("spawning direct skill {}", transform.translation);

            commands
                .spawn((
                    common_bundle,
                    ShowAabbGizmo::default(),
                    SpatialBundle::from_transform(transform),
                ))
                .id()
        }
        SkillInstance::Projectile(_) => {
            // debug!("spawning projectile skill");

            let handle = prop.unwrap();
            if let PropHandle::Scene(handle) = handle {
                commands
                    .spawn((
                        common_bundle,
                        SceneBundle {
                            scene: handle,
                            transform,
                            ..default()
                        },
                        ShowAabbGizmo::default(),
                    ))
                    .id()
            } else if let PropHandle::Mesh(handle) = handle {
                commands
                    .spawn((
                        common_bundle,
                        PbrBundle {
                            transform,
                            mesh: handle,
                            material: material.unwrap(),
                            ..default()
                        },
                        ShowAabbGizmo::default(),
                    ))
                    .id()
            } else {
                unreachable!()
            }
        }
        SkillInstance::Area(_) => {
            //println!("spawning area skill");

            let handle = prop.unwrap();
            if let PropHandle::Scene(handle) = handle {
                commands
                    .spawn((
                        common_bundle,
                        SceneBundle {
                            scene: handle,
                            transform,
                            ..default()
                        },
                        ShowAabbGizmo::default(),
                    ))
                    .id()
            } else if let PropHandle::Mesh(handle) = handle {
                commands
                    .spawn((
                        common_bundle,
                        MaterialMeshBundle {
                            transform,
                            mesh: handle,
                            material: material.unwrap(),
                            ..default()
                        },
                        ShowAabbGizmo::default(),
                    ))
                    .id()
            } else {
                unreachable!()
            }
        }
    };

    if let Some(timer) = timer {
        commands.entity(entity).insert(timer);
    }
}

// TODO determine what the client will do here..
// TODO to avoid traffic the client should handle any interactions that would destroy a skill

/*
/// Returns `true` if the skill should be destroyed
fn handle_effects(
    time: &Time,
    //random: &mut SharedRng,
    skill_use: &mut SkillUse,
    skill_transform: &mut Transform,
    defender_actions: &mut Actions,
) -> bool {
    //println!("info {:?}", skill_use.effects);

    if let Some(effect) = &mut skill_use.effects.iter_mut().find(|e| e.info.is_knockback()) {
        let EffectInfo::Knockback(info) = &effect.info else {
            panic!("expected knockback info");
        };

        let EffectData::Knockback(_) = &mut effect.data else {
            panic!("expected knockback data");
        };

        defender_actions.reset();

        defender_actions.set(Action::new(
            ActionData::Knockback(KnockbackActionData {
                direction: skill_transform.forward(),
                speed: info.speed,
                start: time.elapsed_seconds(),
                duration: info.duration,
            }),
            None,
            false,
        ));
    }

    let despawn =
        if let Some(effect) = &mut skill_use.effects.iter_mut().find(|e| e.info.is_pierce()) {
            let EffectInfo::Pierce(info) = &effect.info else {
                panic!("expected pierce info");
            };

            let EffectData::Pierce(data) = &mut effect.data else {
                panic!("expected pierce data");
            };

            //println!("pierce {} {}", info.pierces, data.count);
            data.count += 1;

            data.count > info.pierces
        } else {
            false
        };

    if despawn {
        return true;
    }

    let despawn =
        if let Some(effect) = &mut skill_use.effects.iter_mut().find(|e| e.info.is_chain()) {
            let EffectInfo::Chain(info) = &mut effect.info else {
                panic!("expected chain info");
            };

            let EffectData::Chain(data) = &mut effect.data else {
                panic!("expected chain data");
            };

            //println!("chain {}", info.chains);
            data.count += 1;

            skill_transform.rotate_y(0.5 - random.f32());

            data.count > info.chains
        } else {
            false
        };

    despawn
}*/
