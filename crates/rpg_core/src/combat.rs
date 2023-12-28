use crate::item::Item;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AttackResult {
    Hit,
    HitCrit,
    Blocked,
    Dodged,
}

#[derive(Debug, Copy, Clone, PartialEq)]
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
