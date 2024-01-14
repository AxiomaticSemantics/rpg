use lightyear::netcode::ClientId;

pub(crate) struct ChatMessage {
    pub(crate) message: String,
    pub(crate) id: u64,
}

pub(crate) struct ChatRoom {
    pub(crate) clients: Vec<ClientId>,
    pub(crate) messages: Vec<ChatMessage>,
    pub(crate) recent_message_ids: Vec<u64>,
}

impl ChatRoom {
    pub(crate) fn new() -> Self {
        Self {
            clients: vec![],
            messages: vec![],
            recent_message_ids: vec![],
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

    pub(crate) fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    pub(crate) fn remove_message(&mut self, message_id: u64) {
        // TODO optimize eventually
        self.messages.retain(|m| m.id != message_id);
    }
}
