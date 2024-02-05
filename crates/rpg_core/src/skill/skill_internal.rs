use super::effect::EffectInfo;

use crate::{damage::DamageDescriptor, metadata::Metadata, stat::StatId, value::Value};

use glam::Vec3;
use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(PartialEq, Clone, Debug, Ser, De)]
pub struct TickableDescripor {
    pub duration: f32,
    pub frequency: f32,
}

#[derive(PartialEq, Clone, Debug, Ser, De)]
pub enum TimerDescriptor {
    Duration(f32),
    Tickable(TickableDescripor),
}

#[derive(Default, PartialEq, Clone, Copy, Debug, Ser, De)]
pub struct SkillSlotId(pub u8);

/// A skill slot
#[derive(Default, Debug, Copy, Clone, PartialEq, Ser, De)]
pub struct SkillSlot {
    pub id: SkillSlotId,
    pub skill_id: Option<SkillId>,
}

impl SkillSlot {
    pub fn new(id: SkillSlotId, skill_id: Option<SkillId>) -> Self {
        Self { id, skill_id }
    }
}

#[derive(Debug, Clone, PartialEq, Ser, De)]
pub struct SkillTarget {
    pub origin: Vec3,
    pub target: Vec3,
}

/// Where a skill originates from/
#[derive(Ser, De, Debug, Clone, PartialEq)]
pub enum OriginKind {
    /// The skill should be spawned from the owner's location
    Direct,
    /// The skill should be spawned in a remote location
    Remote,
    /// The skill should be locked to the owner's location
    Locked,
}

/// Skill identifiers
#[derive(Ser, De, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SkillId {
    BasicDirect,
    BasicBolt,
    BasicOrb,
    BasicAerial,
    AreaLocked,
    AreaRemote,
}

/// Skill use results
#[derive(Debug, PartialEq)]
pub enum SkillUseResult {
    Ok,
    OutOfRange,
    Error,
    InsufficientResources,
    Blocked,
}

#[derive(Ser, De, Default, PartialEq, Debug, Clone)]
pub struct DirectInfo {
    pub range: u32,
    pub frames: u32,
}

#[derive(Debug, Default, Clone)]
pub struct DirectInstance {
    pub info: DirectInfo,
    pub frame: u32,
}

#[derive(Ser, De, Default, PartialEq, Debug, Clone)]
pub struct AreaInfo {
    pub radius: u32,
}

#[derive(Debug, Default, Clone)]
pub struct AreaInstance {
    pub info: AreaInfo,
}

#[derive(Ser, De, Debug, Default, PartialEq, Clone)]
pub struct OrbitInfo {
    pub range: u32,
}

#[derive(Ser, De, Debug, Default, PartialEq, Clone)]
pub struct AerialInfo {}

#[derive(Copy, Clone, Debug, PartialEq, Ser, De)]
pub enum ProjectileShape {
    Sphere,
    Box,
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct ProjectileInfo {
    pub shape: ProjectileShape,
    pub projectiles: u32,
    pub speed: u32,
    pub size: u32,
    pub orbit: Option<OrbitInfo>,
    pub aerial: Option<AerialInfo>,
}

#[derive(Debug, Clone)]
pub struct OrbitData {
    pub origin: Vec3,
}

#[derive(Debug, Clone)]
pub struct ProjectileInstance {
    pub info: ProjectileInfo,
    pub orbit: Option<OrbitData>,
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub enum SkillInfo {
    Direct(DirectInfo),
    Projectile(ProjectileInfo),
    Area(AreaInfo),
}

#[derive(Debug, Clone)]
pub enum SkillInstance {
    Direct(DirectInstance),
    Projectile(ProjectileInstance),
    Area(AreaInstance),
}

#[derive(Debug, Clone)]
pub struct SkillCost {
    pub hp: Option<Value>,
    pub ep: Option<Value>,
    pub mp: Option<Value>,
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct Skill {
    pub id: SkillId,
    pub level: u8,
    pub damage: DamageDescriptor,
    pub info: SkillInfo,
    pub effects: Vec<EffectInfo>,
}

impl Skill {
    pub fn new(
        id: SkillId,
        level: u8,
        damage: DamageDescriptor,
        info: SkillInfo,
        effects: Vec<EffectInfo>,
    ) -> Self {
        Self {
            id,
            level,
            damage,
            info,
            effects,
        }
    }
}

impl Skill {
    pub fn get_skill_cost(&self, metadata: &Metadata) -> SkillCost {
        let skill_meta = &metadata.skill.skills[&self.id];

        let hp_cost = skill_meta.base_cost.iter().find(|c| c.id == StatId(4));
        let ep_cost = skill_meta.base_cost.iter().find(|c| c.id == StatId(8));
        let mp_cost = skill_meta.base_cost.iter().find(|c| c.id == StatId(12));

        let hp_cost = hp_cost.map(|cost| cost.value);
        let ep_cost = ep_cost.map(|cost| cost.value);
        let mp_cost = mp_cost.map(|cost| cost.value);

        SkillCost {
            hp: hp_cost,
            ep: ep_cost,
            mp: mp_cost,
        }
    }
}
