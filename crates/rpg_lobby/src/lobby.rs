use rpg_account::account::AccountId;
use rpg_chat::chat::MessageId;
use rpg_core::unit::HeroGameMode;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LobbyId(pub u64);

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct LobbyPlayer {
    pub account_id: AccountId,
    pub account_name: String,
}

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
    pub players: Vec<LobbyPlayer>,
    pub messages: Vec<LobbyMessage>,
}

impl Lobby {
    pub fn new(id: LobbyId, name: String, game_mode: HeroGameMode) -> Self {
        Self {
            id,
            name,
            game_mode,
            players: vec![],
            messages: vec![],
        }
    }

    // This is a destructive action
    pub fn clear(&mut self) {
        self.players.clear();
    }

    pub fn has_player(&self, account_id: AccountId) -> bool {
        self.players.iter().any(|a| a.account_id == account_id)
    }

    pub fn add_player(&mut self, player: LobbyPlayer) -> bool {
        if !self.has_player(player.account_id) {
            self.players.push(player);

            true
        } else {
            false
        }
    }

    pub fn remove_player(&mut self, id: AccountId) {
        self.players.retain(|p| p.account_id != id);
    }
}
