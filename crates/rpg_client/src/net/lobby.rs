use bevy::{
    ecs::{
        component::Component,
        event::EventReader,
        system::{Query, Res, ResMut, Resource},
    },
    log::info,
};

use rpg_account::account::AccountId;
use rpg_lobby::lobby::LobbyId;
use rpg_network_protocol::protocol::*;

use lightyear::client::events::MessageEvent;

pub(crate) struct LobbyInfo {
    pub(crate) id: LobbyId,
    pub(crate) name: String,
    pub(crate) accounts: Vec<AccountId>,
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
            accounts: join_msg.0.accounts.clone(),
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
            accounts: create_msg.0.accounts.clone(),
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
