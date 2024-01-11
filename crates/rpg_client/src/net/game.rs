use bevy::{
    ecs::{component::Component, event::EventReader, system::Query},
    log::info,
};

use rpg_network_protocol::protocol::*;

use lightyear::client::{
    components::{ComponentSyncMode, SyncComponent},
    events::MessageEvent,
};

pub(crate) fn receive_player_move(mut move_events: EventReader<MessageEvent<SCMovePlayer>>) {
    if PlayerPosition::mode() != ComponentSyncMode::Full {
        return;
    }

    for event in move_events.read() {
        info!("move");
    }
}

pub(crate) fn receive_player_rotation(mut rotation_events: EventReader<MessageEvent<SCRotPlayer>>) {
    if PlayerPosition::mode() != ComponentSyncMode::Full {
        return;
    }

    for event in rotation_events.read() {
        info!("rotation");
    }
}
