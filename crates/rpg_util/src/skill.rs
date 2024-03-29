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
    skill::{effect::*, OriginKind, Skill, SkillId, SkillInstance, SkillSlot, SkillTarget},
    uid::{InstanceUid, Uid},
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

#[derive(Default, Debug, Component, Deref, DerefMut)]
pub struct Skills(pub Vec<Skill>);

#[derive(Debug, Component)]
pub struct SkillUse {
    pub instance_uid: InstanceUid,
    pub owner: Uid,
    pub id: SkillId,
    pub damage: DamageDescriptor,
    pub instance: SkillInstance,
    pub want_despawn: bool,
    pub effects: Vec<EffectInstance>,
}

impl SkillUse {
    pub fn new(
        instance_uid: InstanceUid,
        owner: Uid,
        id: SkillId,
        damage: DamageDescriptor,
        instance: SkillInstance,
        effects: Vec<EffectInstance>,
    ) -> Self {
        Self {
            instance_uid,
            owner,
            id,
            damage,
            instance,
            want_despawn: false,
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

/// Skill slots
#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct SkillSlots {
    pub slots: Vec<SkillSlot>,
}

impl SkillSlots {
    pub fn new(slots: Vec<SkillSlot>) -> Self {
        Self { slots }
    }
}

pub fn clean_skills(
    mut commands: Commands,
    mut skill_q: Query<(Entity, &Transform, &SkillUse, Option<&SkillTimer>)>,
) {
    for (entity, transform, skill_use, timer) in &mut skill_q {
        if skill_use.want_despawn {
            commands.entity(entity).despawn_recursive();
            continue;
        }

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
            SkillInstance::Area(_) => {
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
) -> SkillTarget {
    let skill_meta = &metadata.skill.skills[&skill_id];

    match skill_meta.origin_kind {
        OriginKind::Direct => SkillTarget {
            origin: unit_transform.translation + skill_meta.origin * *unit_transform.forward(),
            target,
        },
        OriginKind::Remote => SkillTarget {
            origin: unit_transform.translation + skill_meta.origin,
            target,
        },

        OriginKind::Locked => SkillTarget {
            origin: unit_transform.translation + skill_meta.origin,
            target,
        },
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
                        ((info.info.speed as f32 / 100.) * dt) % std::f32::consts::TAU,
                    );

                    transform.translate_around(orbit.origin, rotation);
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
