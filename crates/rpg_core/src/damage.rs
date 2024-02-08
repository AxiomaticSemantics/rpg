use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, PartialEq, Debug, Copy, Clone)]
pub enum DamageKind {
    Physical,
    Magic,
    Toxic,
}

#[derive(Ser, De, Debug, PartialEq, Copy, Clone)]
pub enum DamageValueDescriptor {
    Flat(u32),
    MinMax(u32, u32),
}

#[derive(Ser, De, Debug, PartialEq, Copy, Clone)]
pub struct DamageDescriptor {
    pub kind: DamageKind,
    pub value: DamageValueDescriptor,
}

#[derive(Ser, De, Debug, PartialEq, Clone)]
pub struct Damage {
    pub kind: DamageKind,
    pub amount: u32,
}

impl Damage {
    pub const fn new(kind: DamageKind, amount: u32) -> Self {
        Self { kind, amount }
    }
}
