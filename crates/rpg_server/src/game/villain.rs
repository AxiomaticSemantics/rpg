use crate::assets::MetadataResources;

use rpg_core::{metadata::Metadata, uid::NextUid};
use rpg_util::{
    actions::{Action, ActionData, Actions, AttackData},
    skill::get_skill_origin,
    unit::{Corpse, Hero, Unit, UnitBundle, Villain, VillainBundle},
};
use util::random::SharedRng;

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    math::Vec3,
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
    transform::components::Transform,
};

#[derive(Component, Default, Debug, Deref, DerefMut)]
pub(crate) struct ThinkTimer(pub(crate) Timer);

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub(crate) enum VillainState {
    #[default]
    Idle,
    Roaming,
    Tracking,
}

#[derive(Component)]
pub(crate) struct VillainController {
    pub(crate) state: VillainState,
}

impl VillainController {
    fn think(&self, actions: &mut Actions, position: &Vec3, target: &Vec3) {
        //
    }
}

pub(crate) fn spawn_villain(
    commands: &mut Commands,
    next_uid: &mut NextUid,
    origin: &Vec3,
    metadata: &Metadata,
    rng: &mut SharedRng,
) {
    let mut unit = rpg_core::unit::generation::generate(&mut rng.0, metadata, next_uid, 1);

    unit.add_default_skills(metadata);

    let dir_roll = std::f32::consts::TAU * (0.5 - rng.f32());
    let distance = 14_f32;

    let mut transform = Transform::from_translation(*origin);
    transform.rotate_y(dir_roll);
    transform.translation += transform.forward() * distance;

    let unit_info = unit.info.villain();
    let villain_info = &metadata.unit.villains[&unit_info.id];

    // spawn
    commands.spawn((VillainBundle {
        villain: Villain,
        unit: UnitBundle::new(Unit(unit)),
    },));

    /*
    actor::spawn_actor(
        commands,
        metadata,
        rnderables,
        unit,
        None,
        None,
        Some(transform),
    );
    */
}

pub fn villain_think(
    time: Res<Time>,
    mut rng: ResMut<SharedRng>,
    metadata: Res<MetadataResources>,
    hero_q: Query<(&Transform, &Unit), (With<Hero>, Without<Villain>, Without<Corpse>)>,
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
    let (hero_transform, hero_unit) = hero_q.single();

    for (transform, unit, mut villain, mut actions, mut think_timer) in &mut villain_q {
        think_timer.tick(time.delta());

        if !hero_unit.is_alive() {
            villain.state = VillainState::Roaming;
        }

        let distance =
            (transform.translation.distance(hero_transform.translation) * 100.).round() as u32;

        let max_distance =
            (metadata.0.unit.villains[&unit.info.villain().id].max_vision * 100.).round() as u32;
        if distance > max_distance {
            villain.state = VillainState::Idle;
            continue;
        }

        villain.think(
            &mut actions,
            &transform.translation,
            &hero_transform.translation,
        );

        let target_dir = (hero_transform.translation - transform.translation).normalize_or_zero();
        let rot_diff = transform.forward().dot(target_dir) - 1.;
        let want_look = rot_diff.abs() > 0.001;
        if want_look {
            actions.request(Action::new(
                ActionData::LookPoint(hero_transform.translation),
                None,
                true,
            ));
        }

        if actions.movement.is_none() && villain.state != VillainState::Idle {
            villain.state = VillainState::Idle;
            actions.set(Action::new(ActionData::MoveEnd, None, false));
        }

        let skill_id = unit.active_skills.primary.skill.unwrap();
        let skill_info = &metadata.0.skill.skills[&skill_id];

        let wanted_range = (skill_info.use_range as f32 * 0.5) as u32;
        let wanted_range = wanted_range.clamp(150, wanted_range.max(150));
        let in_range = skill_info.use_range > 0 && distance < wanted_range;
        if rot_diff.abs() < 0.1 {
            if !in_range {
                actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
                villain.state = VillainState::Tracking;
                continue;
            }

            /*if actions.movement.is_none() && villain.state != VillainState::Idle {
                villain.state = VillainState::Idle;
                actions.set(Action::new(ActionData::MoveEnd, None, false));
            }*/

            if think_timer.finished()
                && actions.attack.is_none()
                && villain.state == VillainState::Tracking
            {
                //println!("distance {distance} use range {}", skill_info.use_range);

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
}
