pub mod equipment;
pub mod inventory;
mod storage_internal;

pub use storage_internal::{
    Slot, SlotIndex, Storage, StorageIndex, StorageNode, StorageSlot, UnitStorage, STORAGE_BELT,
    STORAGE_BODY, STORAGE_BOOTS, STORAGE_GLOVES, STORAGE_HELMET, STORAGE_INVENTORY,
    STORAGE_LEFT_ARM, STORAGE_RIGHT_ARM, STORAGE_STASH,
};
