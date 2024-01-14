use rpg_account::account::AccountId;
use rpg_chat::chat::{Channel, ChannelId, MessageId};

use bevy::ecs::system::Resource;

use lightyear::netcode::ClientId;

use std::collections::HashMap;

#[derive(Default, Resource)]
pub(crate) struct Chat {
    channels: HashMap<ChannelId, Channel>,
}

impl Chat {
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
