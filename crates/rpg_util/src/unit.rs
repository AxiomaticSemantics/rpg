use bevy::{
    ecs::{bundle::Bundle, component::Component},
    prelude::{Deref, DerefMut},
    time::Timer,
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
}

impl UnitBundle {
    pub fn new(unit: Unit) -> Self {
        Self { unit }
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
