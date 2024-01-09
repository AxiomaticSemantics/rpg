use crate::character::{Character, CharacterInfo};

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Clone, Ser, De)]
pub struct Account {
    pub info: AccountInfo,
    pub characters: Vec<Character>,
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct AccountInfo {
    pub name: String,
    pub character_info: Vec<CharacterInfo>,
}
