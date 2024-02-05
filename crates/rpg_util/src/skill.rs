use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::Event,
        system::{Commands, Query, Res},
    },
    hierarchy::DespawnRecursiveExt,
    math::{Quat, Vec3},
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
    transform::components::Transform,
};

use rpg_core::{
    damage::DamageDescriptor,
    metadata::Metadata,
    skill::{effect::*, Origin, SkillId, SkillInstance},
    uid::Uid,
    unit::UnitKind,
};

#[derive(Event)]
pub struct SkillContactEvent {
    pub entity: Entity,
    pub owner: Entity,
    pub owner_kind: UnitKind,
    pub defender: Entity,
}

#[derive(Debug)]
pub struct Tickable {
    pub timer: Timer,
    pub ticker: Timer,
    pub can_damage: bool,
}

#[derive(Debug, Component)]
pub enum SkillTimer {
    Duration(Timer),
    Tickable(Tickable),
}

#[derive(Debug, Clone)]
pub struct InvulnerabilityTimer {
    pub entity: Entity,
    pub timer: Timer,
}

#[derive(Default, Debug, Clone, Component, Deref, DerefMut)]
pub struct Invulnerability(pub Vec<InvulnerabilityTimer>);

#[derive(Debug, Component)]
pub struct SkillUse {
    pub owner: Uid,
    pub id: SkillId,
    pub damage: DamageDescriptor,
    pub instance: SkillInstance,
    pub effects: Vec<EffectInstance>,
}

impl SkillUse {
    pub fn new(
        owner: Uid,
        id: SkillId,
        damage: DamageDescriptor,
        instance: SkillInstance,
        effects: Vec<EffectInstance>,
    ) -> Self {
        Self {
            owner,
            id,
            damage,
            instance,
            effects,
        }
    }
}

#[derive(Bundle)]
pub struct SkillUseBundle {
    pub skill: SkillUse,
}

impl SkillUseBundle {
    pub fn new(skill: SkillUse) -> Self {
        Self { skill }
    }
}

pub fn clean_skills(
    mut commands: Commands,
    mut skill_q: Query<(Entity, &Transform, &SkillUse, Option<&SkillTimer>)>,
) {
    for (entity, transform, skill_use, timer) in &mut skill_q {
        if let Some(timer) = timer {
            if let SkillTimer::Duration(timer) = timer {
                if timer.just_finished() {
                    commands.entity(entity).despawn_recursive();
                    continue;
                }
            } else if let SkillTimer::Tickable(tickable) = timer {
                if tickable.timer.just_finished() {
                    commands.entity(entity).despawn_recursive();
                    continue;
                }
            }
        }

        let despawn = match &skill_use.instance {
            SkillInstance::Projectile(info) => {
                if info.info.aerial.is_some() {
                    transform.translation.y < info.info.size as f32 / 100.
                } else {
                    false
                }
            }
            SkillInstance::Direct(info) => {
                // info!("direct skill: {info:?}");
                info.frame >= info.info.frames
            }
            SkillInstance::Area(info) => {
                // info!("area skill: {info:?}");
                false
            }
        };

        if despawn {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn get_skill_origin(
    metadata: &Metadata,
    unit_transform: &Transform,
    target: Vec3,
    skill_id: SkillId,
) -> (Vec3, Vec3) {
    let skill_meta = &metadata.skill.skills[&skill_id];

    match &skill_meta.origin {
        Origin::Direct(data) => (
            unit_transform.translation + data.offset * *unit_transform.forward(),
            unit_transform.translation + data.offset * *unit_transform.forward(),
        ),
        Origin::Remote(data) => (unit_transform.translation + data.offset, target),
        Origin::Locked(data) => (
            unit_transform.translation + data.offset,
            unit_transform.translation + data.offset,
        ),
    }
}

pub fn update_skill(
    time: Res<Time>,
    mut skill_q: Query<(&mut Transform, &mut SkillUse, Option<&mut SkillTimer>)>,
) {
    let dt = time.delta_seconds();
    for (mut transform, mut skill_use, timer) in &mut skill_q {
        if let Some(mut timer) = timer {
            if let SkillTimer::Duration(ref mut timer) = &mut *timer {
                timer.tick(time.delta());
            } else if let SkillTimer::Tickable(ref mut tickable) = &mut *timer {
                tickable.timer.tick(time.delta());
                tickable.ticker.tick(time.delta());
                if tickable.ticker.just_finished() {
                    tickable.can_damage = true;
                    tickable.ticker.reset();
                }
            }
        }

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
            }
        }
    }
}
