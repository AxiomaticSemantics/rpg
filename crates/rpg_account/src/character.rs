use rpg_core::{
    passive_tree::PassiveSkillGraph,
    storage::UnitStorage,
    unit::{HeroGameMode, Unit},
};

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Clone, PartialEq, PartialOrd, Ser, De)]
pub struct CharacterInfo {
    pub name: String,
    pub hero_mode: HeroGameMode,
}

#[derive(Debug, Clone, Ser, De)]
pub struct Character {
    pub unit: Unit,
    pub storage: UnitStorage,
    pub passive_tree: PassiveSkillGraph,
}
