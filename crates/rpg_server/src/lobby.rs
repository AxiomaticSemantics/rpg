use rpg_account::account::AccountId;
use rpg_core::game_mode::GameMode;
use rpg_lobby::lobby::{Lobby, LobbyId, LobbyPlayer};

use bevy::ecs::system::Resource;

#[derive(Default, Resource)]
pub(crate) struct LobbyManager {
    pub(crate) lobbies: Vec<Lobby>,
    pub(crate) next_lobby_id: LobbyId,
}

impl LobbyManager {
    pub(crate) fn add_lobby(&mut self, name: String, game_mode: GameMode) -> Option<LobbyId> {
        let id = self.next_lobby_id;
        if !self.lobbies.iter().any(|l| l.id == id) {
            let lobby = Lobby::new(self.next_lobby_id, name, game_mode);
            self.lobbies.push(lobby);

            self.next_lobby_id.0 += 1;

            Some(id)
        } else {
            None
        }
    }

    pub(crate) fn remove_lobby(&mut self, id: LobbyId) {
        self.lobbies.retain(|l| l.id != id);
    }

    pub(crate) fn has_lobby(&self, id: LobbyId) -> bool {
        self.lobbies.iter().any(|l| l.id == id)
    }

    pub(crate) fn get_lobby(&self, id: LobbyId) -> Option<&Lobby> {
        self.lobbies.iter().find(|l| l.id == id)
    }

    pub(crate) fn get_lobby_mut(&mut self, id: LobbyId) -> Option<&mut Lobby> {
        self.lobbies.iter_mut().find(|l| l.id == id)
    }

    pub(crate) fn has_player(&self, id: LobbyId, account_id: AccountId) -> bool {
        if let Some(lobby) = self.lobbies.iter().find(|l| l.id == id) {
            lobby.has_player(account_id)
        } else {
            false
        }
    }

    pub(crate) fn add_player(&mut self, id: LobbyId, account_id: AccountId, name: String) -> bool {
        if let Some(lobby) = self.lobbies.iter_mut().find(|l| l.id == id) {
            lobby.add_player(LobbyPlayer {
                account_id,
                account_name: name,
            })
        } else {
            false
        }
    }

    pub(crate) fn remove_player(&mut self, id: LobbyId, account_id: AccountId) {
        if let Some(lobby) = self.lobbies.iter_mut().find(|l| l.id == id) {
            lobby.remove_player(account_id);
        }
    }
}
