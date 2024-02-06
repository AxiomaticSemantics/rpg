use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Default, PartialEq, PartialOrd, Copy, Clone, Ser, De)]
pub enum GameMode {
    #[default]
    Normal,
    Hardcore,
}
