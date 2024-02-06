use crate::{
    damage::{DamageKind, DamageValue},
    unit::HeroReward,
};

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Ser, De, Clone, PartialEq)]
pub struct DamageResult {
    pub kind: DamageKind,
    pub damage: DamageValue,
    pub total: u32,
    pub is_crit: bool,
}

#[derive(Debug, Ser, De, Clone, PartialEq)]
pub struct VillainDeathResult {
    pub damage: DamageResult,
    pub reward: Option<HeroReward>,
}

#[derive(Debug, Ser, De, Clone, PartialEq)]
pub struct HeroDeathResult {
    pub damage: DamageResult,
}

#[derive(Debug, Ser, De, Clone, PartialEq)]
pub enum CombatResult {
    Damage(DamageResult),
    VillainDeath(VillainDeathResult),
    HeroDeath(HeroDeathResult),
    Blocked,
    Dodged,
    // FIXME rename this
    Error,
}
