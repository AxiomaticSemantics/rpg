use crate::server::{
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

use lightyear::server::events::MessageEvent;

pub(crate) fn receive_chat_join(
    mut commands: Commands,
    mut join_reader: EventReader<MessageEvent<CSChatJoin>>,
    mut net_params: NetworkParamsRW,
) {
    for event in join_reader.read() {
        let client = net_params.context.clients.get(event.context()).unwrap();
        if client.is_authenticated_player() {
            info!("already authenticated client attempted to create account {client:?}");
            continue;
        }

        // Allow authenticated admins to create accounts
        if client.is_authenticated_admin() {}
    }
}
