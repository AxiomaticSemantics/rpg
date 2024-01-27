use crate::{damage::Damage, item::Item};

#[derive(Debug, Clone, PartialEq)]
pub enum AttackResult {
    Hit(Damage),
    HitCrit(Damage),
    Blocked,
    Dodged,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatResult {
    Attack(AttackResult),
    Death(AttackResult),
    // FIXME rename this
    IsDead,
}

#[derive(Debug, Clone)]
pub struct DeathResult {
    pub items: Vec<Item>,
}
