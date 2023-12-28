use fastrand::Rng;

use num_enum::{FromPrimitive, IntoPrimitive};
use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(
    FromPrimitive, IntoPrimitive, Debug, Default, Copy, Clone, PartialEq, Eq, Hash, Ser, De,
)]
#[repr(u8)]
pub enum Class {
    #[default]
    Str,
    Dex,
    Int,
    StrDex,
    DexInt,
    IntStr,
    StrDexInt,
}

impl Class {
    fn sample(&self, rng: &mut Rng) -> Class {
        rng.u8(0..=Self::StrDexInt.into()).into()
    }
}
