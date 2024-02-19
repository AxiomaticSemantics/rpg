use super::{equipment::*, inventory::*};
use crate::{item::Item, uid::Uid};

use serde_derive::{Deserialize as De, Serialize as Ser};

pub const STORAGE_ID_CURSOR: StorageIndex = StorageIndex(0);
pub const STORAGE_ID_HELMET: StorageIndex = StorageIndex(1);
pub const STORAGE_ID_BODY: StorageIndex = StorageIndex(2);
pub const STORAGE_ID_GLOVES: StorageIndex = StorageIndex(3);
pub const STORAGE_ID_BOOTS: StorageIndex = StorageIndex(4);
pub const STORAGE_ID_BELT: StorageIndex = StorageIndex(5);
pub const STORAGE_ID_LEFT_ARM: StorageIndex = StorageIndex(6);
pub const STORAGE_ID_RIGHT_ARM: StorageIndex = StorageIndex(7);
pub const STORAGE_ID_INVENTORY: StorageIndex = StorageIndex(8);
pub const STORAGE_ID_STASH: StorageIndex = StorageIndex(9);

// NOTE this must be kept in sync with the above nodes
pub const STORAGE_NODES: usize = 10;

pub const HERO_STASH_SLOTS: usize = 12 * 24;

/// A storage slot index
#[derive(Ser, De, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StorageIndex(pub u8);

/// An inventory slot index
#[derive(Ser, De, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SlotIndex(pub u16);

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
    fn slot_from_uid(&self, storage_index: StorageIndex, uid: Uid) -> Option<&Slot>;
    fn slot_from_uid_mut(&mut self, storage_index: StorageIndex, uid: Uid) -> Option<&mut Slot>;
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

    fn slot_from_uid(&self, storage_index: StorageIndex, uid: Uid) -> Option<&Slot> {
        let Some(storage) = self.storage.iter().find(|s| s.index == storage_index) else {
            println!("no storage {:?}", self.storage);
            return None;
        };

        storage
            .node
            .iter()
            .find(|s| s.item.as_ref().is_some_and(|s| s.uid == uid))
    }

    fn slot_from_uid_mut(&mut self, storage_index: StorageIndex, uid: Uid) -> Option<&mut Slot> {
        let Some(storage) = self.storage.iter_mut().find(|s| s.index == storage_index) else {
            return None;
        };

        storage
            .node
            .iter_mut()
            .find(|s| s.item.as_ref().is_some_and(|s| s.uid == uid))
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
        let mut storage = Vec::with_capacity(STORAGE_NODES);

        for i in 0..STORAGE_NODES as u8 {
            let slots = match i {
                _ if STORAGE_ID_CURSOR.0 == i => 1,
                _ if STORAGE_ID_HELMET.0 == i => EQUIPMENT_HELMET_SLOTS,
                _ if STORAGE_ID_BODY.0 == i => EQUIPMENT_BODY_SLOTS,
                _ if STORAGE_ID_GLOVES.0 == i => EQUIPMENT_GLOVE_SLOTS,
                _ if STORAGE_ID_BOOTS.0 == i => EQUIPMENT_BOOT_SLOTS,
                _ if STORAGE_ID_BELT.0 == i => EQUIPMENT_BELT_SLOTS,
                _ if STORAGE_ID_LEFT_ARM.0 == i => EQUIPMENT_LEFT_ARM_SLOTS,
                _ if STORAGE_ID_RIGHT_ARM.0 == i => EQUIPMENT_RIGHT_ARM_SLOTS,
                _ if STORAGE_ID_INVENTORY.0 == i => HERO_INVENTORY_SLOTS,
                _ if STORAGE_ID_STASH.0 == i => HERO_STASH_SLOTS,
                _ => unreachable!(),
            };

            let mut node = Vec::with_capacity(slots);
            for n in 0..slots {
                node.push(Slot {
                    index: SlotIndex(n as u16),
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
