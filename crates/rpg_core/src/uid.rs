use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Default, PartialEq, Debug, Copy, Clone)]
pub struct Uid(pub u64);

#[derive(Ser, De, Default, PartialEq, Debug)]
pub struct NextUid(pub Uid);

impl NextUid {
    pub fn next(&mut self) {
        self.0 .0 += 1;
    }
}
