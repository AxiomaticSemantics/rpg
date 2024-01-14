use rpg_account::account::AccountId;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Copy, Clone, PartialEq, Debug)]
pub struct MessageId(pub u64);

#[derive(Ser, De, Copy, Clone, PartialEq, Debug)]
pub struct ChannelId(pub u64);

#[derive(Ser, De, PartialEq, Debug)]
pub struct Message {
    pub message: String,
    pub id: MessageId,
    pub sender: AccountId,
}

#[derive(Ser, De, PartialEq, Debug)]
pub struct Channel {
    pub clients: Vec<AccountId>,
    pub messages: Vec<Message>,
    pub recent_message_ids: Vec<MessageId>,
}

impl Channel {
    pub fn new() -> Self {
        Self {
            clients: vec![],
            messages: vec![],
            recent_message_ids: vec![],
        }
    }

    pub fn add_client(&mut self, id: AccountId) {
        if !self.clients.contains(&id) {
            self.clients.push(id);
        }
    }

    pub fn remove_client(&mut self, id: AccountId) {
        self.clients.retain(|c| *c != id);
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn remove_message(&mut self, message_id: MessageId) {
        // TODO optimize eventually
        self.messages.retain(|m| m.id != message_id);
    }
}
