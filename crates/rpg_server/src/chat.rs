use rpg_account::account::AccountId;
use rpg_chat::chat::{Channel, ChannelId, MessageId};

use bevy::ecs::system::Resource;

use lightyear::netcode::ClientId;

use std::collections::HashMap;

#[derive(Default, Resource)]
pub(crate) struct Chat {
    rooms: HashMap<ChannelId, Channel>,
}

impl Chat {}
