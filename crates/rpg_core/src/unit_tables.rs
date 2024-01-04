use crate::{class::Class, skill::SkillId, villain::VillainId};

use std::borrow::Cow;
use std::collections::HashMap;

use serde_derive::Deserialize as De;

#[derive(De)]
pub struct VillainsTableEntry {
    pub name: Cow<'static, str>,
    pub class: Class,
    pub skill_id: SkillId,
    pub think_cooldown: f32,
    pub xp_reward: u64,
    pub drop_chances: u16,
    pub drop_chance: f32,
    pub move_speed: f32,
    pub max_vision: f32,
}

#[derive(De)]
pub struct HeroTable {
    pub default_skills: HashMap<Class, SkillId>,
}

#[derive(De)]
pub struct VillainTable {
    pub damage_scale: f32,
}

#[derive(De)]
pub struct UnitTable {
    pub hero: HeroTable,
    pub villain: VillainTable,
    pub villains: HashMap<VillainId, VillainsTableEntry>,
}
