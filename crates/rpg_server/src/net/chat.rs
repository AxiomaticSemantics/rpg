use super::server::{
    AuthorizationStatus, ClientType, NetworkContext, NetworkParamsRO, NetworkParamsRW,
};

use rpg_network_protocol::protocol::*;

use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        event::EventReader,
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
    prelude::{Deref, DerefMut},
};

use lightyear::{server::events::MessageEvent, shared::replication::components::NetworkTarget};

pub(crate) fn receive_chat_join(
    mut commands: Commands,
    mut join_reader: EventReader<MessageEvent<CSChatJoin>>,
    mut net_params: NetworkParamsRW,
) {
    for event in join_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to join chat: {client:?}");
            continue;
        }

        // TODO Handle rejections for banned accounts etc.
        net_params.server.send_message_to_target::<Channel1, _>(
            SCChatJoinSuccess(0),
            NetworkTarget::Only(vec![*client_id]),
        );
    }
}

pub(crate) fn receive_chat_leave(
    mut commands: Commands,
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
    mut commands: Commands,
    mut message_reader: EventReader<MessageEvent<CSChatChannelMessage>>,
    mut net_params: NetworkParamsRW,
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

        net_params.server.send_message_to_target::<Channel1, _>(
            SCChatMessage {
                channel_id: channel_msg.channel_id,
                message_id: channel_msg.message_id,
                message: channel_msg.message.clone(),
            },
            NetworkTarget::Only(vec![*client_id]),
        );
    }
}
