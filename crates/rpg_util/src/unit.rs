use crate::{
    actions::Actions,
    skill::{SkillSlots, Skills},
};

use bevy::{
    ecs::{bundle::Bundle, component::Component},
    prelude::{Deref, DerefMut},
};

#[derive(Component)]
pub struct Corpse;

#[derive(Component)]
pub struct Hero;

#[derive(Component)]
pub struct Villain;

#[derive(Debug, Component, Deref, DerefMut)]
pub struct Unit(pub rpg_core::unit::Unit);

#[derive(Bundle)]
pub struct UnitBundle {
    pub unit: Unit,
    pub skills: Skills,
    pub skill_slots: SkillSlots,
    pub actions: Actions,
}

impl UnitBundle {
    pub fn new(unit: Unit, skills: Skills, skill_slots: SkillSlots) -> Self {
        Self {
            unit,
            actions: Actions::default(),
            skills,
            skill_slots,
        }
    }
}

#[derive(Bundle)]
pub struct HeroBundle {
    pub hero: Hero,
    pub unit: UnitBundle,
}

#[derive(Bundle)]
pub struct VillainBundle {
    pub villain: Villain,
    pub unit: UnitBundle,
}
