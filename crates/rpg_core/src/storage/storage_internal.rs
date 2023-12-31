use super::{equipment::*, inventory::*};
use crate::item::Item;

use serde_derive::{Deserialize as De, Serialize as Ser};

pub const STORAGE_HELMET: u32 = 0;
pub const STORAGE_BODY: u32 = 1;
pub const STORAGE_GLOVES: u32 = 2;
pub const STORAGE_BOOTS: u32 = 3;
pub const STORAGE_BELT: u32 = 4;
pub const STORAGE_LEFT_ARM: u32 = 5;
pub const STORAGE_RIGHT_ARM: u32 = 6;
pub const STORAGE_INVENTORY: u32 = 7;
pub const STORAGE_STASH: u32 = 8;

pub const HERO_STASH_SLOTS: usize = 12 * 24;

/// A storage slot index
#[derive(Ser, De, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StorageIndex(pub u32);

/// An inventory slot index
#[derive(Ser, De, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SlotIndex(pub u32);

/// An item storage slot
#[derive(Ser, De, Clone, Debug, Default)]
pub struct Slot {
    pub index: SlotIndex,
    pub item: Option<Item>,
}

#[derive(Ser, De, Clone, Debug)]
pub struct StorageNode {
    pub index: StorageIndex,
    pub node: Vec<Slot>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct StorageSlot {
    pub storage_index: StorageIndex,
    pub slot_index: SlotIndex,
}

pub trait Storage {
    fn get_empty_slot(&self) -> Option<&Slot>;
    fn get_empty_slot_mut(&mut self) -> Option<&mut Slot>;
    fn set_slot_item(&mut self, storage_index: StorageIndex, slot_index: SlotIndex, item: Item);
    fn swap_slot(&mut self, source: StorageSlot, target: StorageSlot);
    fn slot_has_item(&self, slot: StorageSlot) -> bool;
    fn slot_from_index(&self, storage_index: StorageIndex, slot_index: SlotIndex) -> Option<&Slot>;
    fn slot_from_index_mut(
        &mut self,
        storage_index: StorageIndex,
        slot_index: SlotIndex,
    ) -> Option<&mut Slot>;
}

#[derive(Debug, Clone, Ser, De)]
pub struct UnitStorage {
    pub storage: Vec<StorageNode>,
}

impl Default for UnitStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl Storage for UnitStorage {
    fn set_slot_item(&mut self, storage_index: StorageIndex, slot_index: SlotIndex, item: Item) {
        let storage = &mut self
            .storage
            .iter_mut()
            .find(|s| s.index == storage_index)
            .unwrap();
        let slot = storage
            .node
            .iter_mut()
            .find(|s| s.index == slot_index)
            .unwrap();

        slot.item = Some(item);
    }

    fn slot_has_item(&self, slot: StorageSlot) -> bool {
        self.slot_from_index(slot.storage_index, slot.slot_index)
            .unwrap()
            .item
            .is_some()
    }

    fn swap_slot(&mut self, source: StorageSlot, target: StorageSlot) {
        if source == target {
            return;
        }

        assert!(
            self.storage[source.storage_index.0 as usize].node[source.slot_index.0 as usize]
                .item
                .is_some()
        );

        let item = self.storage[source.storage_index.0 as usize].node[source.slot_index.0 as usize]
            .item
            .take();
        self.storage[target.storage_index.0 as usize].node[target.slot_index.0 as usize].item =
            item;

        /*
        mem::swap(
            &mut self.storage[target.storage_index.0 as usize].node[target.slot_index.0 as usize]
                .item,
            &mut self.storage[source.storage_index.0 as usize].node[source.slot_index.0 as usize]
                .item,
        );*/
    }

    fn slot_from_index(&self, storage_index: StorageIndex, slot_index: SlotIndex) -> Option<&Slot> {
        let Some(storage) = self.storage.iter().find(|s| s.index == storage_index) else {
            println!("no storage {:?}", self.storage);
            return None;
        };

        storage.node.iter().find(|s| s.index == slot_index)
    }

    fn slot_from_index_mut(
        &mut self,
        storage_index: StorageIndex,
        slot_index: SlotIndex,
    ) -> Option<&mut Slot> {
        let Some(storage) = self.storage.iter_mut().find(|s| s.index == storage_index) else {
            return None;
        };

        storage.node.iter_mut().find(|s| s.index == slot_index)
    }

    fn get_empty_slot(&self) -> Option<&Slot> {
        let inventory = &self.storage[7];

        inventory.node.iter().find(|s| s.item.is_none())
    }

    fn get_empty_slot_mut(&mut self) -> Option<&mut Slot> {
        let inventory = &mut self.storage[7];

        inventory.node.iter_mut().find(|s| s.item.is_none())
    }
}

impl UnitStorage {
    pub fn new() -> Self {
        let mut storage = Vec::with_capacity(9);

        for i in 0..9 {
            let slots = match i {
                STORAGE_HELMET => EQUIPMENT_HELMET_SLOTS,
                STORAGE_BODY => EQUIPMENT_BODY_SLOTS,
                STORAGE_GLOVES => EQUIPMENT_GLOVE_SLOTS,
                STORAGE_BOOTS => EQUIPMENT_BOOT_SLOTS,
                STORAGE_BELT => EQUIPMENT_BELT_SLOTS,
                STORAGE_LEFT_ARM => EQUIPMENT_LEFT_ARM_SLOTS,
                STORAGE_RIGHT_ARM => EQUIPMENT_RIGHT_ARM_SLOTS,
                STORAGE_INVENTORY => HERO_INVENTORY_SLOTS,
                STORAGE_STASH => HERO_STASH_SLOTS,
                _ => unreachable!(),
            };

            let mut node = Vec::with_capacity(slots);
            for n in 0..slots {
                node.push(Slot {
                    index: SlotIndex(n as u32),
                    item: None,
                });
            }

            storage.push(StorageNode {
                index: StorageIndex(i),
                node,
            });
        }

        Self { storage }
    }
}

/*
    pub fn slot_from_uid(&self, uid: ItemUid) -> Option<&Slot> {
        self.slots.iter().find(|s| match &s.item {
            Some(item) => item.uid == uid,
            _ => false,
        })
    }

    pub fn slot_from_uid_mut(&mut self, uid: ItemUid) -> Option<&mut Slot> {
        self.slots.iter_mut().find(|s| match &s.item {
            Some(item) => item.uid == uid,
            None => false,
        })
    }
}*/
