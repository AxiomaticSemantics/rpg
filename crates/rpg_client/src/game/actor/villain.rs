#![allow(clippy::too_many_arguments)]

use crate::{
    game::{
        actions::{Action, ActionData, Actions, AttackData},
        actor::{
            self,
            unit::{CorpseTimer, Hero, Unit},
        },
        assets::RenderResources,
        metadata::MetadataResources,
        plugin::GameState,
        skill,
    },
    random::Random,
};

use rpg_core::uid::NextUid;

use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    math::Vec3,
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
    transform::components::Transform,
    utils::Duration,
};

#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct ThinkTimer(pub Timer);

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
pub enum VillainState {
    #[default]
    Idle,
    Roaming,
    Tracking,
}

#[derive(Debug, Default, Component)]
pub struct Villain {
    pub look_target: Option<Vec3>,
    pub state: VillainState,
}

impl Villain {
    pub fn think(&mut self, actions: &mut Actions, villain_position: &Vec3, hero_position: &Vec3) {
        match self.state {
            VillainState::Idle => {}
            VillainState::Roaming => {}
            VillainState::Tracking => {}
        }
    }
}

#[derive(Bundle)]
pub struct VillainBundle {
    pub villain: Villain,
    pub think_timer: ThinkTimer,
}

#[derive(Resource, Default)]
pub struct VillainSpawner {
    pub units: u32,
    pub frequency: f32,
    pub timer: Timer,
}

impl VillainSpawner {
    pub fn update_frequency(&mut self, frequency: f32) {
        if frequency != self.frequency {
            println!("updating frequency from {} to {frequency}", self.frequency);
            self.frequency = frequency;
            self.timer.set_duration(Duration::from_secs_f32(frequency));
            if self.timer.elapsed_secs() > frequency {
                self.timer.reset();
            }
        }
    }
}

pub fn spawner(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<VillainSpawner>,
    mut state: ResMut<GameState>,
    mut random: ResMut<Random>,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
    hero_q: Query<(&Unit, &Transform), With<Hero>>,
) {
    // Eventually resurrection or continuation of villain actions may be desired, keep this check
    // here for now
    let (hero_unit, hero_transform) = hero_q.single();
    if !hero_unit.is_alive() {
        return;
    }

    spawner.timer.tick(time.delta());
    if spawner.timer.finished() || state.session_stats.spawned == 0 {
        let frequency = spawner.frequency * 0.995;
        if state.session_stats.spawned != 0 {
            spawner.update_frequency(frequency);
        }

        spawn_villain(
            &mut commands,
            &mut state.next_uid,
            &hero_transform.translation,
            &metadata,
            &renderables,
            &mut random,
        );

        state.session_stats.spawned += 1;
    }
}

pub fn villain_think(
    time: Res<Time>,
    mut rng: ResMut<Random>,
    metadata: Res<MetadataResources>,
    hero_q: Query<(&Transform, &Unit), (Without<Villain>, With<Hero>)>,
    mut villain_q: Query<
        (
            &Transform,
            &Unit,
            &mut Villain,
            &mut Actions,
            &mut ThinkTimer,
        ),
        Without<CorpseTimer>,
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
            (metadata.rpg.unit.villains[&unit.info.villain().id].max_vision * 100.).round() as u32;
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
        let skill_info = &metadata.rpg.skill.skills[&skill_id];

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

                let (origin, target) = skill::get_skill_origin(
                    &metadata,
                    transform,
                    hero_transform.translation,
                    skill_id,
                );

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

fn spawn_villain(
    commands: &mut Commands,
    next_uid: &mut NextUid,
    origin: &Vec3,
    metadata: &MetadataResources,
    renderables: &RenderResources,
    rng: &mut Random,
) {
    let mut unit = rpg_core::unit::generation::generate(&mut rng.0, &metadata.rpg, next_uid, 1);

    unit.add_default_skills(&metadata.rpg);

    let dir_roll = std::f32::consts::TAU * (0.5 - rng.f32());
    let distance = 14_f32;

    let mut transform = Transform::from_translation(*origin);
    transform.rotate_y(dir_roll);
    transform.translation += transform.forward() * distance;

    actor::spawn_actor(
        commands,
        metadata,
        renderables,
        unit,
        None,
        None,
        Some(transform),
    );
}
