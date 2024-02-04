#![allow(clippy::too_many_arguments)]
use super::{
    assets::RenderResources,
    plugin::GameSessionCleanup,
    prop::{PropHandle, PropInfo},
};

use rpg_core::{
    skill::{
        skill_tables::SkillTableEntry, AreaInstance, DirectInstance, OrbitData, Origin,
        ProjectileInstance, ProjectileShape, SkillId, SkillInfo, SkillInstance,
    },
    uid::Uid,
};
use rpg_util::skill::*;

use util::{cleanup::CleanupStrategy, math::AabbComponent};

use bevy::{
    asset::{Assets, Handle},
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res},
    },
    gizmos::aabb::ShowAabbGizmo,
    hierarchy::DespawnRecursiveExt,
    log::debug,
    math::{bounding::Aabb3d, Vec3},
    pbr::{MaterialMeshBundle, PbrBundle, StandardMaterial},
    render::{
        mesh::{
            shape::{Circle, Icosphere},
            Mesh,
        },
        prelude::SpatialBundle,
    },
    scene::SceneBundle,
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
    utils::default,
};

use std::borrow::Cow;

// FIXME
// need to redo react to network messages for audio and animations

/*
        match combat_result {
            CombatResult::Attack(attack) => match attack {
                AttackResult::Blocked => {
                    if defender.kind == UnitKind::Hero {
                        game_state.session_stats.blocks += 1;
                    } else {
                        game_state.session_stats.times_blocked += 1;
                    }

                    d_audio.push("hit_blocked".into());

                    match &instance.instance {
                        SkillInstance::Direct(_) | SkillInstance::Projectile(_) => {
                            commands.entity(s_entity).despawn_recursive();
                            continue;
                        }
                        SkillInstance::Area(_) => {}
                    }
                }
                AttackResult::Dodged => {
                    if defender.kind == UnitKind::Hero {
                        game_state.session_stats.dodges += 1;
                    } else {
                        game_state.session_stats.times_dodged += 1;
                    }

                    d_audio.push("hit_blocked".into());
                }
                _ => {
                    d_audio.push("hit_soft".into());

                    *d_anim_state = AnimationState {
                        repeat: RepeatAnimation::Never,
                        paused: false,
                        index: 0,
                    };

                    if defender.kind == UnitKind::Villain {
                        game_state.session_stats.hits += 1;
                    } else {
                        game_state.session_stats.villain_hits += 1;
                    }
                }
            },
            CombatResult::Death(_) => {
                debug!("death");

                d_audio.push("hit_death".into());

                d_actions.reset();
                *d_anim_state = AnimationState {
                    repeat: RepeatAnimation::Never,
                    paused: false,
                    index: 1,
                };

                if defender.kind == UnitKind::Villain {
                    game_state.session_stats.kills += 1;
                    game_state.session_stats.hits += 1;

                    /*
                    if let Some(death) = defender.handle_death(
                        &mut attacker,
                        &metadata.rpg,
                        &mut random.0,
                        &mut game_state.next_uid,
                    ) {
                        game_state.session_stats.items_spawned += death.items.len() as u32;

                        ground_drops.0.push(GroundItemDrop {
                            source: event.defender_entity,
                            items: death.items,
                        });
                    }*/
                } else {
                    game_state.session_stats.villain_hits += 1;
                    game_state.state = PlayState::Death(GameOverState::Pending);
                }

                commands.entity(event.defender_entity).insert(Corpse);
                /*commands
                .entity(event.defender_entity)
                .insert(CorpseTimer(Timer::from_seconds(60., TimerMode::Once)));*/
            }
            _ => {}
        }
}*/

