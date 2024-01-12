use rpg_core::{
    passive_tree::PassiveSkillGraph,
    storage::UnitStorage,
    uid::Uid,
    unit::{HeroGameMode, Unit},
};

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Copy, Clone, PartialEq, Ser, De)]
pub struct CharacterSlot(pub usize);

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct CharacterRecord {
    pub info: CharacterInfo,
    pub character: Character,
}

impl PartialEq for Character {
    fn eq(&self, other: &Self) -> bool {
        self.unit.uid == other.unit.uid
    }
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct CharacterInfo {
    pub name: String,
    pub slot: CharacterSlot,
    pub uid: Uid,
    pub game_mode: HeroGameMode,
}

#[derive(Debug, Clone, Ser, De)]
pub struct Character {
    pub unit: Unit,
    pub storage: UnitStorage,
    pub passive_tree: PassiveSkillGraph,
}
