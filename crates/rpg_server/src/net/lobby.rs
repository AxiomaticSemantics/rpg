use super::server::NetworkParamsRW;
use crate::{account::AccountInstance, lobby::LobbyManager, server_state::ServerMetadataResource};

use rpg_chat::chat::MessageId;
use rpg_lobby::lobby::{Lobby, LobbyId, LobbyMessage, LobbyPlayer};
use rpg_network_protocol::protocol::*;

use bevy::{
    ecs::{
        event::EventReader,
        system::{Query, Res, ResMut},
    },
    log::info,
};

use lightyear::{server::events::MessageEvent, shared::NetworkTarget};

pub(crate) fn receive_lobby_create(
    mut lobby_manager: ResMut<LobbyManager>,
    mut create_reader: EventReader<MessageEvent<CSLobbyCreate>>,
    mut net_params: NetworkParamsRW,
    account_q: Query<&AccountInstance>,
) {
    for event in create_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to create a lobby: {client:?}");
            continue;
        }
        let account_id = client.account_id.unwrap();
        let account = account_q.get(client.entity).unwrap();

        let create_msg = event.message();

        if let Some(lobby_id) =
            lobby_manager.add_lobby(create_msg.name.clone(), create_msg.game_mode)
        {
            info!("lobby created");

            lobby_manager.add_player(lobby_id, account_id, account.info.name.clone());

            net_params.server.send_message_to_target::<Channel1, _>(
                SCLobbyCreateSuccess(Lobby {
                    id: lobby_id,
                    name: create_msg.name.clone(),
                    game_mode: create_msg.game_mode,
                    messages: vec![],
                    players: vec![LobbyPlayer {
                        account_id,
                        account_name: account.info.name.clone(),
                    }],
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
    mut lobby_manager: ResMut<LobbyManager>,
    mut join_reader: EventReader<MessageEvent<CSLobbyJoin>>,
    mut net_params: NetworkParamsRW,
    account_q: Query<&AccountInstance>,
) {
    for event in join_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to join a lobby: {client:?}");
            continue;
        }
        let account_id = client.account_id.unwrap();
        let account = account_q.get(client.entity).unwrap();

        let join_msg = event.message();

        if let Some(lobby) = lobby_manager.get_lobby_mut(join_msg.0) {
            info!("client joined join");
            if lobby.add_player(LobbyPlayer {
                account_id,
                account_name: account.info.name.clone(),
            }) {
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
    mut lobby_manager: ResMut<LobbyManager>,
    mut join_reader: EventReader<MessageEvent<CSLobbyLeave>>,
    mut net_params: NetworkParamsRW,
) {
    for event in join_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to leave a lobby: {client:?}");
            continue;
        }

        let Some(lobby) = lobby_manager.get_lobby_mut(LobbyId(0)) else {
            info!("client attemped to leave a lobby that does not exist: {client:?}");
            continue;
        };

        lobby.remove_player(client.account_id.unwrap());

        info!("client left lobby");

        // TODO Handle rejections for banned accounts etc.
        net_params.server.send_message_to_target::<Channel1, _>(
            SCLobbyLeaveSuccess,
            NetworkTarget::Only(vec![*client_id]),
        );
    }
}

pub(crate) fn receive_lobby_message(
    mut server_metadata: ResMut<ServerMetadataResource>,
    mut lobby_manager: ResMut<LobbyManager>,
    mut message_reader: EventReader<MessageEvent<CSLobbyMessage>>,
    mut net_params: NetworkParamsRW,
    account_q: Query<&AccountInstance>,
) {
    for event in message_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to leave a lobby: {client:?}");
            continue;
        }

        let message_id = server_metadata.0.next_message_id;
        server_metadata.0.next_message_id.0 += 1;

        let lobby_msg = event.message();
        info!("lobby message: {lobby_msg:?}");

        let Some(lobby) = lobby_manager.get_lobby_mut(lobby_msg.id) else {
            info!("client sent message to a lobby that does not exist: {client:?}");
            continue;
        };

        let account = account_q.get(client.entity).unwrap();

        let lobby_message = LobbyMessage {
            id: message_id,
            sender_id: account.0.info.id,
            sender: account.0.info.name.clone(),
            message: lobby_msg.message.clone(),
        };

        let account_ids: Vec<_> = lobby.players.iter().map(|p| p.account_id).collect();

        let client_ids = net_params
            .context
            .get_client_ids_for_account_ids(&account_ids);

        net_params.server.send_message_to_target::<Channel1, _>(
            SCLobbyMessage(LobbyMessage {
                id: MessageId(0),
                sender_id: client.account_id.unwrap(),
                sender: account.0.info.name.clone(),
                message: lobby_msg.message.clone(),
            }),
            NetworkTarget::Only(client_ids.clone()),
        );

        lobby.messages.push(lobby_message);
    }
}
