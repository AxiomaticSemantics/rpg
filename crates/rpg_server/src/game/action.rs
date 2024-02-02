use super::{plugin::AabbResources, skill};
use crate::{account::AccountInstance, assets::MetadataResources, net::server::NetworkParamsRW};

use rpg_core::{skill::SkillUseResult, unit::UnitKind};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::{ActionData, Actions, State},
    unit::{Corpse, Unit},
};

use util::random::SharedRng;

use lightyear::shared::NetworkTarget;

use bevy::{
    ecs::{
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
    math::Vec3,
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};

pub(crate) fn action(
    mut commands: Commands,
    mut net_params: NetworkParamsRW,
    time: Res<Time>,
    metadata: Res<MetadataResources>,
    mut aabbs: ResMut<AabbResources>,
    mut rng: ResMut<SharedRng>,
    mut unit_q: Query<
        (
            Entity,
            &mut Unit,
            &mut Transform,
            &mut Actions,
            Option<&AccountInstance>,
        ),
        Without<Corpse>,
    >,
) {
    use std::f32::consts;

    let dt = time.delta_seconds();

    for (entity, mut unit, mut transform, mut actions, account) in &mut unit_q {
        // All of the following action handlers are in a strict order

        // First react to any knockback events, this blocks all other actions
        if let Some(action) = &mut actions.knockback {
            let ActionData::Knockback(knockback) = action.data else {
                panic!("expected knockback data");
            };

            if time.elapsed_seconds() < knockback.start + knockback.duration {
                let target =
                    -transform.forward() * time.delta_seconds() * (knockback.speed as f32 / 100.);
                transform.translation += target;
            } else {
                action.state = State::Completed;
            }

            continue;
        }

        // Next if the user is able to initiate an attack do so
        if let Some(action) = &mut actions.attack {
            let ActionData::Attack(attack) = action.data else {
                panic!("expected attack data");
            };

            match &mut action.state {
                State::Pending => {
                    let distance = (attack.user.distance(attack.target) * 100.).round() as u32;
                    let skill_id = unit.active_skills.primary.skill.unwrap();
                    assert_eq!(skill_id, attack.skill_id);

                    let Some(skill_info) = metadata.0.skill.skills.get(&skill_id) else {
                        panic!("skill metadata not found");
                    };

                    match unit.can_use_skill(&metadata.0, attack.skill_id, distance) {
                        SkillUseResult::Blocked
                        | SkillUseResult::OutOfRange
                        | SkillUseResult::InsufficientResources => {
                            action.state = State::Completed;
                            //println!("skill use blocked {:?}", unit.skills);
                            continue;
                        }
                        SkillUseResult::Ok => {}
                        SkillUseResult::Error => {
                            panic!("Skill use error");
                        }
                    }

                    let duration = skill_info.use_duration_secs
                        * unit.stats.vitals.stats["Cooldown"].value.f32();

                    action.timer = Some(Timer::from_seconds(duration, TimerMode::Once));
                    action.state = State::Timer;
                }
                State::Active => {
                    let distance = (attack.user.distance(attack.target) * 100.).round() as u32;
                    let skill_use_result = unit.use_skill(&metadata.0, attack.skill_id, distance);
                    match skill_use_result {
                        SkillUseResult::Ok => {}
                        _ => panic!("This should never happen. {skill_use_result:?}"),
                    }

                    let Some(skill) = unit.skills.iter().find(|s| s.id == attack.skill_id) else {
                        panic!("skill missing");
                    };
                    let Some(skill_info) = metadata.0.skill.skills.get(&attack.skill_id) else {
                        panic!("skill metadata not found");
                    };

                    /* TODO track stats per player in server
                    if unit.kind == UnitKind::Hero {
                        state.session_stats.attacks += 1;
                    } else {
                        state.session_stats.villain_attacks += 1;
                    }*/

                    let (skill_aabb, skill_transform, skill_use) = skill::prepare_skill(
                        &attack, &time, &mut rng, &mut aabbs, skill_info, skill, &unit, &transform,
                    );

                    info!("spawning skill");
                    skill::spawn_instance(
                        &mut commands,
                        skill_aabb,
                        skill_transform,
                        skill_use,
                        entity,
                    );

                    net_params.server.send_message_to_target::<Channel1, _>(
                        SCSpawnSkill {
                            id: skill.id,
                            uid: unit.uid,
                            origin: attack.origin,
                            target: attack.target,
                        },
                        NetworkTarget::All,
                    );

                    // The action is completed at this point
                    action.state = State::Completed;
                    action.timer = None;
                }
                _ => {}
            }
            continue;
        }

        if let Some(action) = &mut actions.look {
            let wanted = if let ActionData::LookPoint(target) = action.data {
                transform.looking_at(target, Vec3::Y)
            } else if let ActionData::LookDir(dir) = action.data {
                transform.looking_to(dir, Vec3::Y)
            } else {
                panic!("Invalid action data");
            };

            let diff = transform.rotation.angle_between(wanted.rotation);
            let speed = consts::TAU * 1.33;
            let ratio = (speed * dt) / diff;

            let lerped = transform
                .rotation
                .slerp(wanted.rotation, ratio.clamp(0., 1.));
            if transform.rotation != lerped {
                transform.rotation = lerped;
            }

            let direction = transform.forward();

            if unit.kind == UnitKind::Hero {
                let client = net_params
                    .context
                    .get_client_from_account_id(account.as_ref().unwrap().0.info.id)
                    .unwrap();

                net_params.server.send_message_to_target::<Channel1, _>(
                    SCRotPlayer(direction),
                    NetworkTarget::Only(vec![client.id]),
                );

                net_params.server.send_message_to_target::<Channel1, _>(
                    SCRotUnit {
                        uid: unit.uid,
                        direction,
                    },
                    NetworkTarget::AllExcept(vec![client.id]),
                );
            } else {
                net_params.server.send_message_to_target::<Channel1, _>(
                    SCRotUnit {
                        uid: unit.uid,
                        direction,
                    },
                    NetworkTarget::All,
                );
            }

            action.state = State::Completed;
        }

        if let Some(action) = &mut actions.movement {
            match &action.state {
                State::Pending => {
                    let movespeed = unit.get_effective_movement_speed() as f32 / 100. * dt;
                    if unit.can_run() {
                        unit.stats.consume_stamina(dt);
                    }

                    let translation = transform.forward() * movespeed;
                    transform.translation += translation;

                    if unit.kind == UnitKind::Hero {
                        let client = net_params
                            .context
                            .get_client_from_account_id(account.as_ref().unwrap().0.info.id)
                            .unwrap();

                        net_params.server.send_message_to_target::<Channel1, _>(
                            SCMovePlayer(transform.translation),
                            NetworkTarget::Only(vec![client.id]),
                        );

                        net_params.server.send_message_to_target::<Channel1, _>(
                            SCMoveUnit {
                                uid: unit.uid,
                                position: transform.translation,
                            },
                            NetworkTarget::AllExcept(vec![client.id]),
                        );
                    } else {
                        net_params.server.send_message_to_target::<Channel1, _>(
                            SCMoveUnit {
                                uid: unit.uid,
                                position: transform.translation,
                            },
                            NetworkTarget::All,
                        );
                    }

                    action.state = State::Active;
                }
                State::Active => {
                    let movespeed = unit.get_effective_movement_speed() as f32 / 100. * dt;
                    if unit.can_run() {
                        unit.stats.consume_stamina(dt);
                    }

                    let translation = transform.forward() * movespeed;
                    transform.translation += translation;

                    if unit.kind == UnitKind::Hero {
                        let client = net_params
                            .context
                            .get_client_from_account_id(account.as_ref().unwrap().0.info.id)
                            .unwrap();

                        net_params.server.send_message_to_target::<Channel1, _>(
                            SCMovePlayer(transform.translation),
                            NetworkTarget::Only(vec![client.id]),
                        );

                        net_params.server.send_message_to_target::<Channel1, _>(
                            SCMoveUnit {
                                uid: unit.uid,
                                position: transform.translation,
                            },
                            NetworkTarget::AllExcept(vec![client.id]),
                        );
                    } else {
                        net_params.server.send_message_to_target::<Channel1, _>(
                            SCMoveUnit {
                                uid: unit.uid,
                                position: transform.translation,
                            },
                            NetworkTarget::All,
                        );
                    }
                }
                State::Finalize => {
                    let movespeed = unit.get_effective_movement_speed() as f32 / 100. * dt;
                    if unit.can_run() {
                        unit.stats.consume_stamina(dt);
                    }

                    let translation = transform.forward() * movespeed;
                    transform.translation += translation;

                    if unit.kind == UnitKind::Hero {
                        let client = net_params
                            .context
                            .get_client_from_account_id(account.as_ref().unwrap().0.info.id)
                            .unwrap();

                        net_params.server.send_message_to_target::<Channel1, _>(
                            SCMovePlayerEnd(transform.translation),
                            NetworkTarget::Only(vec![client.id]),
                        );

                        net_params.server.send_message_to_target::<Channel1, _>(
                            SCMoveUnitEnd {
                                uid: unit.uid,
                                position: transform.translation,
                            },
                            NetworkTarget::AllExcept(vec![client.id]),
                        );
                    } else {
                        net_params.server.send_message_to_target::<Channel1, _>(
                            SCMoveUnitEnd {
                                uid: unit.uid,
                                position: transform.translation,
                            },
                            NetworkTarget::All,
                        );
                    }
                    action.state = State::Completed;
                }
                _ => {}
            }
        }
    }
}
