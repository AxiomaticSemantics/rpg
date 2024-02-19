use super::plugin::GameSessionCleanup;

use crate::{account::AccountInstance, assets::MetadataResources, net::server::NetworkParamsRW};

use rpg_core::{
    metadata::Metadata,
    skill::{SkillSlot, SkillSlotId},
    uid::NextUid,
    unit::{UnitInfo, UnitKind, VillainInfo},
    villain::VillainId,
};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::{Action, ActionData, Actions, AttackData, State},
    skill::{get_skill_origin, SkillSlots, Skills},
    unit::{Corpse, Hero, Unit, UnitBundle, Villain, VillainBundle},
};

use util::{cleanup::CleanupStrategy, math::AabbComponent, random::SharedRng};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
    math::{bounding::Aabb3d, Vec3},
    prelude::{Deref, DerefMut},
    time::{Time, Timer, TimerMode},
    transform::{components::Transform, TransformBundle},
};

use std::f32::consts::TAU;

#[derive(Component, Default, Debug, Deref, DerefMut)]
pub(crate) struct ThinkTimer(pub(crate) Timer);

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct RoamInfo {
    pub(crate) target: Vec3,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct TargetInfo(pub(crate) Option<Entity>);

#[derive(Default, Debug, PartialEq, Clone)]
pub(crate) enum GoalInfo {
    #[default]
    Inactive,
    Roam(RoamInfo),
    Target(TargetInfo),
}

impl GoalInfo {
    pub(crate) fn is_target(&self) -> bool {
        matches!(self, Self::Target(_))
    }

    pub(crate) fn is_roaming(&self) -> bool {
        matches!(self, Self::Roam(_))
    }
}

#[derive(Default, Debug)]
pub(crate) struct Goal {
    info: GoalInfo,
}

impl Goal {
    pub(crate) fn has_goal(&self) -> bool {
        self.info != GoalInfo::Inactive
    }
}

#[derive(Default, Debug, Component)]
pub(crate) struct VillainController {
    pub(crate) goal: Goal,
    pub(crate) spawned_on: Vec<Entity>,
    pub(crate) origin: Vec3,
}

impl VillainController {
    fn new(origin: Vec3) -> Self {
        Self {
            origin,
            spawned_on: vec![],
            goal: Goal::default(),
        }
    }

    fn think(&self, actions: &mut Actions, position: &Vec3, target: &Vec3) {
        //
    }
}

pub(crate) fn spawn(
    commands: &mut Commands,
    next_uid: &mut NextUid,
    origin: &Vec3,
    metadata: &Metadata,
    aabb: Aabb3d,
    villain_id: VillainId,
) {
    let villain_meta = &metadata.unit.villains[&villain_id];
    let mut unit = rpg_core::unit::Unit::new(
        next_uid.get(),
        villain_meta.class,
        UnitKind::Villain,
        UnitInfo::Villain(VillainInfo { id: villain_id }),
        1,
        villain_meta.name.clone(),
        metadata,
    );
    next_uid.next();

    let mut skills = vec![];
    unit.add_default_skills(&mut skills, metadata);

    let mut slots = vec![];
    slots.push(SkillSlot::new(SkillSlotId(0), Some(skills[0].id)));
    let skill_slots = SkillSlots::new(slots);

    let transform = Transform::from_translation(*origin);

    // spawn
    commands.spawn((
        CleanupStrategy::DespawnRecursive,
        GameSessionCleanup,
        AabbComponent(aabb),
        VillainBundle {
            villain: Villain,
            unit: UnitBundle::new(Unit(unit), Skills(skills), skill_slots),
        },
        ThinkTimer(Timer::from_seconds(4.0, TimerMode::Repeating)),
        VillainController::new(transform.translation),
        TransformBundle::from(transform),
    ));
}

// TODO optimize
pub(crate) fn remote_spawn(
    mut net_params: NetworkParamsRW,
    hero_q: Query<(Entity, &Transform, &AccountInstance), (With<Hero>, Without<Corpse>)>,
    mut villain_q: Query<
        (
            &Transform,
            &Unit,
            &Skills,
            &SkillSlots,
            &mut VillainController,
        ),
        (With<Villain>, Without<Corpse>),
    >,
) {
    for (transform, unit, skills, skill_slots, mut controller) in &mut villain_q {
        for (hero_entity, hero_transform, account) in &hero_q {
            let distance = transform.translation.distance(hero_transform.translation);
            if distance > 16.0 {
                continue;
            }

            // Check that the hero is not already spawned on this client
            if controller.spawned_on.contains(&hero_entity) {
                continue;
            }

            let villain_info = unit.info.villain().clone();

            // info!("spawning nearby monster on client {villain_info:?}");
            controller.spawned_on.push(hero_entity);

            let client_id = net_params
                .context
                .get_client_from_account_id(account.0.info.id)
                .unwrap()
                .client_id;

            let message = bincode::serialize(&ServerMessage::SCSpawnVillain(SCSpawnVillain {
                position: transform.translation,
                direction: *transform.forward(),
                info: villain_info,
                level: unit.level,
                uid: unit.uid,
                skills: skills.0.clone(),
                skill_slots: skill_slots.slots.clone(),
            }))
            .unwrap();

            net_params
                .server
                .send_message(client_id, ServerChannel::Message, message);
        }
    }
}

pub(crate) fn find_target(
    metadata: Res<MetadataResources>,
    hero_q: Query<(Entity, &Transform), (With<Hero>, Without<Villain>, Without<Corpse>)>,
    mut villain_q: Query<
        (&Transform, &Unit, &mut VillainController, &mut Actions),
        (With<Villain>, Without<Corpse>),
    >,
) {
    for (transform, unit, mut villain, actions) in &mut villain_q {
        let GoalInfo::Target(info) = &mut villain.goal.info else {
            continue;
        };

        if actions.attack.is_some() || actions.knockback.is_some() {
            continue;
        }

        let max_distance =
            (metadata.rpg.unit.villains[&unit.info.villain().id].max_vision * 100.).round() as u32;

        // Check if the current target is out of range and if so invalidate it
        if let Ok((_, hero_transform)) = hero_q.get(info.0.unwrap()) {
            let distance =
                (transform.translation.distance(hero_transform.translation) * 100.).round() as u32;
            if distance > max_distance {
                // The targeted entity is out of range, unset the target
                info.0 = None;
            } else {
                // The targeted entity is in range, keep the current target
                continue;
            }
        } else {
            info.0 = None;
        }

        if info.0.is_some() {
            continue;
        }

        // There is no current target, attempt to find one
        let mut nearest = None::<Entity>;
        let mut nearest_distance = max_distance;

        for (hero_entity, hero_transform) in &hero_q {
            // TODO check villain and hero `ZoneId` to avoid needless computations
            let distance =
                (transform.translation.distance(hero_transform.translation) * 100.).round() as u32;
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest = Some(hero_entity);
            }
        }

        if nearest.is_some() {
            info.0 = nearest;
        }
    }
}

