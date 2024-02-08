use crate::{
    stat::{StatId, StatModifier},
    uid::Uid,
    value::Value,
};

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(PartialEq, Eq, PartialOrd, Hash, Default, Copy, Clone, Debug, Ser, De)]
pub struct ItemId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Ser, De)]
pub enum ItemKind {
    Gem,
    Potion,
    Currency,
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Ser, De)]
pub enum Rarity {
    #[default]
    Normal,
    Magic,
    Rare,
    Legendary,
    Unique,
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub enum GemClass {
    Attack,
    Defense,
}

#[derive(Debug, Clone, Ser, De)]
pub struct GemDescriptor {
    pub class: GemClass,
    pub max_modifiers: u8,
}

#[derive(Debug, Clone, Ser, De)]
pub struct PotionDescriptor {
    pub id: StatId,
}

#[derive(Debug, Clone, Copy, Ser, De)]
pub struct CurrencyId(pub u16);

#[derive(Debug, Clone, Ser, De)]
pub struct CurrencyDescriptor {
    pub id: CurrencyId,
}

#[derive(Debug, Clone, Ser, De)]
pub enum ItemDescriptor {
    Gem(GemDescriptor),
    Potion(PotionDescriptor),
    Currency(CurrencyDescriptor),
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct GemInfo {
    pub rarity: Rarity,
    pub level: u8,
    pub modifiers: Vec<StatModifier>,
    pub identified: bool,
    pub socketable: bool,
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct PotionInfo {
    pub id: StatId,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct CurrencyInfo {}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub enum ItemInfo {
    Gem(GemInfo),
    Potion(PotionInfo),
    Currency(CurrencyInfo),
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct ItemDrops {
    pub source: Uid,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct Item {
    pub uid: Uid,
    pub id: ItemId,
    pub info: ItemInfo,
}

impl Item {
    pub fn new(uid: Uid, id: ItemId, info: ItemInfo) -> Self {
        Self { uid, id, info }
    }
}

pub mod generation {
    use fastrand::Rng;
    use ordered_float::OrderedFloat;

    use crate::{
        item::{
            CurrencyInfo, GemInfo, Item, ItemDescriptor, ItemId, ItemInfo, ItemKind, PotionInfo,
            Rarity,
        },
        metadata::Metadata,
        stat::{
            modifier::{Affix, Modifier, ModifierFormat, ModifierId, ModifierKind, Operation},
            StatModifier,
        },
        uid::NextUid,
        unit_tables::VillainsTableEntry,
        value::{Sample, Value, ValueKind},
    };

    pub(crate) fn roll_item_drops(
        metadata: &Metadata,
        villain_info: &VillainsTableEntry,
        rng: &mut Rng,
        next_uid: &mut NextUid,
    ) -> Vec<Item> {
        let mut drops = Vec::new();
        for _ in 0..villain_info.drop_chances {
            let drop_roll = rng.f32();
            if drop_roll < villain_info.drop_chance {
                let item = super::generation::generate(rng, metadata, 1, next_uid);
                drops.push(item);
            }
        }

        drops
    }

    pub fn generate(rng: &mut Rng, metadata: &Metadata, level: u8, next_uid: &mut NextUid) -> Item {
        assert!(level > 0, "Level must be non-zero");

        let kind_roll = rng.f32();
        let kind = match kind_roll {
            v if v <= metadata.item.drop_info.base.gem => ItemKind::Gem,
            v if v <= metadata.item.drop_info.base.potion => ItemKind::Potion,
            v if v <= metadata.item.drop_info.base.currency => ItemKind::Currency,
            _ => panic!("Unexpected rng output"),
        };

        let rarity_info = &metadata.item.drop_info.rarity;
        let rarity_roll = rng.f32();
        let rarity = match rarity_roll {
            v if v <= rarity_info.normal.chance => Rarity::Normal,
            v if v <= rarity_info.magic.chance => Rarity::Magic,
            v if v <= rarity_info.rare.chance => Rarity::Rare,
            v if v <= rarity_info.legendary.chance => Rarity::Legendary,
            v if v <= rarity_info.unique.chance => Rarity::Unique,
            _ => unreachable!("rarity_roll outside of unit interval"),
        };

        // TODO add flags
        let (item_table_id, storable, socketable) = match kind {
            ItemKind::Gem => (
                rng.u16(metadata.item.gem_ids.begin.0..=metadata.item.gem_ids.end.0),
                true,
                true,
            ),
            ItemKind::Potion => (
                rng.u16(metadata.item.potion_ids.begin.0..=metadata.item.potion_ids.end.0),
                false,
                false,
            ),
            ItemKind::Currency => (
                rng.u16(metadata.item.currency_ids.begin.0..=metadata.item.currency_ids.end.0),
                true,
                false,
            ),
        };

        let entry = metadata.item.items.get(&ItemId(item_table_id)).unwrap();
        let mut modifiers = vec![];

        let item_info = match &entry.info {
            ItemDescriptor::Potion(info) => {
                let modifier_meta = &metadata
                    .modifier
                    .modifiers
                    .values()
                    .find(|m| m.stat_id == info.id)
                    .unwrap();

                //println!("{info:?} {:?}", modifier_meta);

                let stat_meta = metadata
                    .stat
                    .stats
                    .values()
                    .find(|s| s.id == info.id)
                    .unwrap();

                let amount = match stat_meta.value_kind {
                    ValueKind::U32 => {
                        Value::sample(rng, *modifier_meta.min.u32()..=*modifier_meta.max.u32())
                    }

                    ValueKind::U64 => {
                        Value::sample(rng, *modifier_meta.min.u64()..=*modifier_meta.max.u64())
                    }
                    // Only integer primitives are used
                    _ => unreachable!(),
                };

                /*
                let modifier = Modifier::new(
                    modifier_meta.id,
                    amount,
                    Operation::Add,
                    ModifierKind::Normal,
                    ModifierFormat::Flat,
                );

                modifiers.push(StatModifier::new(modifier_meta.stat_id, modifier));
                */

                ItemInfo::Potion(PotionInfo {
                    id: info.id,
                    value: amount,
                })
            }
            ItemDescriptor::Currency(_) => ItemInfo::Currency(CurrencyInfo {}),
            ItemDescriptor::Gem(_) => {
                let rarity_info = &metadata.item.drop_info.rarity;

                let max_modifers = match rarity {
                    Rarity::Normal => rarity_info.normal.max_affixes,
                    Rarity::Magic => rarity_info.magic.max_affixes,
                    Rarity::Rare => rarity_info.rare.max_affixes,
                    Rarity::Legendary => rarity_info.legendary.max_affixes,
                    Rarity::Unique => rarity_info.unique.max_affixes,
                };

                let modifier_count = rng.u8(1..=max_modifers);

                for _count in 0..modifier_count {
                    // TODO handle id clash
                    // TODO decide where max generation attemps should be stored
                    // for _attempt in max_attempts {}

                    let affix_kind = if rng.bool() {
                        Affix::Prefix
                    } else {
                        Affix::Suffix
                    };

                    let modifier_id = match affix_kind {
                        Affix::Prefix => ModifierId(rng.u16(
                            metadata.modifier.prefix_ids.begin.0
                                ..=metadata.modifier.prefix_ids.end.0,
                        )),
                        Affix::Suffix => ModifierId(rng.u16(
                            metadata.modifier.suffix_ids.begin.0
                                ..=metadata.modifier.suffix_ids.end.0,
                        )),
                    };

                    let modifier_meta = &metadata.modifier.modifiers[&modifier_id];

                    let stat_meta = metadata
                        .stat
                        .stats
                        .values()
                        .find(|s| s.id == modifier_meta.stat_id)
                        .unwrap();

                    let amount = match stat_meta.value_kind {
                        ValueKind::U32 => {
                            Value::sample(rng, *modifier_meta.min.u32()..=*modifier_meta.max.u32())
                        }

                        ValueKind::U64 => {
                            Value::U64(rng.u64(modifier_meta.min.u64()..=modifier_meta.max.u64()))
                        }
                        ValueKind::F32 => Value::F32(OrderedFloat(
                            modifier_meta.min.f32()
                                + (modifier_meta.max.f32() - modifier_meta.min.f32()) * rng.f32(),
                        )),
                        ValueKind::F64 => Value::F64(OrderedFloat(
                            modifier_meta.min.f64()
                                + (modifier_meta.max.f64() - modifier_meta.min.f64()) * rng.f64(),
                        )),
                    };

                    let modifier = Modifier::new(
                        modifier_id,
                        amount,
                        Operation::Add,
                        ModifierKind::Normal,
                        ModifierFormat::Flat,
                    );

                    //println!("adding {modifier:?} to gem");
                    modifiers.push(StatModifier::new(modifier_meta.stat_id, modifier));
                }

                if rarity != Rarity::Normal {
                    println!("generated {rarity:?} item");
                }

                assert!(!modifiers.is_empty());

                ItemInfo::Gem(GemInfo {
                    rarity,
                    level,
                    modifiers,
                    identified: false,
                    socketable: true,
                })
            }
        };

        let item = Item {
            uid: next_uid.get(),
            id: ItemId(item_table_id),
            info: item_info,
        };

        next_uid.next();

        item
    }
}
