use crate::{
    stat::{StatId, StatModifier},
    uid::Uid,
};

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(PartialEq, Copy, Clone, Default, Debug, Ser, De)]
pub struct ItemUid(pub Uid);

#[derive(PartialEq, Eq, PartialOrd, Hash, Default, Copy, Clone, Debug, Ser, De)]
pub struct ItemId(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Ser, De)]
pub enum ItemKind {
    Resource,
    Gem,
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
pub struct GemInfo {
    pub class: GemClass,
    pub max_modifiers: u8,
}

#[derive(Debug, Clone, Ser, De)]
pub struct ResourceInfo {
    pub id: StatId,
}

#[derive(Debug, Clone, Ser, De)]
pub enum ItemInfo {
    Resource(ResourceInfo),
    Gem(GemInfo),
}

#[derive(Debug, Clone, Ser, De)]
pub struct Item {
    pub uid: ItemUid,
    pub id: ItemId,
    pub level: u8,
    pub rarity: Rarity,
    pub modifiers: Vec<StatModifier>,
    pub storable: bool,
    pub socketable: bool,
}

impl Item {
    pub fn new(
        uid: ItemUid,
        id: ItemId,
        level: u8,
        rarity: Rarity,
        modifiers: Vec<StatModifier>,
        storable: bool,
        socketable: bool,
    ) -> Self {
        Self {
            uid,
            id,
            level,
            rarity,
            modifiers,
            storable,
            socketable,
        }
    }
}

pub mod generation {
    use fastrand::Rng;
    use ordered_float::OrderedFloat;

    use crate::{
        item::{Item, ItemId, ItemInfo, ItemKind, ItemUid, Rarity},
        metadata::Metadata,
        stat::{
            modifier::{Affix, Modifier, ModifierFormat, ModifierId, ModifierKind, Operation},
            value::{Sample, Value, ValueKind},
            StatModifier,
        },
        uid::NextUid,
    };

    pub fn generate(rng: &mut Rng, metadata: &Metadata, level: u8, next_uid: &mut NextUid) -> Item {
        assert!(level > 0, "Level must be non-zero");

        let uid = next_uid.0;

        let kind_roll = rng.f32();
        let kind = match kind_roll {
            v if v <= metadata.item.drop_info.base.gem => ItemKind::Gem,
            v if v <= metadata.item.drop_info.base.resource => ItemKind::Resource,
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
            ItemKind::Resource => (
                rng.u16(metadata.item.resource_ids.begin.0..=metadata.item.resource_ids.end.0),
                false,
                false,
            ),
        };

        let entry = metadata.item.items.get(&ItemId(item_table_id)).unwrap();
        let mut modifiers = vec![];

        match &entry.info {
            ItemInfo::Resource(info) => {
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
                        Value::U64(rng.u64(modifier_meta.min.u64()..=modifier_meta.max.u64()))
                    }
                    ValueKind::F32 => Value::F32(OrderedFloat(rng.f32())),
                    ValueKind::F64 => Value::F64(OrderedFloat(rng.f64())),
                };

                let modifier = Modifier::new(
                    modifier_meta.id,
                    amount,
                    Operation::Add,
                    ModifierKind::Normal,
                    ModifierFormat::Flat,
                );

                modifiers.push(StatModifier::new(modifier_meta.stat_id, modifier));
            }
            ItemInfo::Gem(info) => {
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
                        ValueKind::F32 => Value::F32(OrderedFloat(rng.f32())),
                        ValueKind::F64 => Value::F64(OrderedFloat(rng.f64())),
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
            }
        }

        let item = Item {
            uid: ItemUid(uid),
            id: ItemId(item_table_id),
            level,
            rarity,
            modifiers,
            storable,
            socketable,
        };

        next_uid.0 .0 += 1;

        item
    }
}
