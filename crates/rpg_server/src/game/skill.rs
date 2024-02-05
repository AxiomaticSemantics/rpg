use super::{
    plugin::{AabbResources, GameSessionCleanup, GameState},
    unit::CorpseTimer,
};
use crate::{
    account::AccountInstance, assets::MetadataResources, net::server::NetworkParamsRW,
    server_state::ServerMetadataResource,
};

use rpg_core::{
    combat::CombatResult,
    item::ItemDrops,
    skill::{
        effect::*, skill_tables::SkillTableEntry, AreaInstance, DirectInstance, OrbitData, Origin,
        ProjectileInstance, ProjectileShape, Skill, SkillInfo, SkillInstance, TimerDescriptor,
    },
    unit::UnitKind,
};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::{Action, ActionData, Actions, AttackData, KnockbackData as KnockbackActionData},
    item::GroundItemDrops,
    skill::{
        Invulnerability, InvulnerabilityTimer, SkillContactEvent, SkillTimer, SkillUse,
        SkillUseBundle, Tickable,
    },
    unit::{Corpse, Unit},
};

use util::{
    cleanup::CleanupStrategy,
    math::{intersect_aabb, AabbComponent},
    random::{Rng, SharedRng},
};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::{EventReader, EventWriter},
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    log::{debug, info},
    math::{bounding::Aabb3d, Vec3},
    time::{Time, Timer, TimerMode},
    transform::{components::Transform, TransformBundle},
};

use lightyear::shared::NetworkTarget;

use std::borrow::Cow;

#[derive(Debug, Component)]
pub(crate) struct SkillOwner {
    pub entity: Entity,
    pub owner_kind: UnitKind,
}

pub struct SkillTarget {
    origin: Vec3,
    target: Vec3,
}

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

fn get_target_info(
    caster_transform: &Transform,
    skill_meta: &SkillTableEntry,
    attack_data: &AttackData,
) -> SkillTarget {
    let origin = match &skill_meta.origin {
        Origin::Direct(_) => {
            caster_transform.translation
                + Vec3::new(0., 1.2, 0.)
                + caster_transform.forward() * (skill_meta.use_range as f32 / 100. / 2.)
        }
        Origin::Remote(data) => data.offset + attack_data.target,
        Origin::Locked(_) => caster_transform.translation,
    };

    SkillTarget {
        origin,
        target: attack_data.target,
    }
}

