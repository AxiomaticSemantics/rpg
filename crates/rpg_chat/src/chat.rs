use rpg_account::account::AccountId;

use serde_derive::{Deserialize as De, Serialize as Ser};

#[derive(Ser, De, Default, Hash, Copy, Clone, PartialEq, Eq, Debug)]
pub struct MessageId(pub u64);

#[derive(Ser, De, Default, Hash, Copy, Clone, PartialEq, Eq, Debug)]
pub struct ChannelId(pub u64);

#[derive(Ser, De, Clone, PartialEq, Debug)]
pub struct Message {
    pub message: String,
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub sender: AccountId,
}

#[derive(Ser, De, PartialEq, Debug)]
pub struct Channel {
    pub name: String,
    pub id: ChannelId,
    pub subscribers: Vec<AccountId>,
    pub messages: Vec<Message>,
    pub recent_message_ids: Vec<MessageId>,
}

impl Channel {
    pub fn new(name: String, id: ChannelId) -> Self {
        Self {
            name,
            id,
            subscribers: vec![],
            messages: vec![],
            recent_message_ids: vec![],
        }
    }

    pub fn add_subscriber(&mut self, id: AccountId) {
        if !self.subscribers.contains(&id) {
            self.subscribers.push(id);
        }
    }

    pub fn remove_subscriber(&mut self, id: AccountId) {
        self.subscribers.retain(|c| *c != id);
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn remove_message(&mut self, message_id: MessageId) {
        // TODO optimize eventually
        self.messages.retain(|m| m.id != message_id);
    }
}
