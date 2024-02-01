use serde_derive::{Deserialize as De, Serialize as Ser};

use super::modifier::{self, Modifier};
use crate::value::{Value, ValueKind};

use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Hash, Ser, De)]
pub struct StatId(pub u16);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ser, De)]
pub struct StatDescriptor {
    pub id: StatId,
    pub name: Cow<'static, str>,
    pub value_kind: ValueKind,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ser, De)]
pub struct Stat {
    pub id: StatId,
    pub value: Value,
}

impl fmt::Display for Stat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} {}", self.id, self.value)
    }
}

impl Stat {
    pub const fn new(id: StatId, value: Value) -> Self {
        Self { id, value }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ser, De)]
pub struct StatModifier {
    pub id: StatId,
    pub modifier: Modifier,
}

impl StatModifier {
    pub fn new(id: StatId, modifier: Modifier) -> Self {
        Self { id, modifier }
    }
}

impl fmt::Display for StatModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", modifier::format_modifier(&self.modifier),)
    }
}

#[derive(Ser, De, PartialEq, Debug, Clone)]
pub enum StatChange {
    Gain(Value),
    Loss(Value),
}

#[derive(Ser, De, PartialEq, Debug, Clone)]
pub struct StatUpdate {
    pub id: StatId,
    pub total: Value,
    pub change: StatChange,
}
