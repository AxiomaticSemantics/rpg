use fastrand::Rng;
use serde_derive::{Deserialize as De, Serialize as Ser};

use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(
    Default, FromPrimitive, IntoPrimitive, Debug, Copy, Clone, PartialEq, Eq, Hash, Ser, De,
)]
#[repr(u16)]
pub enum VillainId {
    #[default]
    Wizard,
    Swordsman,
    Archer,
    Cleric,
}

impl VillainId {
    pub fn sample(rng: &mut Rng) -> VillainId {
        rng.u16(0..Self::Cleric.into()).into()
    }
}
