use bevy::{
    ecs::{
        component::Component,
        event::EventReader,
        system::{Query, Resource},
    },
    log::info,
};

use rpg_network_protocol::protocol::*;

use lightyear::client::events::MessageEvent;

#[derive(Default, Resource)]
pub(crate) struct Lobby;

pub(crate) fn receive_join_success(mut join_events: EventReader<MessageEvent<SCLobbyJoinSuccess>>) {
    for event in join_events.read() {
        info!("lobby join success");
    }
}

pub(crate) fn receive_join_error(mut join_events: EventReader<MessageEvent<SCLobbyJoinError>>) {
    for event in join_events.read() {
        info!("lobby join error");
    }
}

pub(crate) fn receive_create_success(
    mut join_events: EventReader<MessageEvent<SCLobbyCreateSuccess>>,
) {
    for event in join_events.read() {
        info!("lobby create success");
    }
}

pub(crate) fn receive_create_error(mut join_events: EventReader<MessageEvent<SCLobbyCreateError>>) {
    for event in join_events.read() {
        info!("lobby create error");
    }
}
