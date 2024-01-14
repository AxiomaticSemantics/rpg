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