pub(crate) fn villain_think(
    time: Res<Time>,
    mut rng: ResMut<SharedRng>,
    metadata: Res<MetadataResources>,
    hero_q: Query<&Transform, (With<Hero>, Without<Villain>, Without<Corpse>)>,
    mut villain_q: Query<
        (
            &Transform,
            &Unit,
            &Skills,
            &SkillSlots,
            &mut VillainController,
            &mut Actions,
            &mut ThinkTimer,
        ),
        (With<Villain>, Without<Corpse>),
    >,
) {
    for (transform, unit, skills, skill_slots, mut villain, mut actions, mut think_timer) in
        &mut villain_q
    {
        think_timer.tick(time.delta());

        let villain_id = unit.info.villain().id;
        let villain_meta = &metadata.rpg.unit.villains[&villain_id];

        if let GoalInfo::Inactive = &villain.goal.info {
            assert!(actions.is_inactive());

            if think_timer.finished() {
                // debug!("selecting roam target");
                let target = if villain.origin.abs_diff_eq(transform.translation, 0.5) {
                    let (s_x, s_y) = (0.5 - rng.f32(), 0.5 - rng.f32());
                    let s_x = if s_x > 0. {
                        4. + s_x * 8.
                    } else {
                        -4. + s_x * 8.
                    };
                    let s_y = if s_y > 0. {
                        4. + s_y * 8.
                    } else {
                        -4. + s_y * 8.
                    };

                    transform.translation + Vec3::new(s_x, 0.0, s_y)
                } else {
                    villain.origin
                };

                villain.goal.info = GoalInfo::Roam(RoamInfo { target });
            } else {
                continue;
            }
        }

        if let GoalInfo::Roam(info) = &mut villain.goal.info {
            if info.target.abs_diff_eq(transform.translation, 0.01) {
                // debug!("goal reached");

                actions.movement.as_mut().unwrap().state = State::Finalize;
                villain.goal.info = GoalInfo::Inactive;
                think_timer.reset();
            } else {
                // TODO add a time limit, ensure progression is made
                actions.request(Action::new(ActionData::LookPoint(info.target), None, true));
                if actions.movement.is_none() {
                    actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
                }
            }
        } else if let GoalInfo::Target(info) = &mut villain.goal.info {
            // Hero liveness should be checked in find_target
            let Some(target) = &info.0 else {
                panic!("a valid target is expected");
            };

            let Ok(hero_transform) = &hero_q.get(*target) else {
                villain.goal.info = GoalInfo::Inactive;
                continue;
            };
            let distance =
                (transform.translation.distance(hero_transform.translation) * 100.).round() as u32;
            if distance > (villain_meta.max_vision * 100.).floor() as u32 {
                info.0 = None;
                villain.goal.info = GoalInfo::Inactive;
                continue;
            }

            let target_dir =
                (hero_transform.translation - transform.translation).normalize_or_zero();
            let rot_diff = transform.forward().dot(target_dir) - 1.;
            let want_look = rot_diff.abs() > 0.01;
            if want_look {
                actions.request(Action::new(
                    ActionData::LookPoint(hero_transform.translation),
                    None,
                    true,
                ));
            }
            let skill_id = skill_slots.slots[0].skill_id.unwrap();
            let skill_info = &metadata.rpg.skill.skills[&skill_id];

            let wanted_range = (skill_info.use_range as f32 * 0.5) as u32;
            let wanted_range = wanted_range.clamp(150, wanted_range.max(150));
            let in_range = skill_info.use_range > 0 && distance < wanted_range;
            if !in_range {
                if actions.movement.is_none() {
                    // info!("move request");
                    actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
                }
                continue;
            }
            if let Some(action) = &mut actions.movement {
                if action.state == State::Active {
                    action.state = State::Finalize;
                }
                continue;
            }

            assert!(actions.movement.is_none());
            if actions.attack.is_none() {
                // debug!("distance {distance} use range {}", skill_info.use_range);

                let skill_target = get_skill_origin(
                    &metadata.rpg,
                    transform,
                    hero_transform.translation,
                    skill_id,
                );

                actions.request(Action::new(
                    ActionData::Attack(AttackData {
                        skill_id,
                        user: transform.translation,
                        skill_target,
                    }),
                    None,
                    true,
                ));
            }
            villain.goal.info = GoalInfo::Inactive;
        }

        // TODO
        // - ensure unit is in the same zone
        // - ensure the zone is a combat zone
    }
}
