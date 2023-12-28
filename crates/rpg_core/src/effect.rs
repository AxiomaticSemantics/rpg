use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, Debug, Default, Copy, Clone, PartialEq, De)]
pub struct EffectCounter {
    pub curr: u8,
    pub max: u8,
}

impl EffectCounter {
    pub fn new(curr: u8, max: u8) -> Self {
        Self { curr, max }
    }

    pub fn increment(&mut self) {
        assert!(self.curr < self.max);

        self.curr += 1;
    }

    pub const fn is_max(&self) -> bool {
        self.curr == self.max
    }
}
