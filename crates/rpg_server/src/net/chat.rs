use super::server::{ClientMessageEvent, NetworkParamsRW};
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

pub(crate) fn receive_chat_join(
    mut chat: ResMut<ChatManager>,
    mut join_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
    account_q: Query<&AccountInstance>,
) {
    for event in join_reader.read() {
        let ClientMessage::CSChatJoin(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get(&client_id).unwrap();
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

                let message =
                    bincode::serialize(&ServerMessage::SCChatJoinSuccess(SCChatJoinSuccess(0)))
                        .unwrap();
                net_params
                    .server
                    .send_message(client_id, ServerChannel::Message, message);
            }
        }
    }
}

pub(crate) fn receive_chat_leave(
    mut leave_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
) {
    for event in leave_reader.read() {
        let ClientMessage::CSChatLeave(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to leave chat: {client:?}");
            continue;
        }

        // TODO Handle rejections for banned accounts etc.
        let message = bincode::serialize(&ServerMessage::SCChatLeave(SCChatLeave)).unwrap();
        net_params
            .server
            .send_message(client_id, ServerChannel::Message, message);
    }
}

pub(crate) fn receive_chat_channel_message(
    mut message_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
    mut chat: ResMut<ChatManager>,
) {
    for event in message_reader.read() {
        let ClientMessage::CSChatChannelMessage(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to message chat: {client:?}");
            continue;
        }

        // TODO Handle rejections for banned accounts etc.
        let Some(channel) = chat.get_channel(msg.0.channel_id) else {
            continue;
        };

        let subscriber_ids: Vec<_> = channel
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

        let message = bincode::serialize(&ServerMessage::SCChatMessage(SCChatMessage(Message {
            channel_id: msg.0.channel_id,
            id: msg.0.id,
            sender: msg.0.sender.clone(),
            message: msg.0.message.clone(),
        })))
        .unwrap();
        for client_id in subscriber_ids {
            net_params
                .server
                .send_message(client_id, ServerChannel::Message, message.clone());
        }
    }
}
