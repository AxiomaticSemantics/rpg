use bevy::{
    ecs::{bundle::Bundle, component::Component, entity::Entity},
    log::info,
    math::Vec3,
    prelude::{Deref, DerefMut},
    time::Timer,
    transform::components::Transform,
};

use rpg_core::{
    damage::Damage,
    metadata::Metadata,
    skill::{effect::*, Origin, SkillId, SkillInstance},
    unit::UnitKind,
};

#[derive(Default, Debug, Component, Deref, DerefMut)]
pub struct SkillTimer(pub Option<Timer>);

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
    pub owner: Entity,
    pub owner_kind: UnitKind,
    pub id: SkillId,
    pub damage: Damage,
    pub instance: SkillInstance,
    pub effects: Vec<EffectInstance>,
    pub tickable: Option<Tickable>,
}

impl SkillUse {
    pub fn new(
        owner: Entity,
        owner_kind: UnitKind,
        id: SkillId,
        damage: Damage,
        instance: SkillInstance,
        effects: Vec<EffectInstance>,
        tickable: Option<Tickable>,
    ) -> Self {
        Self {
            owner,
            owner_kind,
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
    pub timer: SkillTimer,
}

impl SkillUseBundle {
    pub fn new(skill: SkillUse, timer: SkillTimer) -> Self {
        Self { skill, timer }
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
