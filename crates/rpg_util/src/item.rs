use bevy::{
    ecs::{bundle::Bundle, component::Component, system::Resource},
    prelude::{Deref, DerefMut},
};

use rpg_core::{
    item::{Item, ItemDrops},
    storage::{StorageSlot as RpgStorageSlot, UnitStorage as RpgUnitStorage},
};

#[derive(Component, Deref, DerefMut)]
pub struct GroundItem(pub Option<Item>);

#[derive(Component)]
pub struct StorableItem;

#[derive(Component, Copy, Clone, Deref, DerefMut, PartialEq, Eq, Debug)]
pub struct StorageSlot(pub RpgStorageSlot);

#[derive(Component, Deref, DerefMut)]
pub struct UnitStorage(pub RpgUnitStorage);

impl Default for UnitStorage {
    fn default() -> Self {
        Self(RpgUnitStorage::new())
    }
}

#[derive(Bundle)]
pub struct GroundItemBundle {
    pub item: GroundItem,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct GroundItemDrops(pub Vec<ItemDrops>);
