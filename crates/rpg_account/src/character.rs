use rpg_core::unit::Unit;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Ser, De)]
pub struct CharacterInfo {
    pub name: String,
}

#[derive(Debug, Ser, De)]
pub struct Character {
    pub unit: Unit,
}
