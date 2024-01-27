use bevy::{
    ecs::{bundle::Bundle, component::Component, entity::Entity, system::Resource},
    prelude::{Deref, DerefMut},
};

use rpg_core::{
    item::{Item, ItemDrops},
    uid::Uid,
};

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

#[derive(Resource, Default, Deref, DerefMut)]
pub struct GroundItemDrops(pub Vec<ItemDrops>);
