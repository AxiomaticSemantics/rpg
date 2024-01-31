use crate::{account::AccountInstance, assets::MetadataResources, net::server::NetworkParamsRW};

use rpg_core::{
    metadata::Metadata,
    uid::NextUid,
    unit::{UnitInfo, UnitKind, VillainInfo},
    villain::VillainId,
};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::{Action, ActionData, Actions, AttackData, State},
    skill::get_skill_origin,
    unit::{Corpse, Hero, Unit, UnitBundle, Villain, VillainBundle},
};

use util::{
    math::{Aabb, AabbComponent},
    random::SharedRng,
};

use lightyear::shared::NetworkTarget;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
    math::Vec3,
    prelude::{Deref, DerefMut},
    time::{Time, Timer, TimerMode},
    transform::{components::Transform, TransformBundle},
};

#[derive(Component, Default, Debug, Deref, DerefMut)]
pub(crate) struct ThinkTimer(pub(crate) Timer);

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub(crate) enum VillainState {
    #[default]
    Deactivated,
    Idle,
    Roaming,
    Tracking,
}

#[derive(Debug, Component)]
pub(crate) struct VillainController {
    pub(crate) state: VillainState,
    pub(crate) spawned_on: Vec<Entity>,
    target: Entity,
}

impl Default for VillainController {
    fn default() -> Self {
        Self {
            state: VillainState::default(),
            spawned_on: vec![],
            target: Entity::PLACEHOLDER,
        }
    }
}

impl VillainController {
    fn new(state: VillainState) -> Self {
        Self {
            state,
            spawned_on: vec![],
            target: Entity::PLACEHOLDER,
        }
    }

    fn has_target(&self) -> bool {
        self.target != Entity::PLACEHOLDER
    }

    fn think(&self, actions: &mut Actions, position: &Vec3, target: &Vec3) {
        if !self.is_activated() {
            return;
        }
    }

    fn is_activated(&self) -> bool {
        self.state != VillainState::Deactivated
    }

    fn activate(&mut self) -> bool {
        if !self.is_activated() {
            self.state = VillainState::Idle;

            true
        } else {
            false
        }
    }
}

pub(crate) fn spawn(
    commands: &mut Commands,
    next_uid: &mut NextUid,
    origin: &Vec3,
    metadata: &Metadata,
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
        None,
        metadata,
    );
    next_uid.next();

    unit.add_default_skills(metadata);

    let aabb = AabbComponent(Aabb::from_min_max(
        Vec3::new(-0.3, 0.0, -0.2),
        Vec3::new(0.3, 1.2, 0.2),
    ));

    let transform = Transform::from_translation(*origin);
    let unit_info = unit.info.villain();

    info!(
        "spawning villain uid {:?} {unit_info:?} at {:?}",
        unit.uid, transform.translation
    );

    // spawn
    commands.spawn((
        aabb,
        VillainBundle {
            villain: Villain,
            unit: UnitBundle::new(Unit(unit)),
        },
        ThinkTimer(Timer::from_seconds(4.0, TimerMode::Repeating)),
        VillainController::new(VillainState::Idle),
        TransformBundle::from(transform),
    ));
}

