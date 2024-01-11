use bevy::{
    ecs::{component::Component, event::EventReader, system::Query},
    log::info,
};

use rpg_network_protocol::protocol::*;

use lightyear::client::{
    components::{ComponentSyncMode, SyncComponent},
    events::MessageEvent,
};

pub(crate) fn receive_join_success(mut join_events: EventReader<MessageEvent<SCChatJoinSuccess>>) {
    for event in join_events.read() {
        info!("chat join success");
    }
}

pub(crate) fn receive_join_error(mut join_events: EventReader<MessageEvent<SCChatJoinError>>) {
    for event in join_events.read() {
        info!("chat join error");
    }
}

pub(crate) fn receive_channel_join_success(
    mut join_events: EventReader<MessageEvent<SCChatChannelJoinSuccess>>,
) {
    for event in join_events.read() {
        info!("chat channel join success");
    }
}

pub(crate) fn receive_channel_join_error(
    mut join_events: EventReader<MessageEvent<SCChatChannelJoinError>>,
) {
    for event in join_events.read() {
        info!("chat channel join error");
    }
}

pub(crate) fn receive_chat_message(mut message_events: EventReader<MessageEvent<SCChatMessage>>) {
    for event in message_events.read() {
        info!("chat message {:?}", event.message());
    }
}