// TODO probably move this also?
pub fn clean_skills(
    mut commands: Commands,
    time: Res<Time>,
    mut skill_q: Query<(Entity, &Transform, &SkillUse)>,
) {
    for (entity, transform, skill_use) in &mut skill_q {
        let despawn = match &skill_use.instance {
            SkillInstance::Projectile(info) => match info.info.duration {
                Some(d) => time.elapsed_seconds() - info.start_time >= d,
                None => {
                    if info.info.aerial.is_some() {
                        transform.translation.y < info.info.size as f32 / 100.
                    } else {
                        false
                    }
                }
            },
            SkillInstance::Direct(info) => {
                // debug!("direct skill: {info:?}");
                info.frame >= info.info.frames
            }
            SkillInstance::Area(info) => {
                // debug!("area skill: {info:?}");
                time.elapsed_seconds() - info.start_time >= info.info.duration
            }
        };

        if despawn {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub(crate) fn prepare_skill(
    owner: Uid,
    origin: &Vec3,
    target: &Vec3,
    time: &Time,
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

    let (aabb, skill_use, transform, mesh, material, tickable) = match &skill_info.info {
        SkillInfo::Direct(_) => {
            let aabb = renderables.aabbs["direct_attack"];
            let SkillInfo::Direct(info) = &skill_info.info else {
                panic!("Expected direct attack")
            };

            let instance = SkillInstance::Direct(DirectInstance {
                info: info.clone(),
                frame: 0,
            });

            let skill_transform = Transform::from_translation(*origin).looking_at(*target, Vec3::Y);

            (aabb, instance, skill_transform, None, None, None)
        }
        SkillInfo::Projectile(info) => {
            //println!("spawn {speed} {duration} {size}");

            let tickable = info.tick_rate.as_ref().map(|tr| Tickable {
                timer: Timer::from_seconds(*tr, TimerMode::Repeating),
                can_damage: true,
            });

            let (mesh_handle, aabb) = if info.shape == ProjectileShape::Box {
                let handle = renderables.props["bolt_01"].handle.clone();
                let aabb = renderables.aabbs["bolt_01"];

                (handle, aabb)
            } else {
                let radius = info.size as f32 / 100. / 2.;
                let key = format!("orb_radius_{}", info.size);

                let (handle, aabb) = if renderables.props.contains_key(key.as_str()) {
                    let prop_handle = renderables.props[key.as_str()].handle.clone();
                    let aabb = renderables.aabbs[key.as_str()];

                    (prop_handle, aabb)
                } else {
                    let mut mesh = Mesh::try_from(Icosphere {
                        radius,
                        ..default()
                    })
                    .unwrap();

                    mesh.generate_tangents().unwrap();
                    let aabb = mesh.compute_aabb().unwrap();
                    let handle = meshes.add(mesh);

                    let aabb = Aabb3d {
                        min: aabb.min().into(),
                        max: aabb.max().into(),
                    };

                    renderables
                        .aabbs
                        .insert(Cow::Owned(key.as_str().into()), aabb);

                    let prop_handle = PropHandle::Mesh(handle);
                    renderables
                        .props
                        .insert(key.into(), PropInfo::new(prop_handle.clone()));

                    (prop_handle, aabb)
                };

                (handle, aabb)
            };

            let time = time.elapsed_seconds();

            let spawn_transform = if info.orbit.is_some() {
                Transform::from_translation(*origin + Vec3::new(0., 1.2, 0.))
            } else if info.aerial.is_some() {
                //println!("prepare aerial {attack_data:?}");
                Transform::from_translation(*origin).looking_at(*target, Vec3::Y)
            } else {
                Transform::from_translation(*origin).looking_at(*target, Vec3::Y)
            };

            let instance_info = SkillInstance::Projectile(ProjectileInstance {
                info: info.clone(),
                start_time: time,
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
                Some(mesh_handle),
                Some(renderables.materials["lava"].clone_weak()),
                tickable,
            )
        }
        SkillInfo::Area(info) => {
            let skill_instance = SkillInstance::Area(AreaInstance {
                info: info.clone(),
                start_time: time.elapsed_seconds(),
            });

            let tickable = info.tick_rate.as_ref().map(|tr| Tickable {
                timer: Timer::from_seconds(*tr, TimerMode::Repeating),
                can_damage: true,
            });

            let transform = Transform::from_translation(*origin + Vec3::new(0.0, 0.01, 0.0))
                .looking_to(Vec3::NEG_Y, Vec3::Y);

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
                // The required resource are not cached, add them into the cache
                let mut mesh: Mesh = Circle::new(radius).into();
                let _ = mesh.generate_tangents();
                //let aabb = mesh.compute_aabb().unwrap();

                // 2d shapes are on the XY plane
                let aabb = Aabb3d {
                    min: Vec3::new(-radius, -radius, 0.0),
                    max: Vec3::new(radius, radius, 0.5),
                };

                let handle = meshes.add(mesh);
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
                tickable,
            )
        }
    };

    let instance = SkillUse::new(
        owner,
        skill_id,
        skill_info.base_damage.clone(),
        skill_use,
        vec![], // FIXME effects,
        tickable,
    );

    (aabb, transform, instance, mesh, material)
}

pub(crate) fn spawn_instance(
    commands: &mut Commands,
    aabb: Aabb3d,
    transform: Transform,
    skill_use_instance: SkillUse,
    prop: Option<PropHandle>,
    material: Option<Handle<StandardMaterial>>,
) {
    // FIXME skill timer needs to be passed in here
    let skill_use = SkillUseBundle::new(skill_use_instance);

    let common_bundle = (
        GameSessionCleanup,
        CleanupStrategy::DespawnRecursive,
        AabbComponent(aabb),
        skill_use,
    );

    match &common_bundle.3.skill.instance {
        SkillInstance::Direct(_) => {
            // debug!("spawning direct skill {}", transform.translation);

            commands.spawn((
                common_bundle,
                ShowAabbGizmo::default(),
                SpatialBundle::from_transform(transform),
            ));
        }
        SkillInstance::Projectile(_) => {
            // debug!("spawning projectile skill");

            let handle = prop.unwrap();
            if let PropHandle::Scene(handle) = handle {
                commands.spawn((
                    common_bundle,
                    SceneBundle {
                        scene: handle,
                        transform,
                        ..default()
                    },
                    ShowAabbGizmo::default(),
                ));
            } else if let PropHandle::Mesh(handle) = handle {
                commands.spawn((
                    common_bundle,
                    PbrBundle {
                        transform,
                        mesh: handle,
                        material: material.unwrap(),
                        ..default()
                    },
                    ShowAabbGizmo::default(),
                ));
            }
        }
        SkillInstance::Area(_) => {
            //println!("spawning area skill");

            let handle = prop.unwrap();
            if let PropHandle::Scene(handle) = handle {
                commands.spawn((
                    common_bundle,
                    SceneBundle {
                        scene: handle,
                        transform,
                        ..default()
                    },
                    ShowAabbGizmo::default(),
                ));
            } else if let PropHandle::Mesh(handle) = handle {
                commands.spawn((
                    common_bundle,
                    MaterialMeshBundle {
                        transform,
                        mesh: handle,
                        material: material.unwrap(),
                        ..default()
                    },
                    ShowAabbGizmo::default(),
                ));
            }
        }
    }
}

// TODO determine what the client will do here..
// TODO just delete if this is already in the server, make a note there
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
