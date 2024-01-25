#![allow(clippy::too_many_arguments)]
use super::{
    actor::animation::AnimationState,
    assets::RenderResources,
    metadata::MetadataResources,
    plugin::{GameOverState, GameSessionCleanup, GameState, PlayState},
    prop::{PropHandle, PropInfo},
};

use audio_manager::plugin::AudioActions;
use rpg_core::{
    combat::{AttackResult, CombatResult},
    damage::Damage,
    skill::{
        effect::*, skill_tables::SkillTableEntry, AreaInstance, DirectInstance, OrbitData, Origin,
        ProjectileInstance, ProjectileShape, Skill, SkillId, SkillInfo, SkillInstance,
    },
    unit::UnitKind,
};
use rpg_util::{
    actions::{Action, ActionData, Actions, AttackData, KnockbackData as KnockbackActionData},
    item::{GroundItemDrop, GroundItemDrops},
    skill::*,
    unit::{Corpse, Unit},
};

use util::{
    cleanup::CleanupStrategy,
    math::{intersect_aabb, Aabb as UtilAabb, AabbComponent},
    random::SharedRng,
};

use bevy::{
    animation::RepeatAnimation,
    asset::{Assets, Handle},
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::{Event, EventReader, EventWriter},
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::aabb::ShowAabbGizmo,
    hierarchy::DespawnRecursiveExt,
    log::debug,
    math::{Quat, Vec3},
    pbr::{MaterialMeshBundle, PbrBundle, StandardMaterial},
    prelude::{Deref, DerefMut},
    render::{
        mesh::{
            shape::{Circle, Icosphere},
            Mesh,
        },
        prelude::SpatialBundle,
        //primitives::Aabb,
    },
    scene::SceneBundle,
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
    utils::default,
};

use std::borrow::Cow;

