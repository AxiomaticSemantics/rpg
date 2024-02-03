use crate::actions::Actions;

use util::math::{intersect_aabb, AabbComponent};

use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        query::{With, Without},
        system::Query,
    },
    prelude::{Deref, DerefMut},
    transform::components::Transform,
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
    pub actions: Actions,
}

impl UnitBundle {
    pub fn new(unit: Unit) -> Self {
        Self {
            unit,
            actions: Actions::default(),
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

// TODO FIXME this is just a buggy hack
pub fn collide_units(
    mut unit_q: Query<(&mut Transform, &AabbComponent), (With<Unit>, Without<Corpse>)>,
) {
    let mut combinations = unit_q.iter_combinations_mut();
    while let Some([(mut t1, a1), (t2, a2)]) = combinations.fetch_next() {
        while intersect_aabb(
            (t1.translation, t1.rotation, a1.0),
            (t2.translation, t2.rotation, a2.0),
        ) {
            let distance = t1.translation.distance(t2.translation);
            let offset = 0.01 * *t1.forward();

            if (t1.translation + offset).distance(t2.translation) > distance {
                t1.translation += offset;
            } else {
                t1.translation -= offset;
            }
        }
    }
}
