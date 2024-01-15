use rpg_account::account::AccountId;
use rpg_lobby::lobby::{Lobby, LobbyId};

use bevy::ecs::system::Resource;

use lightyear::netcode::ClientId;

#[derive(Default, Resource)]
pub(crate) struct LobbyManager {
    pub(crate) lobbies: Vec<Lobby>,
    pub(crate) next_lobby_id: LobbyId,
}

impl LobbyManager {
    pub(crate) fn add_lobby(&mut self, name: String) -> Option<LobbyId> {
        let id = self.next_lobby_id;
        if !self.lobbies.iter().any(|l| l.id == id) {
            let lobby = Lobby::new(self.next_lobby_id, name);
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

    pub(crate) fn add_account(&mut self, id: LobbyId, account_id: AccountId) -> bool {
        if let Some(lobby) = self.lobbies.iter_mut().find(|l| l.id == id) {
            lobby.add_account(account_id)
        } else {
            false
        }
    }
}
