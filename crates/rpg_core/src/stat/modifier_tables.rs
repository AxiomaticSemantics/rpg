use super::{modifier::ModifierId, modifier_pool::ModifierDescriptor};

use std::collections::HashMap;

use serde_derive::Deserialize as De;

#[derive(De)]
pub struct ModifierIds {
    pub begin: ModifierId,
    pub end: ModifierId,
}

#[derive(De)]
pub struct ModifierTable {
    pub prefix_ids: ModifierIds,
    pub suffix_ids: ModifierIds,
    pub prefix_attack_ids: ModifierIds,
    pub prefix_defense_ids: ModifierIds,
    pub suffix_attack_ids: ModifierIds,
    pub suffix_defense_ids: ModifierIds,
    pub reward_ids: ModifierIds,
    pub modifiers: HashMap<ModifierId, ModifierDescriptor>,
}
