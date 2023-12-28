use crate::{
    damage::Damage,
    skill::{
        effect::EffectInfo,
        skill::{Origin, SkillId, SkillInfo},
    },
    stat::stat::Stat,
};

use std::collections::HashMap;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Ser, De)]
pub struct SkillTableEntry {
    pub info: SkillInfo,
    pub origin: Origin,
    pub use_range: u32,
    pub base_damage: Damage,
    pub base_cost: Vec<Stat>,
    pub effects: Option<Vec<EffectInfo>>,
    pub use_duration_secs: f32,
    pub cooldown: Option<f32>,
}

#[derive(Default, Ser, De)]
pub struct SkillTable {
    pub skills: HashMap<SkillId, SkillTableEntry>,
}
