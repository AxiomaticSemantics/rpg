use std::fmt;

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

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str_ref = match self {
            Self::Str => "Warrior",
            Self::Dex => "Ranger",
            Self::Int => "Wizard",
            Self::StrDex => "Duelist",
            Self::DexInt => "Necromancer",
            Self::IntStr => "Cleric",
            Self::StrDexInt => "Rogue",
        };

        write!(f, "{}", str_ref)
    }
}
