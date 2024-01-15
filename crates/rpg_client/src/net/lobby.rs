use bevy::{
    ecs::{
        component::Component,
        event::EventReader,
        system::{Query, Res, ResMut, Resource},
    },
    log::info,
};

use rpg_network_protocol::protocol::*;

use lightyear::client::events::MessageEvent;

#[derive(Default, Resource)]
pub(crate) struct Lobby;

pub(crate) fn receive_join_success(
    mut lobby: ResMut<Lobby>,
    mut join_events: EventReader<MessageEvent<SCLobbyJoinSuccess>>,
) {
    for event in join_events.read() {
        info!("lobby join success");

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
        info!("lobby create success");

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

pub(crate) fn receive_leave(
    mut lobby: ResMut<Lobby>,
    mut join_events: EventReader<MessageEvent<SCLobbyCreateError>>,
) {
    for event in join_events.read() {
        info!("lobby create error");
    }
}
