use crate::{
    damage::{DamageKind, DamageValue},
    item::Item,
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
pub enum CombatResult {
    Damage(DamageResult),
    Death(DamageResult),
    Blocked,
    Dodged,
    // FIXME rename this
    Error,
}

// TODO remove this
#[derive(Debug, Ser, De, Clone, PartialEq)]
pub struct DeathResult {
    pub items: Vec<Item>,
}
