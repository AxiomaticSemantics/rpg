use super::value::Value;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Hash, PartialEq, Eq, Ser, De)]
pub enum Affix {
    Prefix,
    Suffix,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialOrd, PartialEq, Ser, De)]
pub struct ModifierId(pub u16);

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ser, De)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ser, De)]
pub enum ModifierFormat {
    Percent,
    Flat,
}

impl ModifierFormat {
    pub fn is_percent(&self) -> bool {
        matches!(self, Self::Percent)
    }

    pub fn is_flat(&self) -> bool {
        matches!(self, Self::Flat)
    }
}

// TODO rename ModifierScope ??
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, PartialOrd, Ser, De)]
pub enum ModifierKind {
    Normal,
    Base,
    Global,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ser, De)]
pub struct Modifier {
    pub id: ModifierId,
    pub value: Value,
    pub operation: Operation,
    pub format: ModifierFormat,
    pub kind: ModifierKind,
}

impl Modifier {
    pub fn new(
        id: ModifierId,
        value: Value,
        operation: Operation,
        kind: ModifierKind,
        format: ModifierFormat,
    ) -> Self {
        Self {
            id,
            value,
            operation,
            kind,
            format,
        }
    }
}

pub fn format_modifier(modifier: &Modifier) -> String {
    let _kind_str = match &modifier.kind {
        ModifierKind::Normal => "",
        ModifierKind::Base => "Base",
        ModifierKind::Global => "Global",
    };

    let is_percent = match &modifier.format {
        ModifierFormat::Flat => false,
        ModifierFormat::Percent => true,
    };

    let operator_str = match &modifier.operation {
        Operation::Add | Operation::Mul => "+",
        Operation::Sub | Operation::Div => "-",
    };

    match &modifier.value {
        Value::U32(v) => {
            if is_percent {
                format!("{}{:2.1}%", operator_str, v)
            } else {
                format!("{}{}", operator_str, v)
            }
        }
        Value::U64(v) => {
            if is_percent {
                format!("{}{:2.1}%", operator_str, v)
            } else {
                format!("{}{}", operator_str, v)
            }
        }
        Value::F32(v) => {
            if is_percent {
                format!("{}{:2.1}%", operator_str, v * 100.)
            } else {
                format!("{}{}", operator_str, v)
            }
        }
        Value::F64(v) => {
            if is_percent {
                format!("{}{:2.1}%", operator_str, v * 100.)
            } else {
                format!("{}{}", operator_str, v)
            }
        }
    }
}
