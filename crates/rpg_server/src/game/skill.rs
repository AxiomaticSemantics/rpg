use super::plugin::{AabbResources, GameSessionCleanup, GameState};
use crate::{assets::MetadataResources, server_state::ServerMetadataResource};

use rpg_core::{
    combat::{AttackResult, CombatResult},
    skill::{
        effect::*, skill_tables::SkillTableEntry, AreaInstance, DirectInstance, OrbitData, Origin,
        ProjectileInstance, ProjectileShape, Skill, SkillInfo, SkillInstance,
    },
    unit::UnitKind,
};
use rpg_util::{
    actions::{Action, ActionData, Actions, AttackData, KnockbackData as KnockbackActionData},
    item::{GroundItemDrop, GroundItemDrops},
    skill::{
        Invulnerability, InvulnerabilityTimer, SkillContactEvent, SkillTimer, SkillUse,
        SkillUseBundle, Tickable,
    },
    unit::{Corpse, Unit},
};

use util::{
    cleanup::CleanupStrategy,
    math::{intersect_aabb, Aabb, AabbComponent},
    random::{Rng, SharedRng},
};

use bevy::{
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    log::{debug, info},
    math::Vec3,
    time::{Time, Timer, TimerMode},
    transform::{components::Transform, TransformBundle},
};

use std::borrow::Cow;

pub fn update_invulnerability(
    time: Res<Time>,
    mut skill_q: Query<&mut Invulnerability, With<SkillUse>>,
) {
    for mut invulnerability in &mut skill_q {
        invulnerability.iter_mut().for_each(|i| {
            i.timer.tick(time.delta());
        });

        invulnerability.retain(|i| !i.timer.finished());
    }
}

