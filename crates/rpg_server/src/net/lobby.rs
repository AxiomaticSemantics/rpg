use super::server::{ClientType, NetworkContext, NetworkParamsRO, NetworkParamsRW};
use crate::lobby::LobbyManager;

use rpg_lobby::lobby::Lobby;
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

pub(crate) fn receive_lobby_create(
    mut lobby_manager: ResMut<LobbyManager>,
    mut create_reader: EventReader<MessageEvent<CSLobbyCreate>>,
    mut net_params: NetworkParamsRW,
) {
    for event in create_reader.read() {
        info!("lobby create");

        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to create a lobby: {client:?}");
            continue;
        }
        let account_id = client.account_id.unwrap();
        let create_msg = event.message();

        if let Some(lobby_id) = lobby_manager.add_lobby("Default Test Lobby".into()) {
            lobby_manager.add_account(lobby_id, account_id);

            net_params.server.send_message_to_target::<Channel1, _>(
                SCLobbyCreateSuccess(Lobby {
                    id: lobby_id,
                    name: "Default Test Lobby".into(),
                    accounts: vec![account_id],
                }),
                NetworkTarget::Only(vec![*client_id]),
            );
        } else {
            net_params.server.send_message_to_target::<Channel1, _>(
                SCLobbyCreateError,
                NetworkTarget::Only(vec![*client_id]),
            );
        }
    }
}

pub(crate) fn receive_lobby_join(
    mut commands: Commands,
    mut lobby_manager: ResMut<LobbyManager>,
    mut join_reader: EventReader<MessageEvent<CSLobbyJoin>>,
    mut net_params: NetworkParamsRW,
) {
    for event in join_reader.read() {
        info!("lobby join");
        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to join a lobby: {client:?}");
            continue;
        }
        let account_id = client.account_id.unwrap();

        let join_msg = event.message();

        if let Some(lobby) = lobby_manager.get_lobby_mut(join_msg.0) {
            if lobby.add_account(account_id) {
                // TODO Handle rejections for banned accounts etc.
                net_params.server.send_message_to_target::<Channel1, _>(
                    SCLobbyJoinSuccess(lobby.clone()),
                    NetworkTarget::Only(vec![*client_id]),
                );
            }
        }
    }
}

pub(crate) fn receive_lobby_leave(
    mut commands: Commands,
    mut join_reader: EventReader<MessageEvent<CSLobbyLeave>>,
    mut net_params: NetworkParamsRW,
) {
    for event in join_reader.read() {
        info!("lobby leave");

        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to leave a lobby: {client:?}");
            continue;
        }

        // TODO Handle rejections for banned accounts etc.
        net_params.server.send_message_to_target::<Channel1, _>(
            SCLobbyLeaveSuccess,
            NetworkTarget::Only(vec![*client_id]),
        );
    }
}