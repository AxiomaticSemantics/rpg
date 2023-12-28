use glam::Vec3;
use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct PierceEffect {
    pub pierces: u8,
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct ChainEffect {
    pub chains: u8,
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct SplitEffect {
    pub splits: u8,
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct DotEffect {
    pub frequency: f32,
    pub ticks: u8,
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct KnockbackEffect {
    pub speed: u32,
    pub duration: f32,
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub enum EffectInfo {
    Pierce(PierceEffect),
    Split(SplitEffect),
    Chain(ChainEffect),
    Dot(DotEffect),
    Knockback(KnockbackEffect),
}

impl EffectInfo {
    pub fn is_pierce(&self) -> bool {
        matches!(self, Self::Pierce(_))
    }

    pub fn is_split(&self) -> bool {
        matches!(self, Self::Split(_))
    }

    pub fn is_chain(&self) -> bool {
        matches!(self, Self::Chain(_))
    }

    pub fn is_dot(&self) -> bool {
        matches!(self, Self::Dot(_))
    }

    pub fn is_knockback(&self) -> bool {
        matches!(self, Self::Knockback(_))
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct PierceData {
    pub count: u8,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct ChainData {
    pub direction: Vec3,
    pub count: u8,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct SplitData {
    pub count: u8,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct DotData {
    pub count: u8,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct KnockbackData {
    pub count: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EffectData {
    Pierce(PierceData),
    Chain(ChainData),
    Split(SplitData),
    Dot(DotData),
    Knockback(KnockbackData),
}

#[derive(Debug, Clone)]
pub struct EffectInstance {
    pub info: EffectInfo,
    pub data: EffectData,
}

impl EffectInstance {
    pub fn new(info: EffectInfo, data: EffectData) -> Self {
        Self { info, data }
    }
}
