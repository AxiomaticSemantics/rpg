use bevy::{
    ecs::{component::Component, event::EventReader, system::Query},
    log::info,
};

use rpg_network_protocol::protocol::*;

use lightyear::client::{
    components::{ComponentSyncMode, SyncComponent},
    events::MessageEvent,
};

pub(crate) fn receive_player_join_success(
    mut join_events: EventReader<MessageEvent<SCPlayerJoinSuccess>>,
) {
}

pub(crate) fn receive_player_join_error(
    mut join_events: EventReader<MessageEvent<SCPlayerJoinError>>,
) {
}

pub(crate) fn receive_player_move(mut move_events: EventReader<MessageEvent<SCMovePlayer>>) {
    for event in move_events.read() {
        info!("move");
    }
}

pub(crate) fn receive_player_rotation(mut rotation_events: EventReader<MessageEvent<SCRotPlayer>>) {
    for event in rotation_events.read() {
        info!("rotation");
    }
}
