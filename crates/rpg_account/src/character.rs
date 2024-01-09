use rpg_core::{passive_tree::PassiveSkillGraph, storage::UnitStorage, unit::Unit};

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Ser, De)]
pub struct CharacterInfo {
    pub name: String,
}

#[derive(Debug, Ser, De)]
pub struct Character {
    pub unit: Unit,
    pub storage: UnitStorage,
    pub passive_tree: PassiveSkillGraph,
}
