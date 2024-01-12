use crate::character::CharacterRecord;
use rpg_core::uid::Uid;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct Account {
    pub info: AccountInfo,
    pub characters: Vec<CharacterRecord>,
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct AccountInfo {
    pub uid: Uid,
    pub name: String,
    pub character_slots: usize,
}
