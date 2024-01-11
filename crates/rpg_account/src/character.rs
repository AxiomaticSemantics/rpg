use rpg_core::{
    passive_tree::PassiveSkillGraph,
    storage::UnitStorage,
    uid::Uid,
    unit::{HeroGameMode, Unit},
};

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Clone, PartialEq, PartialOrd, Ser, De)]
pub struct CharacterInfo {
    pub name: String,
    pub uid: Uid,
    pub game_mode: HeroGameMode,
}

#[derive(Debug, Clone, Ser, De)]
pub struct Character {
    pub uid: Uid,
    pub unit: Unit,
    pub storage: UnitStorage,
    pub passive_tree: PassiveSkillGraph,
}

impl PartialEq for Character {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}
