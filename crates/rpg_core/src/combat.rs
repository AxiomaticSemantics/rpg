use crate::{damage::Damage, item::Item};

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Ser, De, Clone, PartialEq)]
pub enum AttackResult {
    Hit(Damage),
    HitCrit(Damage),
    Blocked,
    Dodged,
}

#[derive(Debug, Ser, De, Clone, PartialEq)]
pub enum CombatResult {
    Attack(AttackResult),
    Death(AttackResult),
    // FIXME rename this
    IsDead,
}

#[derive(Debug, Ser, De, Clone)]
pub struct DeathResult {
    pub items: Vec<Item>,
}
