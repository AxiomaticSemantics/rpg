use super::modifier_list::ModifierList;
use crate::value::{Value, ValueKind};

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Debug, Clone)]
pub struct StatList {
    pub value_kind: ValueKind,
    pub modifiers: ModifierList,
    pub add_sum: Value,
    pub mul_sum: Value,
}

impl StatList {
    pub fn new(value_kind: ValueKind) -> Self {
        let add_sum = Value::zero(value_kind);
        let mul_sum = Value::zero(value_kind);

        Self {
            value_kind,
            modifiers: ModifierList::default(),
            add_sum,
            mul_sum,
        }
    }

    pub fn compute_sum(&mut self) {
        self.add_sum = self
            .modifiers
            .add
            .iter()
            .fold(Value::zero(self.value_kind), |s, v| s + *v)
            - self
                .modifiers
                .sub
                .iter()
                .fold(Value::zero(self.value_kind), |s, v| s + *v);

        self.mul_sum = self
            .modifiers
            .mul
            .iter()
            .fold(Value::zero(self.value_kind), |s, v| s + *v)
            - self
                .modifiers
                .div
                .iter()
                .fold(Value::zero(self.value_kind), |s, v| s + *v);
    }
}
