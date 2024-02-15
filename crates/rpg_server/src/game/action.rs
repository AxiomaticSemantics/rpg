use super::{
    plugin::{AabbResources, GameState},
    skill,
    unit::can_move,
};
use crate::{account::AccountInstance, assets::MetadataResources, net::server::NetworkParamsRW};

use rpg_core::{skill::SkillUseResult, unit::UnitKind};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::{ActionData, Actions, State},
    skill::{SkillSlots, Skills},
    unit::{Corpse, Unit},
};

use util::math::AabbComponent;

use bevy::{
    ecs::{
        entity::Entity,
        query::{Changed, Without},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::info,
    math::Vec3,
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};

#[derive(Default, Resource)]
pub(crate) struct MovingUnits(pub(crate) Vec<Entity>);

// TODO split this up further, rpg_util actions needs to be reworked, each action should implement
// it's handlers
// actions should accumulate responses and handle dispatching all network message at once
pub(crate) fn action(
    mut commands: Commands,
    mut net_params: NetworkParamsRW,
    mut game_state: ResMut<GameState>,
    mut moving_units: ResMut<MovingUnits>,
    time: Res<Time>,
    metadata: Res<MetadataResources>,
    mut aabbs: ResMut<AabbResources>,
    mut unit_q: Query<
        (
            Entity,
            &mut Unit,
            &mut Skills,
            &SkillSlots,
            &mut Transform,
            &AabbComponent,
            &mut Actions,
            Option<&AccountInstance>,
        ),
        (Changed<Actions>, Without<Corpse>),
    >,
) {
    use std::f32::consts;

    let dt = time.delta_seconds();

    let mut want_move_units = Vec::new();

    for (entity, mut unit, mut skills, skill_slots, mut transform, _, mut actions, account) in
        &mut unit_q
    {
        // All of the following action handlers are in a strict order

        // First react to any knockback events, this blocks all other actions
        // TODO this needs to be handled in the same manner as movement
        if let Some(action) = &mut actions.knockback {
            let ActionData::Knockback(knockback) = &action.data else {
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
            let ActionData::Attack(attack) = &mut action.data else {
                panic!("expected attack data");
            };

            match &mut action.state {
                State::Pending => {
                    let distance =
                        (attack.user.distance(attack.skill_target.target) * 100.).round() as u32;
                    let skill_id = skill_slots.slots[0].skill_id.unwrap();
                    assert_eq!(skill_id, attack.skill_id);

                    let Some(skill_info) = metadata.rpg.skill.skills.get(&skill_id) else {
                        panic!("skill metadata not found");
                    };

                    match unit.can_use_skill(
                        &mut skills.0,
                        &metadata.rpg,
                        attack.skill_id,
                        distance,
                    ) {
                        SkillUseResult::Blocked
                        | SkillUseResult::OutOfRange
                        | SkillUseResult::InsufficientResources => {
                            action.state = State::Completed;
                            // debug!("skill use blocked {:?}", unit.skills);
                            continue;
                        }
                        SkillUseResult::Ok => {}
                        SkillUseResult::Error => {
                            panic!("Skill use error");
                        }
                    }

                    let message = bincode::serialize(&ServerMessage::SCUnitAttack(SCUnitAttack {
                        uid: unit.uid,
                        skill_id,
                    }))
                    .unwrap();
                    net_params
                        .server
                        .broadcast_message(ServerChannel::Message, message);

                    let duration = skill_info.use_duration_secs
                        * unit.stats.vitals.stats["Cooldown"].value.f32();

                    action.timer = Some(Timer::from_seconds(duration, TimerMode::Once));
                    action.state = State::Timer;
                }
                State::Active => {
                    let distance =
                        (attack.user.distance(attack.skill_target.target) * 100.).round() as u32;
                    let skill_use_result =
                        unit.use_skill(&mut skills, &metadata.rpg, attack.skill_id, distance);
                    match skill_use_result {
                        SkillUseResult::Ok => {}
                        _ => panic!("This should never happen. {skill_use_result:?}"),
                    }

                    let Some(skill) = skills.iter().find(|s| s.id == attack.skill_id) else {
                        panic!("skill missing");
                    };
                    let Some(skill_info) = metadata.rpg.skill.skills.get(&attack.skill_id) else {
                        panic!("skill metadata not found");
                    };

                    /* TODO track stats per player in server
                    if unit.kind == UnitKind::Hero {
                        state.session_stats.attacks += 1;
                    } else {
                        state.session_stats.villain_attacks += 1;
                    }*/

                    let instance_uid = game_state.next_instance_uid.get();

                    let (skill_aabb, skill_transform, skill_use, timer) = skill::prepare_skill(
                        &attack,
                        &mut aabbs,
                        skill_info,
                        skill,
                        &unit,
                        &transform,
                        instance_uid,
                    );

                    game_state.next_instance_uid.next();

                    // debug!("spawning skill");
                    skill::spawn_instance(
                        &mut commands,
                        skill_aabb,
                        skill_transform,
                        skill_use,
                        entity,
                        unit.kind,
                        timer,
                    );

                    let message = bincode::serialize(&ServerMessage::SCSpawnSkill(SCSpawnSkill {
                        instance_uid,
                        id: skill.id,
                        owner_uid: unit.uid,
                        target: attack.skill_target.clone(),
                    }))
                    .unwrap();

                    net_params
                        .server
                        .broadcast_message(ClientChannel::Message, message);

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

                let message =
                    bincode::serialize(&ServerMessage::SCRotPlayer(SCRotPlayer(*direction)))
                        .unwrap();
                net_params
                    .server
                    .send_message(client.client_id, ServerChannel::Message, message);

                let message = bincode::serialize(&ServerMessage::SCRotUnit(SCRotUnit {
                    uid: unit.uid,
                    direction: *direction,
                }))
                .unwrap();
                net_params.server.broadcast_message_except(
                    client.client_id,
                    ServerChannel::Message,
                    message,
                );
            } else {
                let message = bincode::serialize(&ServerMessage::SCRotUnit(SCRotUnit {
                    uid: unit.uid,
                    direction: *direction,
                }))
                .unwrap();
                net_params
                    .server
                    .broadcast_message(ServerChannel::Message, message);
            }

            action.state = State::Completed;
        }

        if let Some(action) = &actions.movement {
            if action.state == State::Pending
                || action.state == State::Active
                || action.state == State::Finalize
            {
                //info!("adding unit to move list");
                want_move_units.push(entity);
            }
        }
    }

    moving_units.0 = want_move_units;
}

// TODO cache info about the movement in `MoveUnits`
pub(crate) fn try_move_units(
    mut moving_units: ResMut<MovingUnits>,
    time: Res<Time>,
    mut move_q: Query<(Entity, &Unit, &Transform, &AabbComponent, &mut Actions), Without<Corpse>>,
) {
    let dt = time.delta_seconds();

    for entity in moving_units.0.iter_mut() {
        let mut combinations = move_q.iter_combinations_mut();
        'combos: loop {
            let Some(
                [(l_entity, l_u, l_t, l_aabb, mut l_a), (r_entity, r_u, r_t, r_aabb, mut r_a)],
            ) = combinations.fetch_next()
            else {
                break 'combos;
            };

            if (l_entity != *entity && r_entity != *entity) || *entity == Entity::PLACEHOLDER {
                continue;
            }

            let is_l = *entity == l_entity;

            if is_l {
                let action = l_a.movement.as_mut().unwrap();
                let movespeed = l_u.get_effective_movement_speed() as f32 / 100. * dt;
                let wanted_translation = l_t.translation + *l_t.forward() * movespeed;

                if !can_move(
                    (&wanted_translation, &l_t.rotation, &l_aabb),
                    (&r_t.translation, &r_t.rotation, &r_aabb),
                ) {
                    *entity = Entity::PLACEHOLDER;
                    action.state = State::Completed;
                    break 'combos;
                }
            } else {
                let action = r_a.movement.as_mut().unwrap();
                let movespeed = r_u.get_effective_movement_speed() as f32 / 100. * dt;
                let wanted_translation = r_t.translation + *r_t.forward() * movespeed;

                if !can_move(
                    (&wanted_translation, &r_t.rotation, &r_aabb),
                    (&l_t.translation, &l_t.rotation, &l_aabb),
                ) {
                    *entity = Entity::PLACEHOLDER;
                    action.state = State::Completed;
                    break 'combos;
                }
            }
        }
        if *entity == Entity::PLACEHOLDER {
            // FIXME changing the entity in-situ precludes handling this correctly
            // TODO The action has been denied, if not already in progress, send a message to connected clients
        }
    }

    moving_units.0.retain(|u| *u != Entity::PLACEHOLDER);
}

pub(crate) fn move_units(
    mut net_params: NetworkParamsRW,
    mut moving_units: ResMut<MovingUnits>,
    time: Res<Time>,
    mut move_q: Query<
        (
            &mut Unit,
            &mut Transform,
            &mut Actions,
            Option<&mut AccountInstance>,
        ),
        Without<Corpse>,
    >,
) {
    let dt = time.delta_seconds();

    for entity in moving_units.0.iter() {
        let (mut m_unit, mut m_t, mut m_action, m_acc) = move_q.get_mut(*entity).unwrap();
        let action = m_action.movement.as_mut().unwrap();

        let movespeed = m_unit.get_effective_movement_speed() as f32 / 100. * dt;
        if m_unit.can_run() {
            m_unit.stats.consume_stamina(dt);
        }
        let wanted_translation = m_t.translation + *m_t.forward() * movespeed;
        m_t.translation = wanted_translation;

        if m_unit.kind == UnitKind::Hero {
            let client = net_params
                .context
                .get_client_from_account_id(m_acc.as_ref().unwrap().0.info.id)
                .unwrap();

            if action.state == State::Finalize {
                let message = bincode::serialize(&ServerMessage::SCMovePlayerEnd(SCMovePlayerEnd(
                    m_t.translation,
                )))
                .unwrap();
                net_params
                    .server
                    .send_message(client.client_id, ServerChannel::Message, message);

                let message = bincode::serialize(&ServerMessage::SCMoveUnitEnd(SCMoveUnitEnd {
                    uid: m_unit.uid,
                    position: m_t.translation,
                }))
                .unwrap();

                net_params.server.broadcast_message_except(
                    client.client_id,
                    ServerChannel::Message,
                    message,
                );
            } else {
                let message =
                    bincode::serialize(&ServerMessage::SCMovePlayer(SCMovePlayer(m_t.translation)))
                        .unwrap();
                net_params
                    .server
                    .send_message(client.client_id, ServerChannel::Message, message);

                let message = bincode::serialize(&ServerMessage::SCMoveUnit(SCMoveUnit {
                    uid: m_unit.uid,
                    position: m_t.translation,
                }))
                .unwrap();
                net_params.server.broadcast_message_except(
                    client.client_id,
                    ServerChannel::Message,
                    message,
                );
            }
        } else {
            if action.state == State::Finalize {
                let message = bincode::serialize(&ServerMessage::SCMoveUnitEnd(SCMoveUnitEnd {
                    uid: m_unit.uid,
                    position: m_t.translation,
                }))
                .unwrap();
                net_params
                    .server
                    .broadcast_message(ServerChannel::Message, message);
            } else {
                let message = bincode::serialize(&ServerMessage::SCMoveUnit(SCMoveUnit {
                    uid: m_unit.uid,
                    position: m_t.translation,
                }))
                .unwrap();
                net_params
                    .server
                    .broadcast_message(ServerChannel::Message, message);
            }
        }

        if action.state == State::Pending {
            action.state = State::Active;
        } else if action.state == State::Finalize {
            action.state = State::Completed;
        }
    }

    moving_units.0.clear();
}
