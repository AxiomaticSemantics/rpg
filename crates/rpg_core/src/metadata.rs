use crate::{
    item_tables::ItemTable,
    level_tables::LevelTable,
    passive_tree::PassiveTreeTable,
    skill::skill_tables::SkillTable,
    stat::{modifier_tables::ModifierTable, stat_tables::StatTable},
    unit_tables::UnitTable,
};

pub struct Metadata {
    pub item: ItemTable,
    pub unit: UnitTable,
    pub skill: SkillTable,
    pub level: LevelTable,
    pub stat: StatTable,
    pub modifier: ModifierTable,
    pub passive_tree: PassiveTreeTable,
}
