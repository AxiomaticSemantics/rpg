use rpg_account::account::AccountId;
use rpg_chat::chat::{Channel, ChannelId, MessageId};

use bevy::ecs::system::{Res, ResMut, Resource};

use lightyear::netcode::ClientId;

use std::collections::HashMap;

#[derive(Default, Resource)]
pub(crate) struct Chat {
    pub(crate) channels: HashMap<ChannelId, Channel>,
    pub(crate) next_channel_id: ChannelId,
}

impl Chat {
    pub(crate) fn new() -> Self {
        Self {
            channels: HashMap::default(),
            next_channel_id: ChannelId(1),
        }
    }

    pub(crate) fn channel_exists(&self, channel_id: ChannelId) -> bool {
        self.channels.contains_key(&channel_id)
    }

    pub(crate) fn get_channel(&self, id: ChannelId) -> Option<&Channel> {
        self.channels.get(&id)
    }

    pub(crate) fn get_channel_mut(&mut self, id: ChannelId) -> Option<&mut Channel> {
        self.channels.get_mut(&id)
    }

    pub(crate) fn add_channel(&mut self, channel: Channel) {
        self.channels.insert(channel.id, channel);
    }

    pub(crate) fn remove_channel(&mut self, channel_id: ChannelId) {
        self.channels.retain(|c, _| *c != channel_id);
    }

    pub(crate) fn add_subscriber(&mut self, channel_id: ChannelId, account_id: AccountId) {
        if let Some(channel) = self.channels.get_mut(&channel_id) {
            channel.add_subscriber(account_id);
        }
    }

    pub(crate) fn remove_subscriber(&mut self, channel_id: ChannelId, account_id: AccountId) {
        if let Some(channel) = self.channels.get_mut(&channel_id) {
            channel.remove_subscriber(account_id);
        }
    }
}

pub(crate) fn setup(mut chat: ResMut<Chat>) {
    let default_channel = Channel::new("Default".into(), ChannelId(0));
    chat.add_channel(default_channel);
}
