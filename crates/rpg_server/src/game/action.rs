use super::{
    plugin::{AabbResources, GameState},
    skill,
};
use crate::{account::AccountInstance, assets::MetadataResources, net::server::NetworkParamsRW};

use rpg_core::{skill::SkillUseResult, unit::UnitKind};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::{ActionData, Actions, State},
    unit::{Corpse, Unit},
};

use util::random::SharedRng;

use lightyear::shared::replication::components::NetworkTarget;

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
    mut game_state: ResMut<GameState>,
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
        info!("actions: {actions:?}");

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

        if let Some(action) = &mut actions.attack {
            let ActionData::Attack(attack) = action.data else {
                panic!("expected attack data");
            };

            match &mut action.state {
                State::Pending => {
                    let distance = (attack.user.distance(attack.target) * 100.).round() as u32;
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

                    let skill_id = unit.active_skills.primary.skill.unwrap();
                    let Some(skill_info) = metadata.0.skill.skills.get(&skill_id) else {
                        panic!("skill metadata not found");
                    };

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
                        entity, &attack, &time, &mut rng, &mut aabbs, skill_info, skill, &unit,
                        &transform,
                    );

                    skill::spawn_instance(&mut commands, skill_aabb, skill_transform, skill_use);

                    action.state = State::Completed;
                    action.timer = None;
                }
                _ => {}
            }
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

            action.state = State::Completed;
        }

        if let Some(action) = &mut actions.movement {
            let movespeed = unit.get_effective_movement_speed() as f32 / 100. * dt;
            if unit.can_run() {
                unit.stats.consume_stamina(dt);
            }

            let translation = transform.forward() * movespeed;
            transform.translation += translation;

            // send movement

            if unit.kind == UnitKind::Hero {
                let client = net_params
                    .context
                    .get_client_from_account_id(account.as_ref().unwrap().0.info.id)
                    .unwrap();

                net_params
                    .server
                    .send_message_to_target::<Channel1, SCMovePlayer>(
                        SCMovePlayer(transform.translation),
                        NetworkTarget::Only(vec![client.id]),
                    )
                    .unwrap();
            }

            action.state = State::Completed;
        }

        if let Some(action) = &mut actions.movement_end {
            // TODO send end movement to client

            action.state = State::Completed;
        }
    }
}
