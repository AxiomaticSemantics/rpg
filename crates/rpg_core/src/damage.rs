use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, PartialEq, Debug, Copy, Clone)]
pub enum DamageKind {
    Physical,
    Magic,
    Toxic,
}

#[derive(Ser, De, Debug, PartialEq, Copy, Clone)]
pub enum DamageValue {
    Flat(u32),
    MinMax(u32, u32),
}

#[derive(Ser, De, Debug, PartialEq, Clone)]
pub struct Damage {
    pub kind: DamageKind,
    pub value: DamageValue,
}

impl Damage {
    pub const fn new(kind: DamageKind, value: DamageValue) -> Self {
        Self { kind, value }
    }
}
