use crate::{
    class::Class,
    item::{Item, ItemInfo},
    metadata::Metadata,
    storage::*,
    value::{Value, ValueKind},
};

use super::{
    modifier::Operation, stat_list::StatList, Stat, StatChange, StatId, StatModifier, StatUpdate,
};

use serde_derive::{Deserialize as De, Serialize as Ser};

use std::borrow::Cow;
use std::collections::HashMap;

pub struct StatIdMap(pub HashMap<Cow<'static, str>, StatId>);

#[derive(Ser, De, Debug, Clone)]
pub struct BaseStats {
    pub stats: HashMap<Cow<'static, str>, Stat>,
}

impl BaseStats {
    pub fn new(metadata: &Metadata) -> Self {
        let mut base = HashMap::with_capacity(metadata.stat.base_stats.len());

        for stat in metadata.stat.base_stats.iter() {
            let descriptor = &metadata.stat.stats[stat];
            base.insert(
                stat.clone(),
                Stat::new(descriptor.id, Value::zero(descriptor.value_kind)),
            );
        }

        Self { stats: base }
    }

    pub fn assign_bonus_stats(&mut self, class: Class, metadata: &Metadata) {
        let stats = &metadata.stat;

        let bonus_stats = match class {
            Class::Str => &stats.class_str,
            Class::Dex => &stats.class_dex,
            Class::Int => &stats.class_int,
            Class::StrDex => &stats.class_str_dex,
            Class::DexInt => &stats.class_dex_int,
            Class::IntStr => &stats.class_int_str,
            Class::StrDexInt => &stats.class_str_dex_int,
        };

        self.stats.get_mut("Str").unwrap().value +=
            stats.base_stat_defaults["Str"] + bonus_stats["Str"];
        self.stats.get_mut("Dex").unwrap().value +=
            stats.base_stat_defaults["Dex"] + bonus_stats["Dex"];
        self.stats.get_mut("Int").unwrap().value +=
            stats.base_stat_defaults["Int"] + bonus_stats["Int"];

        self.stats.get_mut("Movement").unwrap().value = stats.base_stat_defaults["Movement"];
        self.stats.get_mut("RunCost").unwrap().value = stats.base_stat_defaults["RunCost"];
        self.stats.get_mut("Cooldown").unwrap().value = stats.base_stat_defaults["Cooldown"];
        self.stats.get_mut("CritChance").unwrap().value = stats.base_stat_defaults["CritChance"];
        self.stats.get_mut("CritMulti").unwrap().value = stats.base_stat_defaults["CritMulti"];
        self.stats.get_mut("PickupRadius").unwrap().value =
            stats.base_stat_defaults["PickupRadius"];
        self.stats.get_mut("LightRadius").unwrap().value = stats.base_stat_defaults["LightRadius"];

        self.stats.get_mut("EpDrain").unwrap().value = stats.base_stat_defaults["EpDrain"];

        self.stats.get_mut("HpRegen").unwrap().value = stats.base_stat_defaults["HpRegen"];
        self.stats.get_mut("EpRegen").unwrap().value = stats.base_stat_defaults["EpRegen"];
        self.stats.get_mut("MpRegen").unwrap().value = stats.base_stat_defaults["MpRegen"];
    }

    pub fn get(&self, id: &str) -> &Stat {
        debug_assert!(self.stats.contains_key(id));

        &self.stats[id]
    }

    pub fn set(&mut self, id: &str, value: Value) {
        debug_assert!(self.stats.contains_key(id));

        self.stats.get_mut(id).unwrap().value = value;
    }
}

#[derive(Ser, De, Debug, Clone)]
pub struct StatAccumulators {
    pub hp_regen: Stat,
    pub ep_regen: Stat,
    pub mp_regen: Stat,
    pub hp_drain: Stat,
    pub ep_drain: Stat,
    pub mp_drain: Stat,
}

impl StatAccumulators {
    pub fn new(metadata: &Metadata) -> Self {
        Self {
            hp_regen: Stat::new(
                metadata.stat.stats["HpRegen"].id,
                Value::zero(ValueKind::F32),
            ),
            ep_regen: Stat::new(
                metadata.stat.stats["EpRegen"].id,
                Value::zero(ValueKind::F32),
            ),
            mp_regen: Stat::new(
                metadata.stat.stats["MpRegen"].id,
                Value::zero(ValueKind::F32),
            ),
            hp_drain: Stat::new(
                metadata.stat.stats["HpDrain"].id,
                Value::zero(ValueKind::F32),
            ),
            ep_drain: Stat::new(
                metadata.stat.stats["EpDrain"].id,
                Value::zero(ValueKind::F32),
            ),
            mp_drain: Stat::new(
                metadata.stat.stats["MpDrain"].id,
                Value::zero(ValueKind::F32),
            ),
        }
    }

    fn compute_stat_update(
        &mut self,
        metadata: &Metadata,
        vital: &VitalStats,
        accumulated: u32,
        key: &'static str,
    ) -> Option<StatUpdate> {
        let key_max = format!("{key}Max");
        let value_kind = metadata.stat.stats[key].value_kind;

        let original_value = vital.stats[key].value;
        let max_value = vital.stats[key_max.as_str()].value;

        let updated_value = original_value + accumulated;
        let capped_value = updated_value.clamp(Value::zero(value_kind), max_value);

        let gain = capped_value - original_value;
        if *gain.u32() > 0 {
            Some(StatUpdate {
                id: vital.stats[key].id,
                total: capped_value,
                change: StatChange::Gain(gain),
            })
        } else {
            None
        }
    }

    /// If stat accumulators produce any updates return the new total and absolute value gained
    pub fn update_regeneration(
        &mut self,
        metadata: &Metadata,
        vital: &mut VitalStats,
        dt: f32,
    ) -> Vec<StatUpdate> {
        // Update the stat accumulators
        self.hp_regen.value +=
            vital.stats["HpRegen"].value * *vital.stats["HpMax"].value.u32() as f32 * dt;
        self.ep_regen.value +=
            vital.stats["EpRegen"].value * *vital.stats["EpMax"].value.u32() as f32 * dt;
        self.mp_regen.value +=
            vital.stats["MpRegen"].value * *vital.stats["MpMax"].value.u32() as f32 * dt;

        let mut updates = vec![];

        // If there is a value produced take it and update the stat
        if *self.hp_regen.value.f32() >= 1_f32 {
            let value = self.hp_regen.value.f32().floor() as u32;
            if value != 0 {
                self.hp_regen.value -= value as f32;

                let hp_update = self.compute_stat_update(metadata, vital, value, "Hp");
                if let Some(update) = hp_update {
                    vital.stats.get_mut("Hp").unwrap().value = update.total;
                    updates.push(update);
                }
            }
        }

        if *self.ep_regen.value.f32() >= 1_f32 {
            let value = self.ep_regen.value.f32().floor() as u32;
            if value != 0 {
                self.ep_regen.value -= value as f32;

                let ep_update = self.compute_stat_update(metadata, vital, value, "Ep");
                if let Some(update) = ep_update {
                    vital.stats.get_mut("Ep").unwrap().value = update.total;
                    updates.push(update);
                }
            }
        }

        if *self.mp_regen.value.f32() >= 1_f32 {
            let value = self.mp_regen.value.f32().floor() as u32;

            if value != 0 {
                self.mp_regen.value -= value as f32;

                let mp_update = self.compute_stat_update(metadata, vital, value, "Mp");
                if let Some(update) = mp_update {
                    vital.stats.get_mut("Mp").unwrap().value = update.total;
                    updates.push(update);
                }
            }
        }

        updates
    }

    pub fn consume_stamina(&mut self, base: &BaseStats, vital: &mut VitalStats, dt: f32) -> bool {
        if vital.stats["Ep"].value < 1_u32 {
            return false;
        }

        self.ep_drain.value += *base.stats["RunCost"].value.u32() as f32 * dt;
        if self.ep_drain.value >= 1_f32 {
            let accumulated = self.ep_drain.value.f32().floor();
            self.ep_drain.value -= accumulated;
            vital.stats.get_mut("Ep").unwrap().value -= accumulated as u32;
        }

        true
    }
}

#[derive(Ser, De, Debug, Clone)]
pub struct VitalStats {
    pub stats: HashMap<Cow<'static, str>, Stat>,
}

impl VitalStats {
    pub fn new(metadata: &Metadata) -> Self {
        let mut stats = HashMap::with_capacity(metadata.stat.vital_stats.len());

        for id in metadata.stat.vital_stats.iter() {
            let stat = &metadata.stat.stats[id];
            stats.insert(id.clone(), Stat::new(stat.id, Value::zero(stat.value_kind)));
        }

        Self { stats }
    }

    pub fn get_stat(&self, id: &str) -> Option<&Stat> {
        self.stats.get(id)
    }

    pub fn get_mut_stat(&mut self, id: &str) -> Option<&mut Stat> {
        self.stats.get_mut(id)
    }

    pub fn get_stat_from_id(&self, id: StatId) -> Option<&Stat> {
        self.stats.values().find(|s| s.id == id)
    }

    pub fn get_mut_stat_from_id(&mut self, id: StatId) -> Option<&mut Stat> {
        self.stats.values_mut().find(|s| s.id == id)
    }

    pub fn set(&mut self, id: &str, value: Value) {
        debug_assert!(self.stats.contains_key(id));

        self.stats.get_mut(id).unwrap().value = value;
    }

    pub fn set_from_id(&mut self, id: StatId, value: Value) {
        let stat = self.stats.values_mut().find(|s| s.id == id);
        debug_assert!(stat.is_some(), "{id:?}");

        stat.unwrap().value = value;
    }
}

#[derive(Ser, De, Debug, Clone)]
pub struct Stats {
    pub base: BaseStats,
    pub vitals: VitalStats,
    pub accumulators: StatAccumulators,
    pub list: HashMap<Cow<'static, str>, StatList>,
    pub item_stats: HashMap<Cow<'static, str>, StatList>,
    pub passive_skill_stats: HashMap<Cow<'static, str>, StatList>,
}

impl Stats {
    pub fn new(metadata: &Metadata) -> Self {
        let mut list: HashMap<Cow<'static, str>, _> = HashMap::new();
        for stat in &metadata.stat.stats {
            list.insert(stat.0.clone(), StatList::new(stat.1.value_kind));
        }

        let mut item_stats: HashMap<Cow<'static, str>, _> = HashMap::new();
        for stat in &metadata.stat.stats {
            item_stats.insert(stat.0.clone(), StatList::new(stat.1.value_kind));
        }

        let mut passive_skill_stats: HashMap<Cow<'static, str>, _> = HashMap::new();
        for stat in &metadata.stat.stats {
            passive_skill_stats.insert(stat.0.clone(), StatList::new(stat.1.value_kind));
        }

        Self {
            base: BaseStats::new(metadata),
            vitals: VitalStats::new(metadata),
            accumulators: StatAccumulators::new(metadata),
            list,
            item_stats,
            passive_skill_stats,
        }
    }

    pub fn apply_modifier(&mut self, metadata: &Metadata, modifier: &StatModifier) {
        let (str_id, _) = metadata
            .stat
            .stats
            .iter()
            .find(|d| d.1.id == modifier.id)
            .unwrap();
        let modifiers = self.list.get_mut(str_id).unwrap();

        match modifier.modifier.operation {
            Operation::Add => modifiers.modifiers.add.push(modifier.modifier.value),
            Operation::Sub => modifiers.modifiers.sub.push(modifier.modifier.value),
            Operation::Mul => modifiers.modifiers.mul.push(modifier.modifier.value),
            Operation::Div => modifiers.modifiers.div.push(modifier.modifier.value),
        }
    }

    fn apply_item_modifier(&mut self, metadata: &Metadata, modifier: &StatModifier) -> Option<()> {
        let (str_id, _) = metadata.stat.stats.iter().find(|d| d.1.id == modifier.id)?;
        let list_modifier = self.item_stats.get_mut(str_id)?;

        match modifier.modifier.operation {
            Operation::Add => list_modifier.modifiers.add.push(modifier.modifier.value),
            Operation::Sub => list_modifier.modifiers.sub.push(modifier.modifier.value),
            Operation::Mul => list_modifier.modifiers.mul.push(modifier.modifier.value),
            Operation::Div => list_modifier.modifiers.div.push(modifier.modifier.value),
        }

        Some(())
    }

    fn apply_item_modifiers(&mut self, metadata: &Metadata, item: &Item) -> Option<()> {
        let ItemInfo::Gem(info) = &item.info else {
            return None;
        };

        for modifier in &info.modifiers {
            self.apply_item_modifier(metadata, modifier);
        }

        Some(())
    }

    pub fn apply_item_stats(&mut self, metadata: &Metadata, storage: &UnitStorage) {
        self.clear_item_stats();

        for node in storage.storage.iter() {
            if node.index == StorageIndex(STORAGE_INVENTORY)
                || node.index == StorageIndex(STORAGE_STASH)
            {
                continue;
            }

            for item in node.node.iter() {
                let Some(item) = &item.item else {
                    continue;
                };

                self.apply_item_modifiers(metadata, item);
            }
        }

        self.recompute(false);
    }

    pub fn clear_item_stats(&mut self) {
        for list in self.item_stats.values_mut() {
            list.modifiers.add.clear();
            list.modifiers.sub.clear();
            list.modifiers.mul.clear();
            list.modifiers.div.clear();
            list.add_sum = Value::zero(list.value_kind);
            list.mul_sum = Value::zero(list.value_kind);
        }

        //self.item_stats = self.list.clone();
    }

    pub fn clear_passive_stats(&mut self) {
        for list in self.passive_skill_stats.values_mut() {
            list.modifiers.add.clear();
            list.modifiers.sub.clear();
            list.modifiers.mul.clear();
            list.modifiers.div.clear();
            list.add_sum = Value::zero(list.value_kind);
            list.mul_sum = Value::zero(list.value_kind);
        }

        //self.passive_skill_stats = self.list.clone();
    }

    pub fn recompute(&mut self, force: bool) {
        for list in self.list.values_mut() {
            list.compute_sum();
        }

        for list in self.item_stats.values_mut() {
            list.compute_sum();
        }

        for list in self.passive_skill_stats.values_mut() {
            list.compute_sum();
        }

        for stat_name in ["Str", "Dex", "Int"] {
            let item_value = if let Some(list) = self.item_stats.get(stat_name) {
                list.add_sum
            } else {
                Value::U32(0)
            };

            let passive_value = if let Some(list) = self.passive_skill_stats.get(stat_name) {
                list.add_sum
            } else {
                Value::U32(0)
            };

            match stat_name {
                "Str" => {
                    self.vitals.stats.get_mut("HpMax").unwrap().value = (self.base.stats["Str"]
                        .value
                        + self.list["Str"].add_sum
                        + item_value
                        + passive_value)
                        * 2_u32
                        + self.list["Hp"].add_sum;
                }
                "Dex" => {
                    self.vitals.stats.get_mut("EpMax").unwrap().value = (self.base.stats["Dex"]
                        .value
                        + self.list["Dex"].add_sum
                        + item_value
                        + passive_value)
                        * 2_u32
                        + self.list["Ep"].add_sum;
                }
                "Int" => {
                    self.vitals.stats.get_mut("MpMax").unwrap().value = (self.base.stats["Int"]
                        .value
                        + self.list["Int"].add_sum
                        + item_value
                        + passive_value)
                        * 2_u32
                        + self.list["Mp"].add_sum;
                }
                _ => unreachable!(),
            }
        }

        if force {
            self.vitals.stats.get_mut("Hp").unwrap().value = self.vitals.stats["HpMax"].value;
            self.vitals.stats.get_mut("Ep").unwrap().value = self.vitals.stats["EpMax"].value;
            self.vitals.stats.get_mut("Mp").unwrap().value = self.vitals.stats["MpMax"].value;
        }

        self.vitals.stats.get_mut("HpRegen").unwrap().value = self.base.stats["HpRegen"].value
            + self.list["HpRegen"].add_sum
            + self.item_stats["HpRegen"].add_sum
            + self.passive_skill_stats["HpRegen"].add_sum;

        self.vitals.stats.get_mut("EpRegen").unwrap().value =
            self.base.stats["EpRegen"].value + self.list["EpRegen"].add_sum;
        self.vitals.stats.get_mut("MpRegen").unwrap().value =
            self.base.stats["MpRegen"].value + self.list["MpRegen"].add_sum;

        self.vitals.stats.get_mut("LightRadius").unwrap().value =
            self.base.stats["LightRadius"].value + self.list["LightRadius"].add_sum;
        self.vitals.stats.get_mut("PickupRadius").unwrap().value =
            self.base.stats["PickupRadius"].value + self.list["PickupRadius"].add_sum;

        self.vitals.stats.get_mut("IncAttackSpeed").unwrap().value =
            self.list["IncAttackSpeed"].add_sum;
        self.vitals.stats.get_mut("IncCastSpeed").unwrap().value =
            self.list["IncCastSpeed"].add_sum;

        self.vitals.stats.get_mut("Movement").unwrap().value = Value::U32(
            (*self.base.stats["Movement"].value.u32() as f32
                * (1. + *self.list["IncMovement"].add_sum.f32())) as u32,
        );
        self.vitals.stats.get_mut("CritChance").unwrap().value =
            self.base.stats["CritChance"].value + self.list["CritChance"].add_sum;
        self.vitals.stats.get_mut("CritMulti").unwrap().value =
            self.base.stats["CritMulti"].value + self.list["CritMulti"].add_sum;

        /*
        self.vitals.stats.get_mut("ProjSize").unwrap().value =
            self.base.stats["ProjSize"].value + self.list["ProjSize"].add_sum;

        self.vitals.stats.get_mut("ProjSpeed").unwrap().value =
            self.base.stats["ProjSpeed"].value + self.list["ProjSpeed"].add_sum;

        self.vitals.stats.get_mut("ProjDuration").unwrap().value =
            self.base.stats["ProjDuration"].value * (self.list["ProjDuration"].add_sum + 1_f32);

        self.vitals.stats.get_mut("PickupRadius").unwrap().value =
            self.base.stats["PickupRadius"].value + self.list["PickupRadius"].add_sum;
        */

        self.vitals.stats.get_mut("Cooldown").unwrap().value =
            self.base.stats["Cooldown"].value - self.list["Cooldown"].add_sum;
        /*
        self.vitals.stats.get_mut("KnockbackChance").unwrap().value =
            self.list["KnockbackChance"].add_sum;
        */
        self.vitals.stats.get_mut("BlockChance").unwrap().value = self.list["BlockChance"].add_sum;
        self.vitals.stats.get_mut("DodgeChance").unwrap().value = self.list["DodgeChance"].add_sum;
    }

    pub fn apply_regeneration(&mut self, metadata: &Metadata, dt: f32) -> Vec<StatUpdate> {
        self.accumulators
            .update_regeneration(metadata, &mut self.vitals, dt)
    }

    pub fn consume_stamina(&mut self, dt: f32) -> bool {
        self.accumulators
            .consume_stamina(&self.base, &mut self.vitals, dt)
    }
}
