use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Debug, Ser, De)]
pub struct AccountCharacter {
    pub name: String,
}

#[derive(Debug, Ser, De)]
pub struct Account {
    pub name: String,
    pub characters: Vec<AccountCharacter>,
}
