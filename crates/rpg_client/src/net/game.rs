use crate::state::AppState;

use bevy::{
    ecs::{
        component::Component,
        event::EventReader,
        schedule::NextState,
        system::{Query, ResMut},
    },
    log::info,
};

use rpg_network_protocol::protocol::*;

use lightyear::client::{
    components::{ComponentSyncMode, SyncComponent},
    events::MessageEvent,
};

pub(crate) fn receive_player_join_success(
    mut state: ResMut<NextState<AppState>>,
    mut join_events: EventReader<MessageEvent<SCPlayerJoinSuccess>>,
) {
    for event in join_events.read() {
        let join_msg = event.message();
        info!("join success {join_msg:?}");

        state.set(AppState::GameSpawn);
        return;
    }
}

pub(crate) fn receive_player_join_error(
    mut state: ResMut<NextState<AppState>>,
    mut join_events: EventReader<MessageEvent<SCPlayerJoinError>>,
) {
    for event in join_events.read() {
        info!("join error");
        // TODO Error screen
        state.set(AppState::Menu);
        return;
    }
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
