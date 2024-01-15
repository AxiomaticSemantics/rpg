use rpg_account::account::AccountId;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LobbyId(pub u64);

#[derive(Ser, De, Default, Clone, PartialEq)]
pub struct Lobby {
    pub id: LobbyId,
    pub name: String,
    pub accounts: Vec<AccountId>,
}

impl Lobby {
    pub fn new(id: LobbyId, name: String) -> Self {
        Self {
            id,
            name,
            accounts: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.accounts.clear();
    }

    pub fn add_account(&mut self, id: AccountId) -> bool {
        if !self.accounts.contains(&id) {
            self.accounts.push(id);

            true
        } else {
            false
        }
    }

    pub fn remove_account(&mut self, id: AccountId) {
        self.accounts.retain(|c| *c != id);
    }
}
