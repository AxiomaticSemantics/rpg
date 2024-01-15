use bevy::ecs::system::Resource;

use lightyear::netcode::ClientId;

#[derive(Default, Resource)]
pub(crate) struct LobbyManager {
    pub(crate) lobbies: Vec<Lobby>,
    pub(crate) next_lobby_id: u64,
}

impl LobbyManager {
    pub(crate) fn add_lobby(&mut self, name: String) -> Option<u64> {
        let id = self.next_lobby_id;
        if !self.lobbies.iter().any(|l| l.id == id) {
            let lobby = Lobby::new(self.next_lobby_id, name);
            self.lobbies.push(lobby);

            self.next_lobby_id += 1;

            Some(id)
        } else {
            None
        }
    }

    pub(crate) fn remove_lobby(&mut self, id: u64) {
        self.lobbies.retain(|l| l.id != id);
    }

    pub(crate) fn add_client(&mut self, id: u64, client_id: ClientId) -> bool {
        if let Some(lobby) = self.lobbies.iter_mut().find(|l| l.id == id) {
            lobby.add_client(client_id)
        } else {
            false
        }
    }
}

pub(crate) struct Lobby {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) clients: Vec<ClientId>,
}

impl Lobby {
    pub(crate) fn new(id: ClientId, name: String) -> Self {
        Self {
            id,
            name,
            clients: vec![],
        }
    }

    pub(crate) fn clear(&mut self) {
        self.clients.clear();
    }

    pub(crate) fn add_client(&mut self, id: ClientId) -> bool {
        if !self.clients.contains(&id) {
            self.clients.push(id);

            true
        } else {
            false
        }
    }

    pub(crate) fn remove_client(&mut self, id: ClientId) {
        self.clients.retain(|c| *c != id);
    }
}