pub fn handle_contact(
    mut commands: Commands,
    time: Res<Time>,
    metadata: Res<MetadataResources>,
    mut game_state: ResMut<GameState>,
    mut ground_drops: ResMut<GroundItemDrops>,
    mut random: ResMut<SharedRng>,
    mut skill_events: EventReader<SkillContactEvent>,
    mut skill_q: Query<(Entity, &mut Transform, &mut Invulnerability, &mut SkillUse)>,
    mut unit_q: Query<
        (
            Entity,
            &mut Unit,
            &mut Actions,
            &mut AudioActions,
            &mut AnimationState,
            Option<&Corpse>,
        ),
        Without<SkillUse>,
    >,
) {
    for event in skill_events.read() {
        let Ok(
            [(_, mut attacker, _, _, _, _), (d_entity, mut defender, mut d_actions, mut d_audio, mut d_anim_state, d_corpse)],
        ) = unit_q.get_many_mut([event.owner_entity, event.defender_entity])
        else {
            panic!("Unable to query attacker and/or defender unit(s)");
        };

        if d_corpse.is_some() {
            continue;
        }

        let (s_entity, mut s_transform, mut invulnerability, mut instance) =
            skill_q.get_mut(event.entity).unwrap();
        let combat_result =
            defender.handle_attack(&attacker, &metadata.rpg, &mut random.0, &instance.damage);

        match combat_result {
            CombatResult::Attack(attack) => match attack {
                AttackResult::Blocked => {
                    debug!("blocked");
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
                    debug!("dodge");
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

        if let Some(tickable) = &mut instance.tickable {
            tickable.can_damage = false;
        }

        if defender.is_alive()
            && !instance.effects.is_empty()
            && handle_effects(
                &time,
                &mut random,
                &mut instance,
                &mut s_transform,
                &mut d_actions,
            )
        {
            // println!("Despawning skill");
            commands.entity(event.entity).despawn_recursive();
        }
    }
}

pub fn clean_skills(
    mut commands: Commands,
    time: Res<Time>,
    mut skill_q: Query<(Entity, &Transform, &mut SkillTimer, &SkillUse)>,
) {
    for (entity, transform, mut timer, skill_use) in &mut skill_q {
        if let Some(timer) = &mut timer.0 {
            if timer.tick(time.delta()).finished() {
                commands.entity(entity).despawn_recursive();
                continue;
            }
        }

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
                //println!("direct skill: {info:?}");
                info.frame >= info.info.frames
            }
            SkillInstance::Area(info) => {
                //println!("area skill: {info:?}");
                time.elapsed_seconds() - info.start_time >= info.info.duration
            }
        };

        if despawn {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn update_skill(time: Res<Time>, mut skill_q: Query<(&mut Transform, &mut SkillUse)>) {
    let dt = time.delta_seconds();
    for (mut transform, mut skill_use) in &mut skill_q {
        match &mut skill_use.instance {
            SkillInstance::Projectile(info) => {
                // The skill would have been destroyed if it was expired, advance it
                if let Some(orbit) = &info.orbit {
                    let rotation = Quat::from_rotation_y(
                        ((info.info.speed as f32 / 100.) * time.elapsed_seconds())
                            % std::f32::consts::TAU,
                    );

                    let mut target =
                        Transform::from_translation(orbit.origin).with_rotation(rotation);

                    let Some(orbit_info) = &info.info.orbit else {
                        panic!("expected orbit info");
                    };

                    target.translation += target.forward() * (orbit_info.range as f32 / 100.);
                    target.rotate_x(dt.sin());

                    transform.translation = target.translation;
                    transform.rotate_y(dt.cos());
                    transform.rotate_z(dt.sin());
                } else if let Some(_aerial) = &info.info.aerial {
                    transform.translation = transform.translation
                        + transform.forward() * dt * (info.info.speed as f32 / 100.);
                } else {
                    let move_delta = transform.forward() * (dt * (info.info.speed as f32 / 100.));
                    transform.translation += move_delta;
                    transform.rotate_local_z(std::f32::consts::TAU * dt);
                }
            }
            SkillInstance::Direct(info) => {
                info.frame += 1;
                //println!("direct skill: {info:?}");
            }
            SkillInstance::Area(_) => {
                transform.rotate_local_z(2. * dt);
                //println!("update area skill");
                if let Some(tickable) = &mut skill_use.tickable {
                    tickable.timer.tick(time.delta());
                    if tickable.timer.just_finished() {
                        tickable.can_damage = true;
                        tickable.timer.reset();
                    }
                }
            }
        }
    }
}

pub fn collide_skills(
    mut skill_events: EventWriter<SkillContactEvent>,
    mut skill_q: Query<(
        Entity,
        &Transform,
        &AabbComponent,
        &Invulnerability,
        &SkillUse,
    )>,
    unit_q: Query<(Entity, &Transform, &AabbComponent, &Unit), Without<Corpse>>,
) {
    for (s_entity, s_transform, s_aabb, invulnerability, instance) in &mut skill_q {
        if let Some(tickable) = &instance.tickable {
            if !tickable.can_damage {
                continue;
            }
        }

        for (u_entity, u_transform, u_aabb, unit) in &unit_q {
            if !unit.is_alive()
                || unit.kind == instance.owner_kind
                || u_entity == instance.owner
                || invulnerability.iter().any(|i| i.entity == u_entity)
            {
                continue;
            }

            /*println!(
                "unit {} skill {}",
                u_transform.translation, s_transform.translation
            );*/

            let unit_offset = Vec3::new(0.0, 1.2, 0.0);

            let collision = match &instance.instance {
                SkillInstance::Direct(_) | SkillInstance::Projectile(_) => intersect_aabb(
                    (
                        &(s_transform.translation),
                        &UtilAabb {
                            center: s_aabb.center,
                            half_extents: s_aabb.half_extents,
                        },
                    ),
                    (
                        &(u_transform.translation + unit_offset),
                        &UtilAabb {
                            center: u_aabb.center,
                            half_extents: u_aabb.half_extents,
                        },
                    ),
                ),
                SkillInstance::Area(info) => {
                    s_transform.translation.distance(u_transform.translation)
                        <= info.info.radius as f32 / 100.
                }
            };

            if collision {
                skill_events.send(SkillContactEvent {
                    entity: s_entity,
                    owner_entity: instance.owner,
                    defender_entity: u_entity,
                });
            }
        }
    }
}

pub fn prepare_skill(
    owner: Entity,
    attack_data: &AttackData,
    time: &Time,
    random: &mut SharedRng,
    renderables: &mut RenderResources,
    meshes: &mut Assets<Mesh>,
    skill_info: &SkillTableEntry,
    skill: &Skill,
    unit: &Unit,
    unit_transform: &Transform,
) -> (
    UtilAabb,
    Transform,
    SkillUse,
    Option<PropHandle>,
    Option<Handle<StandardMaterial>>,
) {
    let mut origin = match &skill_info.origin {
        Origin::Direct(_) => {
            unit_transform.translation
                + unit_transform.forward() * (skill_info.use_range as f32 / 100. / 2.)
        }
        Origin::Remote(data) => data.offset + attack_data.target,
        Origin::Locked(_) => unit_transform.translation,
    };

    //println!("{:?}", &skill_info.origin);

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

    let (aabb, skill_use, transform, mesh, material, tickable) = match &skill_info.info {
        SkillInfo::Direct(_) => {
            origin.y = 1.2;

            let aabb = renderables.aabbs["direct_attack"];
            let SkillInfo::Direct(info) = &skill.info else {
                panic!("Expected direct attack")
            };

            let instance = SkillInstance::Direct(DirectInstance {
                info: info.clone(),
                frame: 0,
            });

            let skill_transform =
                Transform::from_translation(origin).looking_to(unit_transform.forward(), Vec3::Y);

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
                let aabb = if renderables.aabbs.contains_key("bolt_01") {
                    renderables.aabbs["bolt_01"]
                } else {
                    let aabb = UtilAabb::from_min_max(
                        Vec3::new(-0.1, -0.1, -0.25),
                        Vec3::new(0.1, 0.1, 0.25),
                    );
                    renderables.aabbs.insert("bolt_01".into(), aabb);
                    aabb
                };

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

                    let aabb = UtilAabb {
                        center: aabb.center,
                        half_extents: aabb.half_extents,
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
            let forward = unit_transform.forward();

            let spawn_transform = if info.orbit.is_some() {
                let mut rot_transform = *unit_transform;
                rot_transform.translation += Vec3::new(0., 1.2, 0.);
                rot_transform.rotate_local_y(std::f32::consts::TAU * (0.5 - random.f32()));

                Transform::from_translation(
                    rot_transform.translation + rot_transform.forward() * 2.,
                )
            } else if info.aerial.is_some() {
                //println!("prepare aerial {attack_data:?}");
                Transform::from_translation(origin).looking_at(attack_data.target, Vec3::Y)
            } else {
                Transform::from_translation(
                    unit_transform.translation + Vec3::new(0., 1.2, 0.) + forward * 0.25,
                )
                .looking_to(forward, Vec3::Y)
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

            let transform = Transform::from_translation(origin + Vec3::new(0.0, 0.01, 0.0))
                .looking_to(Vec3::NEG_Y, Vec3::Y);

            let radius = info.radius as f32 / 100.;
            let key = format!("area_radius_{}", info.radius);

            let (mesh_handle, aabb) = if renderables.meshes.contains_key(key.as_str()) {
                (
                    renderables.meshes[key.as_str()].clone_weak(),
                    renderables.aabbs[key.as_str()],
                )
            } else {
                let mut mesh: Mesh = Circle::new(radius).into();
                let _ = mesh.generate_tangents();
                //let aabb = mesh.compute_aabb().unwrap();

                // 2d shapes are on the XY plane
                let aabb = UtilAabb::from_min_max(
                    Vec3::new(-radius, -radius, 0.0),
                    Vec3::new(radius, radius, 0.5),
                );

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
        unit.kind,
        skill.id,
        skill.damage.clone(),
        skill_use,
        effects,
        tickable,
    );

    (aabb, transform, instance, mesh, material)
}

pub(crate) fn spawn_instance(
    commands: &mut Commands,
    aabb: UtilAabb,
    transform: Transform,
    skill_use_instance: SkillUse,
    mesh: Option<PropHandle>,
    material: Option<Handle<StandardMaterial>>,
) {
    let skill_use = SkillUseBundle::new(skill_use_instance, SkillTimer(None));

    let common_bundle = (
        GameSessionCleanup,
        CleanupStrategy::DespawnRecursive,
        AabbComponent(aabb),
        Invulnerability::default(),
        skill_use,
    );

    match &common_bundle.4.skill.instance {
        SkillInstance::Direct(_) => {
            //println!("spawning direct skill {}", transform.translation);

            commands.spawn((
                common_bundle,
                ShowAabbGizmo::default(),
                SpatialBundle::from_transform(transform),
            ));
        }
        SkillInstance::Projectile(_) => {
            //println!("spawning projectile skill");

            let handle = mesh.unwrap();
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
                    //AabbGizmo::default(),
                ));
            }
        }
        SkillInstance::Area(_) => {
            //println!("spawning area skill");

            let handle = mesh.unwrap();
            if let PropHandle::Mesh(handle) = handle {
                commands.spawn((
                    common_bundle,
                    MaterialMeshBundle {
                        transform,
                        mesh: handle,
                        material: material.unwrap(),
                        ..default()
                    },
                    //AabbGizmo::default(),
                ));
            }
        }
    }
}

/// Returns `true` if the skill should be destroyed
fn handle_effects(
    time: &Time,
    random: &mut SharedRng,
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
}