// TODO optimize
pub(crate) fn remote_spawn(
    mut net_params: NetworkParamsRW,
    hero_q: Query<(Entity, &Transform, &AccountInstance), (With<Hero>, Without<Corpse>)>,
    mut villain_q: Query<
        (&Transform, &Unit, &mut VillainController),
        (With<Villain>, Without<Corpse>),
    >,
) {
    for (transform, unit, mut controller) in &mut villain_q {
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

            info!("spawning monster on client {villain_info:?}");
            controller.spawned_on.push(hero_entity);

            let client_id = net_params
                .context
                .get_client_from_account_id(account.0.info.id)
                .unwrap()
                .id;

            net_params.server.send_message_to_target::<Channel1, _>(
                SCSpawnVillain {
                    position: transform.translation,
                    direction: transform.forward(),
                    info: villain_info,
                    level: unit.level,
                    uid: unit.uid,
                },
                NetworkTarget::Only(vec![client_id]),
            );
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
        if !villain.is_activated() {
            info!("not activated");
            continue;
        }

        if actions.attack.is_some() || actions.knockback.is_some() {
            continue;
        }

        let max_distance =
            (metadata.0.unit.villains[&unit.info.villain().id].max_vision * 100.).round() as u32;
        if villain.has_target() {
            if let Ok((_, hero_transform)) = hero_q.get(villain.target) {
                let distance = (transform.translation.distance(hero_transform.translation) * 100.)
                    .round() as u32;
                if distance > max_distance {
                    // The targeted entity is out of range, unset the target
                    villain.target = Entity::PLACEHOLDER;
                } else {
                    // The targeted entity is in range, keep the current target
                    continue;
                }
            } else {
                villain.target = Entity::PLACEHOLDER;
            }
        }

        if villain.has_target() {
            // The villain already has a valid target
            continue;
        }

        let mut nearest = Entity::PLACEHOLDER;
        let mut nearest_distance = max_distance;

        for (hero_entity, hero_transform) in &hero_q {
            let distance =
                (transform.translation.distance(hero_transform.translation) * 100.).round() as u32;
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest = hero_entity;
            }
        }

        if nearest != Entity::PLACEHOLDER {
            villain.target = nearest;
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
            &mut VillainController,
            &mut Actions,
            &mut ThinkTimer,
        ),
        (With<Villain>, Without<Corpse>),
    >,
) {
    // TODO each villian needs advance in it's current target or select a new target at most once
    // per turn
    for (transform, unit, mut villain, mut actions, mut think_timer) in &mut villain_q {
        if !villain.is_activated() {
            continue;
        }

        think_timer.tick(time.delta());

        if !villain.has_target() {
            continue;
        }

        let Ok(hero_transform) = &hero_q.get(villain.target) else {
            villain.target = Entity::PLACEHOLDER;
            continue;
        };

        // TODO
        // - ensure unit is in the same zone
        // - ensure the zone is a combat zone

        villain.think(
            &mut actions,
            &transform.translation,
            &hero_transform.translation,
        );

        let distance =
            (transform.translation.distance(hero_transform.translation) * 100.).round() as u32;
        let villain_id = unit.info.villain().id;
        let villain_meta = &metadata.0.unit.villains[&villain_id];
        if distance > (villain_meta.max_vision * 100.).floor() as u32 {
            // this should be handled elsewhere
            villain.target = Entity::PLACEHOLDER;
            villain.state = VillainState::Idle;
            continue;
        }

        let target_dir = (hero_transform.translation - transform.translation).normalize_or_zero();
        let rot_diff = transform.forward().dot(target_dir) - 1.;
        let want_look = rot_diff.abs() > 0.01;
        if want_look {
            actions.request(Action::new(
                ActionData::LookPoint(hero_transform.translation),
                None,
                true,
            ));
        }

        let skill_id = unit.active_skills.primary.skill.unwrap();
        let skill_info = &metadata.0.skill.skills[&skill_id];

        let wanted_range = (skill_info.use_range as f32 * 0.5) as u32;
        let wanted_range = wanted_range.clamp(150, wanted_range.max(150));
        let in_range = skill_info.use_range > 0 && distance < wanted_range;
        if !in_range {
            if actions.movement.is_none() {
                info!("move request");
                actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
                villain.state = VillainState::Tracking;
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

        if think_timer.finished() && actions.attack.is_none() {
            // debug!("distance {distance} use range {}", skill_info.use_range);

            info!("skill request");
            let (origin, target) =
                get_skill_origin(&metadata.0, transform, hero_transform.translation, skill_id);

            actions.request(Action::new(
                ActionData::Attack(AttackData {
                    skill_id,
                    user: transform.translation,
                    origin,
                    target,
                }),
                None,
                true,
            ));
            think_timer.reset();
        }
    }
}
