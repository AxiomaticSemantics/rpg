use bevy::{
    ecs::{
        component::Component,
        event::EventReader,
        system::{Query, Res, ResMut, Resource},
    },
    log::info,
};

use rpg_account::account::AccountId;
use rpg_chat::chat::MessageId;
use rpg_core::unit::HeroGameMode;
use rpg_lobby::lobby::{LobbyId, LobbyMessage};
use rpg_network_protocol::protocol::*;

use lightyear::client::events::MessageEvent;

pub(crate) struct LobbyInfo {
    pub(crate) id: LobbyId,
    pub(crate) name: String,
    pub(crate) game_mode: HeroGameMode,
    pub(crate) accounts: Vec<AccountId>,
    pub(crate) messages: Vec<LobbyMessage>,
}

#[derive(Default, Resource)]
pub(crate) struct Lobby(pub(crate) Option<LobbyInfo>);

pub(crate) fn receive_join_success(
    mut lobby: ResMut<Lobby>,
    mut join_events: EventReader<MessageEvent<SCLobbyJoinSuccess>>,
) {
    for event in join_events.read() {
        if let Some(lobby) = &mut lobby.0 {
            info!("received join lobby while already in lobby?");
            join_events.clear();
            return;
        }

        info!("lobby join success");

        let join_msg = event.message();
        lobby.0 = Some(LobbyInfo {
            id: join_msg.0.id,
            name: join_msg.0.name.clone(),
            game_mode: join_msg.0.game_mode,
            accounts: join_msg.0.accounts.clone(),
            messages: join_msg.0.messages.clone(),
        });

        join_events.clear();
        return;
    }
}

pub(crate) fn receive_join_error(mut join_events: EventReader<MessageEvent<SCLobbyJoinError>>) {
    for event in join_events.read() {
        info!("lobby join error");
    }
}

pub(crate) fn receive_create_success(
    mut lobby: ResMut<Lobby>,
    mut create_events: EventReader<MessageEvent<SCLobbyCreateSuccess>>,
) {
    for event in create_events.read() {
        if let Some(lobby) = &mut lobby.0 {
            info!("received create lobby while already in lobby?");

            create_events.clear();
            return;
        }

        let create_msg = event.message();

        info!("lobby create success");

        lobby.0 = Some(LobbyInfo {
            id: create_msg.0.id,
            name: create_msg.0.name.clone(),
            game_mode: create_msg.0.game_mode,
            accounts: create_msg.0.accounts.clone(),
            messages: vec![],
        });

        create_events.clear();
        return;
    }
}

pub(crate) fn receive_create_error(
    mut create_events: EventReader<MessageEvent<SCLobbyCreateError>>,
) {
    for event in create_events.read() {
        info!("lobby create error");
    }
}

pub(crate) fn receive_leave_success(
    mut lobby: ResMut<Lobby>,
    mut leave_events: EventReader<MessageEvent<SCLobbyLeaveSuccess>>,
) {
    for event in leave_events.read() {
        info!("lobby leave");

        lobby.0 = None;
        leave_events.clear();
        return;
    }
}

pub(crate) fn receive_leave_error(
    mut lobby: ResMut<Lobby>,
    mut leave_events: EventReader<MessageEvent<SCLobbyLeaveError>>,
) {
    for event in leave_events.read() {
        info!("lobby leave");
    }
}

pub(crate) fn receive_lobby_message(
    mut lobby: ResMut<Lobby>,
    mut message_events: EventReader<MessageEvent<SCLobbyMessage>>,
) {
    for event in message_events.read() {
        let lobby_message = event.message();

        let Some(lobby) = &mut lobby.0 else {
            info!("received lobby message while not in a lobby");
            message_events.clear();
            return;
        };

        info!("lobby message: {lobby_message:?}");

        lobby.messages.push(LobbyMessage {
            id: lobby_message.0.id,
            message: lobby_message.0.message.clone(),
            sender_id: lobby_message.0.sender_id,
            sender: lobby_message.0.sender.clone(),
        });
    }
}
