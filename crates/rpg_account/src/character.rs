use crate::character_statistics::CharacterStatistics;

use rpg_core::{
    game_mode::GameMode,
    passive_tree::UnitPassiveSkills,
    skill::{Skill, SkillSlot},
    storage::UnitStorage,
    uid::Uid,
    unit::Unit,
};

use rpg_world::zone::ZoneId;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Copy, Clone, PartialEq, Ser, De)]
pub struct CharacterSlot(pub usize);

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct CharacterRecord {
    pub info: CharacterInfo,
    pub statistics: CharacterStatistics,
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
    pub game_mode: GameMode,
}

#[derive(Debug, Clone, Ser, De)]
pub struct Character {
    pub unit: Unit,
    pub skills: Vec<Skill>,
    pub skill_slots: Vec<SkillSlot>,
    pub storage: UnitStorage,
    pub passive_tree: UnitPassiveSkills,
    pub waypoints: Vec<ZoneId>,
}
