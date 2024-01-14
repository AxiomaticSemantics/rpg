use lightyear::netcode::ClientId;

pub(crate) struct Lobby {
    pub(crate) name: String,
    pub(crate) id: u64,
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

    pub(crate) fn add_client(&mut self, id: ClientId) {
        if !self.clients.contains(&id) {
            self.clients.push(id);
        }
    }

    pub(crate) fn remove_client(&mut self, id: ClientId) {
        self.clients.retain(|c| *c != id);
    }
}
