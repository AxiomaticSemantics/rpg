use std::collections::HashMap;

use serde_derive::Deserialize as De;

use crate::item::{ItemId, ItemInfo, ItemKind};

#[derive(De)]
pub struct RarityInfo {
    pub chance: f32,
    pub max_affixes: u8,
}

#[derive(De)]
pub struct DropRarity {
    pub normal: RarityInfo,
    pub magic: RarityInfo,
    pub rare: RarityInfo,
    pub legendary: RarityInfo,
    pub unique: RarityInfo,
}

#[derive(De)]
pub struct BaseDropInfo {
    pub gem: f32,
    pub currency: f32,
    pub potion: f32,
}

#[derive(De)]
pub struct IdInfo {
    pub begin: ItemId,
    pub end: ItemId,
}

#[derive(De)]
pub struct GemDropInfo {
    pub attack: f32,
    pub defense: f32,
}

#[derive(De)]
pub struct PotionDropInfo {
    pub hp: f32,
    pub ep: f32,
    pub mp: f32,
}

#[derive(De)]
pub struct CurrencyDropInfo {
    pub scroll: f32,
    pub orb: f32,
}

#[derive(De)]
pub struct DropInfo {
    pub rarity: DropRarity,
    pub base: BaseDropInfo,
    pub currency: CurrencyDropInfo,
    pub potion: PotionDropInfo,
}

#[derive(De)]
pub struct ItemTableEntry {
    pub name: String,
    pub kind: ItemKind,
    pub info: ItemInfo,
}

#[derive(De)]
pub struct ItemTable {
    pub drop_info: DropInfo,
    pub gem_ids: IdInfo,
    pub currency_ids: IdInfo,
    pub scroll_ids: IdInfo,
    pub orb_ids: IdInfo,
    pub potion_ids: IdInfo,
    pub items: HashMap<ItemId, ItemTableEntry>,
}
