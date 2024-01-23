use super::server::NetworkParamsRW;
use crate::{account::AccountInstance, chat::ChatManager};

use rpg_account::account::AccountId;
use rpg_chat::chat::{Channel, ChannelId, Message};
use rpg_network_protocol::protocol::*;

use bevy::{
    ecs::{
        event::EventReader,
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
};

use lightyear::{server::events::MessageEvent, shared::replication::components::NetworkTarget};

pub(crate) fn receive_chat_join(
    mut chat: ResMut<ChatManager>,
    mut join_reader: EventReader<MessageEvent<CSChatJoin>>,
    mut net_params: NetworkParamsRW,
    account_q: Query<&AccountInstance>,
) {
    for event in join_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to join chat: {client:?}");
            continue;
        }

        // TODO Handle rejections for banned accounts etc.

        // TODO For now add the client to a global channel

        for account in &account_q {
            if account.0.info.id != client.account_id.unwrap() {
                continue;
            }

            if chat.channel_exists(ChannelId(0)) {
                chat.add_subscriber(ChannelId(0), client.account_id.unwrap());

                net_params.server.send_message_to_target::<Channel1, _>(
                    SCChatJoinSuccess(0),
                    NetworkTarget::Only(vec![*client_id]),
                );
            }
        }
    }
}

pub(crate) fn receive_chat_leave(
    mut leave_reader: EventReader<MessageEvent<CSChatLeave>>,
    mut net_params: NetworkParamsRW,
) {
    for event in leave_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to leave chat: {client:?}");
            continue;
        }

        // TODO Handle rejections for banned accounts etc.
        net_params.server.send_message_to_target::<Channel1, _>(
            SCChatLeave,
            NetworkTarget::Only(vec![*client_id]),
        );
    }
}

pub(crate) fn receive_chat_channel_message(
    mut message_reader: EventReader<MessageEvent<CSChatChannelMessage>>,
    mut net_params: NetworkParamsRW,
    mut chat: ResMut<ChatManager>,
) {
    for event in message_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to message chat: {client:?}");
            continue;
        }

        let channel_msg = event.message();
        // TODO Handle rejections for banned accounts etc.

        let Some(channel) = chat.get_channel(channel_msg.0.channel_id) else {
            continue;
        };

        let subscriber_ids = channel
            .subscribers
            .iter()
            .map(|s| {
                let client = net_params
                    .context
                    .clients
                    .iter()
                    .find(|c| c.1.account_id.unwrap() == *s)
                    .unwrap();

                *client.0
            })
            .collect();

        net_params.server.send_message_to_target::<Channel1, _>(
            SCChatMessage(Message {
                channel_id: channel_msg.0.channel_id,
                id: channel_msg.0.id,
                sender: channel_msg.0.sender.clone(),
                message: channel_msg.0.message.clone(),
            }),
            NetworkTarget::Only(subscriber_ids),
        );
    }
}
