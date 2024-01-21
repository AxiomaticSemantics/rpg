use crate::character::{CharacterRecord, CharacterSlot};

use rpg_core::uid::Uid;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug, Ser, De)]
pub struct AccountId(pub u64);

#[derive(Default, Debug, Clone, PartialEq, Ser, De)]
pub struct AdminAccount {
    pub info: AdminAccountInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Ser, De)]
pub struct Account {
    pub info: AccountInfo,
    pub characters: Vec<CharacterRecord>,
}

impl Account {
    pub fn get_character_from_slot(&self, slot: CharacterSlot) -> Option<&CharacterRecord> {
        self.characters.iter().find(|c| c.info.slot == slot)
    }

    pub fn get_character_from_uid(&self, uid: Uid) -> Option<&CharacterRecord> {
        self.characters.iter().find(|c| c.info.uid == uid)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Ser, De)]
pub struct AdminAccountInfo {
    pub id: AccountId,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Ser, De)]
pub struct AccountInfo {
    pub id: AccountId,
    pub name: String,
    pub character_slots: usize,
}
