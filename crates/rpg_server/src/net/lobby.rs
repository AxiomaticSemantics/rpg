use super::server::{ClientMessageEvent, NetworkParamsRW};
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

pub(crate) fn receive_lobby_create(
    mut lobby_manager: ResMut<LobbyManager>,
    mut create_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
    account_q: Query<&AccountInstance>,
) {
    for event in create_reader.read() {
        let ClientMessage::CSLobbyCreate(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to create a lobby: {client:?}");
            continue;
        }
        let account_id = client.account_id.unwrap();
        let account = account_q.get(client.entity).unwrap();

        if let Some(lobby_id) = lobby_manager.add_lobby(msg.name.clone(), msg.game_mode) {
            info!("lobby created");

            lobby_manager.add_player(lobby_id, account_id, account.info.name.clone());

            let message = bincode::serialize(&ServerMessage::SCLobbyCreateSuccess(
                SCLobbyCreateSuccess(Lobby {
                    id: lobby_id,
                    name: msg.name.clone(),
                    game_mode: msg.game_mode,
                    messages: vec![],
                    players: vec![LobbyPlayer {
                        account_id,
                        account_name: account.info.name.clone(),
                    }],
                }),
            ))
            .unwrap();

            net_params
                .server
                .send_message(client_id, ServerChannel::Message, message);
        } else {
            let message =
                bincode::serialize(&ServerMessage::SCLobbyCreateError(SCLobbyCreateError)).unwrap();
            net_params
                .server
                .send_message(client_id, ServerChannel::Message, message);
        }
    }
}

pub(crate) fn receive_lobby_join(
    mut lobby_manager: ResMut<LobbyManager>,
    mut join_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
    account_q: Query<&AccountInstance>,
) {
    for event in join_reader.read() {
        let ClientMessage::CSLobbyJoin(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to join a lobby: {client:?}");
            continue;
        }
        let account_id = client.account_id.unwrap();
        let account = account_q.get(client.entity).unwrap();

        if let Some(lobby) = lobby_manager.get_lobby_mut(msg.0) {
            info!("client joined join");
            if lobby.add_player(LobbyPlayer {
                account_id,
                account_name: account.info.name.clone(),
            }) {
                // TODO Handle rejections for banned accounts etc.
                let message = bincode::serialize(&ServerMessage::SCLobbyJoinSuccess(
                    SCLobbyJoinSuccess(lobby.clone()),
                ))
                .unwrap();
                net_params
                    .server
                    .send_message(client_id, ServerChannel::Message, message);
            }
        }
    }
}

pub(crate) fn receive_lobby_leave(
    mut lobby_manager: ResMut<LobbyManager>,
    mut join_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
) {
    for event in join_reader.read() {
        let ClientMessage::CSLobbyLeave(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get(&client_id).unwrap();
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
        let message =
            bincode::serialize(&ServerMessage::SCLobbyLeaveSuccess(SCLobbyLeaveSuccess)).unwrap();
        net_params
            .server
            .send_message(client_id, ServerChannel::Message, message);
    }
}

pub(crate) fn receive_lobby_message(
    mut server_metadata: ResMut<ServerMetadataResource>,
    mut lobby_manager: ResMut<LobbyManager>,
    mut message_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
    account_q: Query<&AccountInstance>,
) {
    for event in message_reader.read() {
        let ClientMessage::CSLobbyMessage(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated() {
            info!("unauthenticated client attempted to leave a lobby: {client:?}");
            continue;
        }

        let message_id = server_metadata.0.next_message_id;
        server_metadata.0.next_message_id.0 += 1;

        info!("lobby message: {msg:?}");

        let Some(lobby) = lobby_manager.get_lobby_mut(msg.id) else {
            info!("client sent message to a lobby that does not exist: {client:?}");
            continue;
        };

        let account = account_q.get(client.entity).unwrap();

        let lobby_message = LobbyMessage {
            id: message_id,
            sender_id: account.0.info.id,
            sender: account.0.info.name.clone(),
            message: msg.message.clone(),
        };

        let account_ids: Vec<_> = lobby.players.iter().map(|p| p.account_id).collect();

        let client_ids = net_params
            .context
            .get_client_ids_for_account_ids(&account_ids);

        let message = bincode::serialize(&ServerMessage::SCLobbyMessage(SCLobbyMessage(
            LobbyMessage {
                id: MessageId(0),
                sender_id: client.account_id.unwrap(),
                sender: account.0.info.name.clone(),
                message: msg.message.clone(),
            },
        )))
        .unwrap();

        for client_id in client_ids {
            net_params
                .server
                .send_message(client_id, ServerChannel::Message, message.clone());
        }

        lobby.messages.push(lobby_message);
    }
}
