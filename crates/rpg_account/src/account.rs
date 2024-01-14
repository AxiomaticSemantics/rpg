use crate::character::CharacterRecord;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Default, Copy, Clone, PartialEq, Eq, Hash, Debug, Ser, De)]
pub struct AccountId(pub u64);

#[derive(Default, Debug, Clone, PartialEq, Ser, De)]
pub struct Account {
    pub info: AccountInfo,
    pub characters: Vec<CharacterRecord>,
}

#[derive(Default, Debug, Clone, PartialEq, Ser, De)]
pub struct AccountInfo {
    pub id: AccountId,
    pub name: String,
    pub character_slots: usize,
}
