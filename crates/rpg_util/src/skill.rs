use bevy::{
    ecs::{bundle::Bundle, component::Component, entity::Entity, event::Event},
    math::Vec3,
    prelude::{Deref, DerefMut},
    time::Timer,
    transform::components::Transform,
};

use rpg_core::{
    damage::Damage,
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
    pub damage: Damage,
    pub instance: SkillInstance,
    pub effects: Vec<EffectInstance>,
    pub tickable: Option<Tickable>,
}

impl SkillUse {
    pub fn new(
        owner: Uid,
        id: SkillId,
        damage: Damage,
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
            unit_transform.translation + data.offset * unit_transform.forward(),
            unit_transform.translation + data.offset * unit_transform.forward(),
        ),
        Origin::Remote(data) => (unit_transform.translation + data.offset, target),
        Origin::Locked(data) => (
            unit_transform.translation + data.offset,
            unit_transform.translation + data.offset,
        ),
    }
}
