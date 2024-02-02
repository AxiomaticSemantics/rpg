use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        event::Event,
        system::{Query, Res},
    },
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
};

#[derive(Event)]
pub struct SkillContactEvent {
    pub entity: Entity,
    pub owner: Entity,
    pub defender: Entity,
}

#[derive(Default, Debug)]
pub struct Tickable {
    pub timer: Timer,
    pub can_damage: bool,
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
    pub tickable: Option<Tickable>,
}

impl SkillUse {
    pub fn new(
        owner: Uid,
        id: SkillId,
        damage: DamageDescriptor,
        instance: SkillInstance,
        effects: Vec<EffectInstance>,
        tickable: Option<Tickable>,
    ) -> Self {
        Self {
            owner,
            id,
            damage,
            instance,
            effects,
            tickable,
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

pub fn update_skill(time: Res<Time>, mut skill_q: Query<(&mut Transform, &mut SkillUse)>) {
    let dt = time.delta_seconds();
    for (mut transform, mut skill_use) in &mut skill_q {
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
                if let Some(tickable) = &mut skill_use.tickable {
                    tickable.timer.tick(time.delta());
                    if tickable.timer.just_finished() {
                        tickable.can_damage = true;
                        tickable.timer.reset();
                    }
                }
            }
        }
    }
}
