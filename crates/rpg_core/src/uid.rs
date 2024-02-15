use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Default, PartialEq, PartialOrd, Debug, Copy, Clone)]
pub struct Uid(u64);

impl Uid {
    pub fn id(&self) -> u64 {
        self.0
    }
}

#[derive(Ser, De, Default, PartialEq, PartialOrd, Debug, Copy, Clone)]
pub struct InstanceUid(u32);

impl InstanceUid {
    pub fn id(&self) -> u32 {
        self.0
    }
}

#[derive(Ser, De, Default, PartialEq, PartialOrd, Debug)]
pub struct NextUid(Uid);

impl NextUid {
    pub fn next(&mut self) {
        self.0 .0 += 1;
    }

    pub fn get(&self) -> Uid {
        self.0
    }
}

#[derive(Ser, De, Default, PartialEq, PartialOrd, Debug)]
pub struct NextInstanceUid(InstanceUid);

impl NextInstanceUid {
    pub fn next(&mut self) {
        self.0 .0 += 1;
    }

    pub fn get(&self) -> InstanceUid {
        self.0
    }
}
