use rpg_account::account::AccountId;
use rpg_chat::chat::MessageId;
use rpg_core::unit::HeroGameMode;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LobbyId(pub u64);

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct LobbyMessage {
    pub id: MessageId,
    pub sender_id: AccountId,
    pub sender: String,
    pub message: String,
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct Lobby {
    pub id: LobbyId,
    pub name: String,
    pub game_mode: HeroGameMode,
    pub accounts: Vec<AccountId>,
    pub messages: Vec<LobbyMessage>,
}

impl Lobby {
    pub fn new(id: LobbyId, name: String, game_mode: HeroGameMode) -> Self {
        Self {
            id,
            name,
            game_mode,
            accounts: vec![],
            messages: vec![],
        }
    }

    pub fn clear(&mut self) {
        self.accounts.clear();
    }

    pub fn has_account(&self, account_id: AccountId) -> bool {
        self.accounts.iter().any(|a| *a == account_id)
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
