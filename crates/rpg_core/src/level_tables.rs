use std::collections::HashMap;
use serde_derive::Deserialize as De;

#[derive(De, Default)]
pub struct LevelLimits {
    pub max: u8
}

#[derive(De, Default)]
pub struct LevelTableEntry {
    pub level: u8,
    pub xp_begin: u64,
    pub xp_end: u64,
}

#[derive(De)]
pub struct LevelTable {
    pub limits: LevelLimits,
    pub levels: HashMap<u8, LevelTableEntry>,
}
