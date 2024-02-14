use bevy::{
    ecs::{
        event::EventReader,
        system::{ResMut, Resource},
    },
    log::info,
};

use rpg_core::game_mode::GameMode;
use rpg_lobby::lobby::{LobbyId, LobbyMessage, LobbyPlayer};
use rpg_network_protocol::protocol::*;

pub(crate) struct LobbyInfo {
    pub(crate) id: LobbyId,
    pub(crate) name: String,
    pub(crate) game_mode: GameMode,
    pub(crate) players: Vec<LobbyPlayer>,
    pub(crate) messages: Vec<LobbyMessage>,
}

#[derive(Default, Resource)]
pub(crate) struct Lobby(pub(crate) Option<LobbyInfo>);

pub(crate) fn receive_join_success(
    mut lobby: ResMut<Lobby>,
    mut join_events: EventReader<ServerMessage>,
) {
    for event in join_events.read() {
        let ServerMessage::SCLobbyJoinSuccess(msg) = event else {
            continue;
        };

        if let Some(_) = &mut lobby.0 {
            info!("received join lobby while already in lobby?");
            join_events.clear();
            return;
        }

        info!("lobby join success");

        lobby.0 = Some(LobbyInfo {
            id: msg.0.id,
            name: msg.0.name.clone(),
            game_mode: msg.0.game_mode,
            players: msg.0.players.clone(),
            messages: msg.0.messages.clone(),
        });
    }
}

pub(crate) fn receive_join_error(mut join_events: EventReader<ServerMessage>) {
    for event in join_events.read() {
        let ServerMessage::SCLobbyJoinError(msg) = event else {
            continue;
        };

        info!("lobby join error");
    }
}

pub(crate) fn receive_create_success(
    mut lobby: ResMut<Lobby>,
    mut create_events: EventReader<ServerMessage>,
) {
    for event in create_events.read() {
        let ServerMessage::SCLobbyCreateSuccess(msg) = event else {
            continue;
        };

        if let Some(_) = &mut lobby.0 {
            info!("received create lobby while already in lobby?");

            create_events.clear();
            return;
        }

        info!("lobby create success");

        lobby.0 = Some(LobbyInfo {
            id: msg.0.id,
            name: msg.0.name.clone(),
            game_mode: msg.0.game_mode,
            players: msg.0.players.clone(),
            messages: vec![],
        });
    }
}

pub(crate) fn receive_create_error(mut create_events: EventReader<ServerMessage>) {
    for event in create_events.read() {
        let ServerMessage::SCLobbyCreateError(msg) = event else {
            continue;
        };

        info!("lobby create error");
    }
}

pub(crate) fn receive_leave_success(
    mut lobby: ResMut<Lobby>,
    mut leave_events: EventReader<ServerMessage>,
) {
    for event in leave_events.read() {
        let ServerMessage::SCLobbyLeaveSuccess(msg) = event else {
            continue;
        };

        info!("lobby leave");

        lobby.0 = None;
    }
}

pub(crate) fn receive_leave_error(
    mut lobby: ResMut<Lobby>,
    mut leave_events: EventReader<ServerMessage>,
) {
    for event in leave_events.read() {
        let ServerMessage::SCLobbyLeaveError(msg) = event else {
            continue;
        };

        lobby.0 = None;
        info!("lobby leave error");
    }
}

pub(crate) fn receive_lobby_message(
    mut lobby: ResMut<Lobby>,
    mut message_events: EventReader<ServerMessage>,
) {
    for event in message_events.read() {
        let ServerMessage::SCLobbyMessage(msg) = event else {
            continue;
        };

        let Some(lobby) = &mut lobby.0 else {
            info!("received lobby message while not in a lobby");
            return;
        };

        info!("lobby message: {msg:?}");

        lobby.messages.push(LobbyMessage {
            id: msg.0.id,
            message: msg.0.message.clone(),
            sender_id: msg.0.sender_id,
            sender: msg.0.sender.clone(),
        });
    }
}
