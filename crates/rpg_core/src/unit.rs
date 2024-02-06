use crate::{
    class::Class,
    combat::{CombatResult, DamageResult, HeroDeathResult, VillainDeathResult},
    damage::{DamageDescriptor, DamageKind, DamageValue, DamageValueDescriptor},
    game_mode::GameMode,
    item::{self, Item},
    metadata::Metadata,
    passive_tree::PassiveSkillGraph,
    skill::{Skill, SkillId, SkillUseResult},
    stat::{modifier::Operation, stat_system::Stats, Stat},
    storage::{StorageIndex, UnitStorage, STORAGE_INVENTORY, STORAGE_STASH},
    uid::{NextUid, Uid},
    value::{Value, ValueKind},
    villain::VillainId,
};

use fastrand::Rng;
use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Default, Debug, Clone, Copy, Eq, PartialEq)]
pub enum UnitKind {
    #[default]
    Hero,
    Villain,
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct LevelReward {
    pub level: u8,
    pub passive_points: u8,
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct HeroReward {
    pub xp_gained: Value,
    pub xp_total: Value,
    pub level: Option<LevelReward>,
}

#[derive(Ser, De, Clone, Debug, PartialEq)]
pub struct HeroInfo {
    pub game_mode: GameMode,
    pub xp_curr: Stat,
}

impl HeroInfo {
    pub fn new(metadata: &Metadata, game_mode: GameMode) -> Self {
        Self {
            game_mode,
            xp_curr: Stat::new(metadata.stat.stats["Xp"].id, Value::zero(ValueKind::U64)),
        }
    }
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct VillainInfo {
    pub id: VillainId,
}

impl VillainInfo {
    pub fn new(id: VillainId) -> Self {
        Self { id }
    }
}

#[derive(Ser, De, Clone, Debug)]
pub enum UnitInfo {
    Hero(HeroInfo),
    Villain(VillainInfo),
}

impl UnitInfo {
    pub fn hero(&self) -> &HeroInfo {
        if let Self::Hero(info) = self {
            info
        } else {
            panic!("Bad unit info access");
        }
    }

    pub fn villain(&self) -> &VillainInfo {
        if let Self::Villain(info) = self {
            info
        } else {
            panic!("Bad unit info access");
        }
    }

    pub fn hero_mut(&mut self) -> &mut HeroInfo {
        if let Self::Hero(info) = self {
            info
        } else {
            panic!("Bad unit info access");
        }
    }

    pub fn villain_mut(&mut self) -> &mut VillainInfo {
        if let Self::Villain(info) = self {
            info
        } else {
            panic!("Bad unit info access");
        }
    }

    #[inline(always)]
    pub const fn is_hero(&self) -> bool {
        matches!(self, Self::Hero(_))
    }

    #[inline(always)]
    pub const fn is_villain(&self) -> bool {
        matches!(self, Self::Villain(_))
    }
}

#[derive(Ser, De, Clone, Debug)]
// FIXME stats and skills should be passed as params
pub struct Unit {
    pub uid: Uid,
    pub kind: UnitKind,
    pub info: UnitInfo,
    pub class: Class,
    pub level: u8,
    pub name: String,
    pub stats: Stats,
    pub passive_skill_points: u8,
}

impl Unit {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        uid: Uid,
        class: Class,
        kind: UnitKind,
        info: UnitInfo,
        level: u8,
        name: impl Into<String>,
        metadata: &Metadata,
    ) -> Self {
        let mut stats = Stats::new(metadata);
        stats.base.assign_bonus_stats(class, metadata);
        stats.recompute(true);

        Self {
            uid,
            kind,
            info,
            class,
            level,
            name: name.into(),
            stats,
            passive_skill_points: 0,
        }
    }

    pub fn add_default_skills(&mut self, skills: &mut Vec<Skill>, metadata: &Metadata) {
        let skill_id = match &self.info {
            UnitInfo::Hero(_) => metadata.unit.hero.default_skills[&self.class],
            UnitInfo::Villain(info) => metadata.unit.villains[&info.id].skill_id,
        };

        self.add_skill(skills, metadata, skill_id, 1);
    }

    pub fn add_skill(
        &mut self,
        skills: &mut Vec<Skill>,
        metadata: &Metadata,
        skill_id: SkillId,
        level: u8,
    ) {
        let skill_info = metadata.skill.skills.get(&skill_id).unwrap();

        let effects = match &skill_info.effects {
            Some(effects) => effects.clone(),
            None => Vec::new(),
        };

        skills.push(Skill::new(
            skill_id,
            level,
            skill_info.base_damage.clone(),
            skill_info.info.clone(),
            effects,
        ));
    }

    #[inline(always)]
    pub fn is_alive(&self) -> bool {
        *self.stats.vitals.stats["Hp"].value.u32() > 0
    }

    pub fn handle_attack(
        &mut self,
        attacker: &mut Self,
        metadata: &Metadata,
        rng: &mut Rng,
        damage: &DamageDescriptor,
    ) -> CombatResult {
        if !self.is_alive() {
            println!("handle_attack, skipping dead unit");
            return CombatResult::Error;
        }

        if self.dodge_attack(attacker, rng) {
            return CombatResult::Dodged;
        }

        if self.block_attack(attacker, rng) {
            return CombatResult::Blocked;
        }

        // TODO split Damage into seperate structs
        let mut damage_roll = match damage.value {
            DamageValueDescriptor::Flat(flat) => flat,
            DamageValueDescriptor::MinMax(min, max) => rng.u32(min..=max),
        };

        let crit_chance = *attacker.stats.vitals.stats["CritChance"].value.f32();
        let crit_multiplier = *attacker.stats.vitals.stats["CritMulti"].value.f32();
        let crit_chance_roll = if crit_chance > 0. { rng.f32() } else { 1. };
        let is_crit = crit_chance > crit_chance_roll;
        if is_crit {
            println!("critical strike: crit_chance:{:.2}% {:.2}% {crit_chance_roll} damage {damage_roll:?} -> {damage:?}",
                crit_chance * 100., crit_multiplier * 100.);
            damage_roll = (damage_roll as f32 * crit_multiplier).floor() as u32;
        }

        if attacker.kind == UnitKind::Villain {
            damage_roll = (damage_roll as f32 * metadata.unit.villain.damage_scale).floor() as u32;
        }

        let curr_hp = self.stats.vitals.stats["Hp"].value;

        let new_hp = self.stats.vitals.stats["Hp"]
            .value
            .u32()
            .saturating_sub(damage_roll);

        let damage_dealt = curr_hp - new_hp;

        *self
            .stats
            .vitals
            .stats
            .get_mut("Hp")
            .unwrap()
            .value
            .u32_mut() = new_hp;

        let damage = DamageResult {
            kind: DamageKind::Physical,
            damage: DamageValue {
                total: new_hp,
                damage: *damage_dealt.u32(),
            },
            total: new_hp,
            is_crit,
        };

        if self.is_alive() {
            CombatResult::Damage(damage)
        } else {
            if self.kind == UnitKind::Villain {
                let villain_info = self.info.villain();
                let villain_meta = &metadata.unit.villains.get(&villain_info.id).unwrap();

                let reward =
                    attacker.reward_experience(metadata, Value::U64(villain_meta.xp_reward));

                let death_result = VillainDeathResult { damage, reward };

                CombatResult::VillainDeath(death_result)
            } else {
                let death_result = HeroDeathResult { damage };

                CombatResult::HeroDeath(death_result)
            }
        }
    }

    pub fn apply_passive_rewards(
        &mut self,
        metadata: &Metadata,
        passive_skill_graph: &PassiveSkillGraph,
    ) {
        self.stats.clear_passive_stats();

        for node_id in &passive_skill_graph.allocated_nodes {
            let node = metadata
                .passive_tree
                .nodes
                .iter()
                .find(|n| n.id == *node_id)
                .unwrap();

            let Some(modifiers) = &node.modifiers else {
                continue;
            };

            for modifier in modifiers {
                let stat_descriptor = &metadata
                    .stat
                    .stats
                    .iter()
                    .find(|s| s.1.id == modifier.id)
                    .unwrap();

                let list = &mut self
                    .stats
                    .passive_skill_stats
                    .get_mut(stat_descriptor.0)
                    .unwrap();

                list.modifiers.add.push(modifier.value);
            }

            //.modifiers
        }

        self.stats.recompute(false);
    }

    /*
    // TODO add Reward type
    pub fn apply_rewards(&mut self, metadata: &Metadata, item: &Item) -> bool {
        assert!(self.is_alive());

        let mut gained_level = false;

        for modifier in item.modifiers.iter() {
            let (str_id, _) = metadata
                .stat
                .stats
                .iter()
                .find(|d| d.1.id == modifier.id)
                .unwrap();

            match modifier.id {
                _ if str_id == "Xp" => {
                    if self.reward_experience(metadata, modifier.modifier.value) {
                        self.stats.recompute(true);
                        gained_level = true;
                    }
                }
                _ if str_id == "Hp" => {
                    println!("apply hp {}", modifier.modifier.value);
                    (self.stats.vitals.stats.get_mut("Hp").unwrap().value +=
                        modifier.modifier.value);
                    if self.stats.vitals.stats["Hp"].value > self.stats.vitals.stats["HpMax"].value
                    {
                        self.stats.vitals.stats.get_mut("Hp").unwrap().value =
                            self.stats.vitals.stats["HpMax"].value;
                    }
                }
                _ if str_id == "Ep" => {
                    println!("apply ep {}", modifier.modifier.value);
                    self.stats.vitals.stats.get_mut("Ep").unwrap().value += modifier.modifier.value;
                    if self.stats.vitals.stats["Ep"].value > self.stats.vitals.stats["EpMax"].value
                    {
                        self.stats.vitals.stats.get_mut("Ep").unwrap().value =
                            self.stats.vitals.stats["EpMax"].value;
                    }
                }
                _ if str_id == "Mp" => {
                    println!("apply mp {}", modifier.modifier.value);
                    self.stats.vitals.stats.get_mut("Mp").unwrap().value += modifier.modifier.value;
                    if self.stats.vitals.stats["Mp"].value > self.stats.vitals.stats["MpMax"].value
                    {
                        self.stats.vitals.stats.get_mut("Mp").unwrap().value =
                            self.stats.vitals.stats["MpMax"].value;
                    }
                }
                _ => unreachable!("This shouldn't happen"),
            }
        }

        gained_level
    }*/

    fn apply_item_modifiers(&mut self, metadata: &Metadata, item: &Item) -> Option<()> {
        for modifier in &item.modifiers {
            let (str_id, _) = metadata
                .stat
                .stats
                .iter()
                .find(|d| d.1.id == modifier.id)
                .unwrap();
            let list_modifier = self.stats.item_stats.get_mut(str_id)?;

            match modifier.modifier.operation {
                Operation::Add => list_modifier.modifiers.add.push(modifier.modifier.value),
                Operation::Sub => list_modifier.modifiers.sub.push(modifier.modifier.value),
                Operation::Mul => list_modifier.modifiers.mul.push(modifier.modifier.value),
                Operation::Div => list_modifier.modifiers.div.push(modifier.modifier.value),
            }
        }

        Some(())
    }

    pub fn apply_item_stats(&mut self, metadata: &Metadata, storage: &UnitStorage) {
        self.stats.clear_item_stats();

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

        self.stats.recompute(false);
    }

    // TODO add a flag to signify max level gained
    fn reward_experience(&mut self, metadata: &Metadata, value: Value) -> Option<HeroReward> {
        if self.kind != UnitKind::Hero {
            return None;
        }

        let hero_info = self.info.hero_mut();
        let curr_level_info = metadata.level.levels.get(&self.level).unwrap();

        let xp_prev = hero_info.xp_curr.value;

        hero_info.xp_curr.value += value;
        if hero_info.xp_curr.value > curr_level_info.xp_end {
            if metadata.level.levels.len() > self.level as usize {
                let new_level_info = metadata.level.levels.get(&(self.level + 1)).unwrap();
                self.level = new_level_info.level;
                self.passive_skill_points += 1;

                return Some(HeroReward {
                    xp_gained: hero_info.xp_curr.value - xp_prev,
                    xp_total: hero_info.xp_curr.value,
                    level: Some(LevelReward {
                        level: self.level,
                        passive_points: self.passive_skill_points,
                    }),
                });
            }

            // Hero is already at max level
            *hero_info.xp_curr.value.u64_mut() = curr_level_info.xp_end;

            return None;
        }

        Some(HeroReward {
            xp_gained: hero_info.xp_curr.value - xp_prev,
            xp_total: hero_info.xp_curr.value,
            level: None,
        })
    }

    pub fn can_run(&self) -> bool {
        *self.stats.vitals.stats["Ep"].value.u32() > 0
    }

    pub fn get_effective_movement_speed(&self) -> u32 {
        let base = *self.stats.vitals.stats["Movement"].value.u32();
        if self.can_run() {
            (base as f32 * 1.5).floor() as u32
        } else {
            base
        }
    }

    pub fn handle_death(
        &mut self,
        _attacker: &Self,
        metadata: &Metadata,
        rng: &mut Rng,
        next_uid: &mut NextUid,
    ) -> Option<Vec<Item>> {
        match self.kind {
            UnitKind::Villain => {
                let villain_info = self.info.villain();
                let villain_meta = metadata.unit.villains.get(&villain_info.id).unwrap();
                let items =
                    item::generation::roll_item_drops(metadata, villain_meta, rng, next_uid);

                // println!("generated {} item(s)", items.len());

                Some(items)
            }
            _ => None,
        }
    }

    fn block_attack(&self, _attacker: &Self, rng: &mut Rng) -> bool {
        let chance = self.stats.vitals.stats["BlockChance"].value.f32();
        rng.f32() <= *chance
    }

    fn dodge_attack(&self, _attacker: &Self, rng: &mut Rng) -> bool {
        let chance = self.stats.vitals.stats["DodgeChance"].value.f32();
        rng.f32() <= *chance
    }

    // Improve this,
    pub fn can_use_skill(
        &self,
        skills: &Vec<Skill>,
        metadata: &Metadata,
        skill_id: SkillId,
        target_distance: u32,
    ) -> SkillUseResult {
        let Some(skill) = skills.iter().find(|s| s.id == skill_id) else {
            return SkillUseResult::Error;
        };

        let skill_info = metadata.skill.skills.get(&skill.id).unwrap();

        if target_distance > skill_info.use_range {
            return SkillUseResult::OutOfRange;
        }

        let skill_cost = skill.get_skill_cost(metadata);

        // First ensure that the unit has enough resources to use the skill
        if let Some(hp_cost) = skill_cost.hp {
            // TODO Decide if skill use should be denied or allow the user to /wrists
            if self.stats.vitals.stats["Hp"].value < hp_cost {
                return SkillUseResult::InsufficientResources;
            }
        }

        if let Some(ep_cost) = skill_cost.ep {
            if self.stats.vitals.stats["Ep"].value < ep_cost {
                return SkillUseResult::InsufficientResources;
            }
        }

        if let Some(mp_cost) = skill_cost.mp {
            if self.stats.vitals.stats["Mp"].value < mp_cost {
                return SkillUseResult::InsufficientResources;
            }
        }

        SkillUseResult::Ok
    }

    pub fn use_skill(
        &mut self,
        skills: &mut Vec<Skill>,
        metadata: &Metadata,
        skill_id: SkillId,
        target_distance: u32,
    ) -> SkillUseResult {
        let request_result = self.can_use_skill(skills, metadata, skill_id, target_distance);
        if request_result != SkillUseResult::Ok {
            return request_result;
        }

        let Some(skill) = skills.iter().find(|s| s.id == skill_id) else {
            return SkillUseResult::Error;
        };

        let skill_cost = skill.get_skill_cost(metadata);

        // Deduct the required resources
        if let Some(hp_cost) = skill_cost.hp {
            self.stats.vitals.stats.get_mut("Hp").unwrap().value -= hp_cost;
        }

        if let Some(ep_cost) = skill_cost.ep {
            self.stats.vitals.stats.get_mut("Ep").unwrap().value -= ep_cost;
        }

        if let Some(mp_cost) = skill_cost.mp {
            self.stats.vitals.stats.get_mut("Mp").unwrap().value -= mp_cost;
        }

        SkillUseResult::Ok
    }
}

pub mod generation {
    use super::*;

    pub fn generate(
        rng: &mut Rng,
        metadata: &Metadata,
        next_uid: &mut NextUid,
        base_level: u8,
    ) -> Unit {
        let villain_id: VillainId = VillainId::sample(rng);

        let villain_table_entry = metadata.unit.villains.get(&villain_id).unwrap();
        let class = villain_table_entry.class;

        let uid = next_uid.get();
        next_uid.next();

        Unit::new(
            uid,
            class,
            UnitKind::Villain,
            UnitInfo::Villain(VillainInfo::new(villain_id)),
            base_level,
            villain_table_entry.name.to_string(),
            metadata,
        )
    }
}
