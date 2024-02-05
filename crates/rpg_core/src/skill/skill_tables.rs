use crate::{
    damage::DamageDescriptor,
    skill::{effect::EffectInfo, OriginKind, SkillId, SkillInfo, TimerDescriptor},
    stat::Stat,
};

use glam::Vec3;

use std::collections::HashMap;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Ser, De)]
pub struct SkillTableEntry {
    pub info: SkillInfo,
    pub origin_kind: OriginKind,
    pub origin: Vec3,
    pub use_range: u32,
    pub base_damage: DamageDescriptor,
    pub base_cost: Vec<Stat>,
    pub effects: Option<Vec<EffectInfo>>,
    pub use_duration_secs: f32,
    pub cooldown: Option<f32>,
    pub timer: Option<TimerDescriptor>,
}

#[derive(Default, Ser, De)]
pub struct SkillTable {
    pub skills: HashMap<SkillId, SkillTableEntry>,
}