pub(crate) fn prepare_skill(
    attack_data: &AttackData,
    aabbs: &mut AabbResources,
    skill_meta: &SkillTableEntry,
    skill: &Skill,
    unit: &Unit,
    unit_transform: &Transform,
) -> (Aabb3d, Transform, SkillUse, Option<SkillTimer>) {
    // TODO move to a new fn in rpg_util
    let mut target = get_target_info(unit_transform, skill_meta, attack_data);

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
        .collect(); */

    let timer = if let Some(timer) = &skill_meta.timer {
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

    let (aabb, skill_use, transform) = match &skill_meta.info {
        SkillInfo::Direct(_) => {
            let aabb = aabbs.aabbs["direct_attack"];
            let SkillInfo::Direct(info) = &skill.info else {
                panic!("Expected direct attack")
            };

            let instance = SkillInstance::Direct(DirectInstance {
                info: info.clone(),
                frame: 0,
            });

            let skill_transform =
                Transform::from_translation(target.origin).looking_at(target.target, Vec3::Y);

            (aabb, instance, skill_transform)
        }
        SkillInfo::Projectile(info) => {
            // debug!("spawn {speed} {duration} {size}");

            let aabb = if info.shape == ProjectileShape::Box {
                aabbs.aabbs["bolt_01"]
            } else {
                // FIXME convert to sphere collider
                let radius = info.size as f32 / 100. / 2.;
                let key = format!("orb_radius_{}", info.size);

                let aabb = if aabbs.aabbs.contains_key(key.as_str()) {
                    aabbs.aabbs[key.as_str()]
                } else {
                    let aabb = Aabb3d {
                        min: Vec3::splat(-radius),
                        max: Vec3::splat(radius),
                    };

                    aabbs.aabbs.insert(Cow::Owned(key.as_str().into()), aabb);

                    aabb
                };

                aabb
            };

            let spawn_transform = if info.orbit.is_some() {
                let mut origin_transform =
                    Transform::from_translation(target.origin).looking_at(target.target, Vec3::Y);
                origin_transform.translation += origin_transform.forward() * 2.;

                origin_transform
            } else if info.aerial.is_some() {
                //println!("prepare aerial {attack_data:?}");
                Transform::from_translation(target.origin).looking_at(target.target, Vec3::Y)
            } else {
                Transform::from_translation(target.origin).looking_to(target.target, Vec3::Y)
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

            (aabb, instance_info, spawn_transform)
        }
        SkillInfo::Area(info) => {
            let skill_instance = SkillInstance::Area(AreaInstance { info: info.clone() });

            let transform = Transform::from_translation(target.origin + Vec3::new(0.0, 0.01, 0.0))
                .looking_to(Vec3::NEG_Y, Vec3::Y);

            let radius = info.radius as f32 / 100.;
            let key = format!("area_radius_{}", info.radius);

            let aabb = if aabbs.aabbs.contains_key(key.as_str()) {
                aabbs.aabbs[key.as_str()]
            } else {
                // 2d shapes are on the XY plane
                let aabb = Aabb3d {
                    min: Vec3::new(-radius, -radius, 0.0),
                    max: Vec3::new(radius, radius, 0.5),
                };

                aabbs.aabbs.insert(Cow::Owned(key.as_str().into()), aabb);

                aabb
            };

            (aabb, skill_instance, transform)
        }
    };

    let instance = SkillUse::new(
        unit.uid,
        skill.id,
        skill.damage.clone(),
        skill_use,
        vec![], // FIXME effects,
    );

    (aabb, transform, instance, timer)
}

pub(crate) fn spawn_instance(
    commands: &mut Commands,
    aabb: Aabb3d,
    transform: Transform,
    skill_use_instance: SkillUse,
    owner: Entity,
    owner_kind: UnitKind,
    timer: Option<SkillTimer>,
) {
    let skill_use = SkillUseBundle::new(skill_use_instance);

    let entity = commands
        .spawn((
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            AabbComponent(aabb),
            Invulnerability::default(),
            skill_use,
            SkillOwner {
                entity: owner,
                owner_kind,
            },
            TransformBundle::from(transform),
        ))
        .id();

    if let Some(timer) = timer {
        commands.entity(entity).insert(timer);
    }
}

pub(crate) fn collide_skills(
    mut skill_events: EventWriter<SkillContactEvent>,
    skill_q: Query<(
        Entity,
        &Transform,
        &AabbComponent,
        &Invulnerability,
        &SkillUse,
        &SkillOwner,
        Option<&mut SkillTimer>,
    )>,
    // owner_q: Query<&Unit>,
    unit_q: Query<(Entity, &Transform, &AabbComponent, &Unit), Without<Corpse>>,
) {
    for (s_entity, s_transform, s_aabb, invulnerability, instance, owner, mut timer) in &skill_q {
        if let Some(SkillTimer::Tickable(tickable)) = &mut timer {
            if !tickable.can_damage {
                continue;
            }
        }

        let owner_kind = owner.owner_kind;
        for (u_entity, u_transform, u_aabb, unit) in &unit_q {
            if !unit.is_alive()
                || unit.uid == instance.owner
                || unit.kind == owner_kind
                || invulnerability.iter().any(|i| i.entity == u_entity)
            {
                continue;
            }

            /*info!(
                "unit {} skill {}",
                u_transform.translation, s_transform.translation
            );*/

            let unit_offset = Vec3::new(0.0, 1.2, 0.0);

            let collision = match &instance.instance {
                SkillInstance::Direct(_) | SkillInstance::Projectile(_) => intersect_aabb(
                    (s_transform.translation, s_transform.rotation, s_aabb.0),
                    (
                        (u_transform.translation + unit_offset),
                        u_transform.rotation,
                        u_aabb.0,
                    ),
                ),
                SkillInstance::Area(info) => {
                    s_transform.translation.distance(u_transform.translation)
                        <= info.info.radius as f32 / 100.
                }
            };

            //info!("collide {instance:?} {collision}");
            if collision {
                skill_events.send(SkillContactEvent {
                    entity: s_entity,
                    owner: owner.entity,
                    owner_kind: owner.owner_kind,
                    defender: u_entity,
                });
            }
        }
    }
}

pub fn handle_contacts(
    mut commands: Commands,
    time: Res<Time>,
    metadata: Res<MetadataResources>,
    game_state: Res<GameState>,
    mut server_metadata: ResMut<ServerMetadataResource>,
    mut net_params: NetworkParamsRW,
    mut ground_drops: ResMut<GroundItemDrops>,
    mut rng: ResMut<SharedRng>,
    mut skill_events: EventReader<SkillContactEvent>,
    mut skill_q: Query<(
        Entity,
        &mut Transform,
        &mut Invulnerability,
        &mut SkillUse,
        Option<&mut SkillTimer>,
    )>,
    mut unit_q: Query<
        (
            Entity,
            &mut Unit,
            &mut Actions,
            &Transform,
            Option<&AccountInstance>,
            Option<&Corpse>,
        ),
        Without<SkillUse>,
    >,
) {
    for event in skill_events.read() {
        let Ok(
            [(_, mut attacker, _, _, a_account, _), (d_entity, mut defender, mut d_actions, d_transform, d_account, d_corpse)],
        ) = unit_q.get_many_mut([event.owner, event.defender])
        else {
            panic!("Unable to query attacker and/or defender unit(s)");
        };

        if d_corpse.is_some() {
            continue;
        }

        let (s_entity, mut s_transform, mut invulnerability, mut instance, timer) =
            skill_q.get_mut(event.entity).unwrap();
        let combat_result =
            defender.handle_attack(&attacker, &metadata.0, &mut rng.0, &instance.damage);

        info!("{combat_result:?}");

        match &combat_result {
            CombatResult::Blocked => {
                debug!("blocked");
                /*if defender.kind == UnitKind::Hero {
                    game_state.session_stats.blocks += 1;
                } else {
                    game_state.session_stats.times_blocked += 1;
                }*/

                net_params.server.send_message_to_target::<Channel1, _>(
                    SCUnitAnim {
                        uid: defender.uid,
                        anim: 1,
                    },
                    NetworkTarget::All,
                );

                match &instance.instance {
                    SkillInstance::Direct(_) | SkillInstance::Projectile(_) => {
                        commands.entity(s_entity).despawn_recursive();
                        continue;
                    }
                    SkillInstance::Area(_) => {}
                }
            }
            CombatResult::Dodged => {
                debug!("dodge");
                /*if defender.kind == UnitKind::Hero {
                    game_state.session_stats.dodges += 1;
                } else {
                    game_state.session_stats.times_dodged += 1;
                }*/
                net_params.server.send_message_to_target::<Channel1, _>(
                    SCUnitAnim {
                        uid: defender.uid,
                        anim: 0,
                    },
                    NetworkTarget::All,
                );
            }
            CombatResult::Damage(damage) => {
                /*if defender.kind == UnitKind::Villain {
                    game_state.session_stats.hits += 1;
                } else {
                    game_state.session_stats.villain_hits += 1;
                }*/

                if defender.kind == UnitKind::Hero {
                    // TODO should probably move the lookup out of game state?
                    let id_info = game_state.get_id_info_from_uid(defender.uid).unwrap();

                    /*let client = net_params
                    .context
                    .get_client_from_account_id(d_account.as_ref().unwrap().0.info.id)
                    .unwrap();*/
                    net_params.server.send_message_to_target::<Channel1, _>(
                        SCCombatResult(combat_result.clone()),
                        NetworkTarget::Only(vec![id_info.client_id]),
                    );
                } else if defender.kind == UnitKind::Villain {
                    net_params.server.send_message_to_target::<Channel1, _>(
                        SCDamage {
                            uid: defender.uid,
                            damage: damage.clone(),
                        },
                        NetworkTarget::All,
                    );
                }

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
            CombatResult::Death(damage) => {
                debug!("death");

                d_actions.reset();

                if defender.kind == UnitKind::Villain {
                    /*game_state.session_stats.kills += 1;
                    game_state.session_stats.hits += 1;*/

                    if let Some(items) = defender.handle_death(
                        &mut attacker,
                        &metadata.0,
                        &mut rng.0,
                        &mut server_metadata.0.next_uid,
                    ) {
                        //game_state.session_stats.items_spawned += death.items.len() as u32;

                        let drops = ItemDrops {
                            source: defender.uid,
                            items: items.clone(),
                        };

                        ground_drops.0.push(drops.clone());

                        net_params.server.send_message_to_target::<Channel1, _>(
                            SCSpawnItems {
                                position: d_transform.translation,
                                items: drops,
                            },
                            NetworkTarget::All,
                        );
                    }

                    // TODO XP for attacker

                    net_params.server.send_message_to_target::<Channel1, _>(
                        SCVillainDeath(defender.uid),
                        NetworkTarget::All,
                    );
                } else {
                    // game_state.session_stats.villain_hits += 1;
                    net_params.server.send_message_to_target::<Channel1, _>(
                        SCHeroDeath(defender.uid),
                        NetworkTarget::All,
                    );
                }

                commands.entity(event.defender).insert((
                    Corpse,
                    CorpseTimer(Timer::from_seconds(60., TimerMode::Once)),
                ));
            }
            _ => {}
        }

        if let Some(mut timer) = timer {
            if let SkillTimer::Tickable(ref mut tickable) = &mut *timer {
                tickable.can_damage = false;
            }
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
            // debug!("Despawning skill");
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
                direction: *skill_transform.forward(),
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