pub(crate) fn prepare_skill(
    owner: Entity,
    attack_data: &AttackData,
    time: &Time,
    random: &mut SharedRng,
    aabbs: &mut AabbResources,
    skill_info: &SkillTableEntry,
    skill: &Skill,
    unit: &Unit,
    unit_transform: &Transform,
) -> (Aabb, Transform, SkillUse) {
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

    let (aabb, skill_use, transform, tickable) = match &skill_info.info {
        SkillInfo::Direct(_) => {
            origin.y = 1.2;

            let aabb = aabbs.aabbs["direct_attack"];
            let SkillInfo::Direct(info) = &skill.info else {
                panic!("Expected direct attack")
            };

            let instance = SkillInstance::Direct(DirectInstance {
                info: info.clone(),
                frame: 0,
            });

            let skill_transform =
                Transform::from_translation(origin).looking_to(unit_transform.forward(), Vec3::Y);

            (aabb, instance, skill_transform, None)
        }
        SkillInfo::Projectile(info) => {
            // debug!("spawn {speed} {duration} {size}");

            let tickable = info.tick_rate.as_ref().map(|tr| Tickable {
                timer: Timer::from_seconds(*tr, TimerMode::Repeating),
                can_damage: true,
            });

            let aabb = if info.shape == ProjectileShape::Box {
                let aabb = if aabbs.aabbs.contains_key("bolt_01") {
                    aabbs.aabbs["bolt_01"]
                } else {
                    let aabb =
                        Aabb::from_min_max(Vec3::new(-0.1, -0.1, -0.25), Vec3::new(0.1, 0.1, 0.25));
                    aabbs.aabbs.insert("bolt_01".into(), aabb);
                    aabb
                };

                aabb
            } else {
                // FIXME convert to sphere collider
                let radius = info.size as f32 / 100. / 2.;
                let key = format!("orb_radius_{}", info.size);

                let aabb = if aabbs.aabbs.contains_key(key.as_str()) {
                    aabbs.aabbs[key.as_str()]
                } else {
                    let aabb = Aabb::from_min_max(Vec3::splat(-radius), Vec3::splat(radius));

                    aabbs.aabbs.insert(Cow::Owned(key.as_str().into()), aabb);

                    aabb
                };

                aabb
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

            (aabb, instance_info, spawn_transform, tickable)
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

            let aabb = if aabbs.aabbs.contains_key(key.as_str()) {
                aabbs.aabbs[key.as_str()]
            } else {
                // 2d shapes are on the XY plane
                let aabb = Aabb::from_min_max(
                    Vec3::new(-radius, -radius, 0.0),
                    Vec3::new(radius, radius, 0.5),
                );

                aabbs.aabbs.insert(Cow::Owned(key.as_str().into()), aabb);

                aabb
            };

            (aabb, skill_instance, transform, tickable)
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

    (aabb, transform, instance)
}

pub(crate) fn spawn_instance(
    commands: &mut Commands,
    aabb: Aabb,
    transform: Transform,
    skill_use_instance: SkillUse,
) {
    let skill_use = SkillUseBundle::new(skill_use_instance, SkillTimer(None));

    commands.spawn((
        GameSessionCleanup,
        CleanupStrategy::DespawnRecursive,
        AabbComponent(aabb),
        Invulnerability::default(),
        skill_use,
        TransformBundle::from(transform),
    ));
}

pub(crate) fn clean_skills(
    mut commands: Commands,
    time: Res<Time>,
    mut skill_q: Query<(Entity, &Transform, &mut SkillTimer, &SkillUse)>,
) {
    for (entity, transform, mut timer, skill_use) in &mut skill_q {
        if let Some(timer) = &mut timer.0 {
            if timer.tick(time.delta()).finished() {
                // TODO send message to all clients that have spawned this skill
                // or rely on the client to auto-despawn?
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

pub(crate) fn collide_skills(
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
            info!("{:?}", unit.info);
            if !unit.is_alive()
                || unit.kind == instance.owner_kind
                || u_entity == instance.owner
                || invulnerability.iter().any(|i| i.entity == u_entity)
            {
                continue;
            }

            // info!("{:?}", unit.info);
            /*println!(
                "unit {} skill {}",
                u_transform.translation, s_transform.translation
            );*/

            let unit_offset = Vec3::new(0.0, 1.2, 0.0);

            let collision = match &instance.instance {
                SkillInstance::Direct(_) | SkillInstance::Projectile(_) => intersect_aabb(
                    (
                        &(s_transform.translation),
                        &Aabb {
                            center: s_aabb.center,
                            half_extents: s_aabb.half_extents,
                        },
                    ),
                    (
                        &(u_transform.translation + unit_offset),
                        &Aabb {
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

pub fn handle_contacts(
    mut commands: Commands,
    time: Res<Time>,
    metadata: Res<MetadataResources>,
    mut server_metadata: ResMut<ServerMetadataResource>,
    mut game_state: ResMut<GameState>,
    mut ground_drops: ResMut<GroundItemDrops>,
    mut rng: ResMut<SharedRng>,
    mut skill_events: EventReader<SkillContactEvent>,
    mut skill_q: Query<(Entity, &mut Transform, &mut Invulnerability, &mut SkillUse)>,
    mut unit_q: Query<(Entity, &mut Unit, &mut Actions, Option<&Corpse>), Without<SkillUse>>,
) {
    for event in skill_events.read() {
        let Ok([(_, mut attacker, _, _), (d_entity, mut defender, mut d_actions, d_corpse)]) =
            unit_q.get_many_mut([event.owner_entity, event.defender_entity])
        else {
            panic!("Unable to query attacker and/or defender unit(s)");
        };

        if d_corpse.is_some() {
            continue;
        }

        let (s_entity, mut s_transform, mut invulnerability, mut instance) =
            skill_q.get_mut(event.entity).unwrap();
        let combat_result =
            defender.handle_attack(&attacker, &metadata.0, &mut rng.0, &instance.damage);

        match combat_result {
            CombatResult::Attack(attack) => match attack {
                AttackResult::Blocked => {
                    debug!("blocked");
                    /*if defender.kind == UnitKind::Hero {
                        game_state.session_stats.blocks += 1;
                    } else {
                        game_state.session_stats.times_blocked += 1;
                    }*/

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
                    /*if defender.kind == UnitKind::Hero {
                        game_state.session_stats.dodges += 1;
                    } else {
                        game_state.session_stats.times_dodged += 1;
                    }*/
                }
                _ => {
                    /*if defender.kind == UnitKind::Villain {
                        game_state.session_stats.hits += 1;
                    } else {
                        game_state.session_stats.villain_hits += 1;
                    }*/

                    if let SkillInstance::Projectile(_) = &instance.instance {
                        if instance
                            .effects
                            .iter()
                            .any(|e| matches!(e.info, EffectInfo::Pierce(_)))
                        {
                            invulnerability.push(InvulnerabilityTimer {
                                entity: d_entity,
                                timer: Timer::from_seconds(0.5, TimerMode::Once),
                            });
                        }
                    }
                }
            },
            CombatResult::Death(_) => {
                debug!("death");

                d_actions.reset();

                if defender.kind == UnitKind::Villain {
                    /*game_state.session_stats.kills += 1;
                    game_state.session_stats.hits += 1;*/

                    if let Some(death) = defender.handle_death(
                        &mut attacker,
                        &metadata.0,
                        &mut rng.0,
                        &mut server_metadata.0.next_uid,
                    ) {
                        //game_state.session_stats.items_spawned += death.items.len() as u32;

                        ground_drops.0.push(GroundItemDrop {
                            source: event.defender_entity,
                            items: death.items,
                        });
                    }
                } else {
                    // game_state.session_stats.villain_hits += 1;
                    // FIXME let the client know that it has died
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
                &mut rng.0,
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

/// Returns `true` if the skill should be destroyed
fn handle_effects(
    time: &Time,
    rng: &mut Rng,
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

            skill_transform.rotate_y(0.5 - rng.f32());

            data.count > info.chains
        } else {
            false
        };

    despawn
}
