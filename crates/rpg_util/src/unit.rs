use crate::{
    actions::UnitActions,
    skill::{SkillSlots, Skills},
};

use rpg_world::zone::ZoneId;

use bevy::{
    ecs::{bundle::Bundle, component::Component},
    prelude::{Deref, DerefMut},
};

#[derive(Default, Component)]
pub struct Waypoints(pub Vec<ZoneId>);

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
    pub actions: UnitActions,
}

impl UnitBundle {
    pub fn new(unit: Unit, skills: Skills, skill_slots: SkillSlots) -> Self {
        Self {
            unit,
            actions: UnitActions::default(),
            skills,
            skill_slots,
        }
    }
}

#[derive(Bundle)]
pub struct HeroBundle {
    pub hero: Hero,
    pub unit: UnitBundle,
    pub waypoints: Waypoints,
}

#[derive(Bundle)]
pub struct VillainBundle {
    pub villain: Villain,
    pub unit: UnitBundle,
}
