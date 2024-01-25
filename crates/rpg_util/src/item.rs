use bevy::{
    ecs::{bundle::Bundle, component::Component, entity::Entity, system::Resource},
    prelude::{Deref, DerefMut},
};

use rpg_core::item::Item;

#[derive(Component, Deref, DerefMut)]
pub struct GroundItem(pub Option<Item>);

#[derive(Component)]
pub struct ResourceItem;

#[derive(Component)]
pub struct StorableItem;

#[derive(Bundle)]
pub struct GroundItemBundle {
    pub item: GroundItem,
}

pub struct GroundItemDrop {
    pub source: Entity,
    pub items: Vec<Item>,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct GroundItemDrops(pub Vec<GroundItemDrop>);
