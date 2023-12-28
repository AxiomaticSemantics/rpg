use super::value::Value;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Default, Clone, Ser, De)]
pub struct ModifierList {
    pub add: Vec<Value>,
    pub sub: Vec<Value>,
    pub mul: Vec<Value>,
    pub div: Vec<Value>,
}
