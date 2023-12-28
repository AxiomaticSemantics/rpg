use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Debug, Clone)]
pub enum DamageKind {
    Physical,
    Magic,
    Toxic,
}

#[derive(Ser, De, Debug, Clone)]
pub enum DamageValue {
    Flat(u32),
    MinMax(u32, u32),
}

#[derive(Ser, De, Debug, Clone)]
pub struct DamageInfo {
    pub value: DamageValue,
}

impl DamageInfo {
    pub const fn new(value: DamageValue) -> Self {
        Self { value }
    }
}

#[derive(Ser, De, Debug, Clone)]
pub struct Damage {
    pub kind: DamageKind,
    pub info: DamageInfo,
}

impl Damage {
    pub const fn new(kind: DamageKind, info: DamageInfo) -> Self {
        Self { kind, info }
    }
}
