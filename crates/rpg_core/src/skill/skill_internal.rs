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

#[derive(Default, PartialEq, Clone, Debug, Ser, De)]
pub struct DirectOrigin {
    pub offset: Vec3,
}

#[derive(Default, PartialEq, Clone, Debug, Ser, De)]
pub struct RemoteOrigin {
    pub offset: Vec3,
}

#[derive(Default, PartialEq, Clone, Debug, Ser, De)]
pub struct LockedOrigin {
    pub offset: Vec3,
}

/// Where a skill originates from/
#[derive(Ser, De, Debug, Clone, PartialEq)]
pub enum Origin {
    /// The skill should be spawned from the owner's location
    Direct(DirectOrigin),
    /// The skill should be spawned in a remote location
    Remote(RemoteOrigin),
    /// The skill should be locked to the owner's location
    Locked(LockedOrigin),
}

impl Origin {
    pub fn is_direct(&self) -> bool {
        matches!(self, Self::Direct(_))
    }

    pub fn is_remote(&self) -> bool {
        matches!(self, Self::Remote(_))
    }

    pub fn is_locked(&self) -> bool {
        matches!(self, Self::Locked(_))
    }
}

/// Skill slot identifiers
#[derive(Ser, De, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum SkillSlotId {
    #[default]
    Primary,
    Secondary,
}

/// A skill slot
#[derive(Ser, De, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SkillSlot {
    pub id: SkillSlotId,
    pub skill: Option<SkillId>,
}

/// Skill slots
#[derive(Ser, De, Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ActiveSkills {
    pub primary: SkillSlot,
    pub secondary: SkillSlot,
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

#[derive(Ser, De, Default, Debug, Clone)]
pub struct DirectInfo {
    pub range: u32,
    pub frames: u32,
}

#[derive(Debug, Default, Clone)]
pub struct DirectInstance {
    pub info: DirectInfo,
    pub frame: u32,
}

#[derive(Ser, De, Default, Debug, Clone)]
pub struct AreaInfo {
    pub radius: u32,
    pub tick_rate: Option<f32>,
}

#[derive(Debug, Default, Clone)]
pub struct AreaInstance {
    pub info: AreaInfo,
}

#[derive(Ser, De, Debug, Default, Clone)]
pub struct OrbitInfo {
    pub range: u32,
}

#[derive(Ser, De, Debug, Default, Clone)]
pub struct AerialInfo {
    pub height: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Ser, De)]
pub enum ProjectileShape {
    Sphere,
    Box,
}

#[derive(Ser, De, Debug, Clone)]
pub struct ProjectileInfo {
    pub shape: ProjectileShape,
    pub projectiles: u32,
    pub speed: u32,
    pub size: u32,
    pub orbit: Option<OrbitInfo>,
    pub aerial: Option<AerialInfo>,
    pub tick_rate: Option<f32>,
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

#[derive(Ser, De, Debug, Clone)]
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

#[derive(Ser, De, Debug, Clone)]
pub struct Skill {
    pub id: SkillId,
    pub level: u8,
    pub damage: DamageDescriptor,
    pub info: SkillInfo,
    pub origin: Origin,
    pub effects: Vec<EffectInfo>,
}

impl Skill {
    pub fn new(
        id: SkillId,
        level: u8,
        damage: DamageDescriptor,
        info: SkillInfo,
        origin: Origin,
        effects: Vec<EffectInfo>,
    ) -> Self {
        Self {
            id,
            level,
            damage,
            info,
            origin,
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
