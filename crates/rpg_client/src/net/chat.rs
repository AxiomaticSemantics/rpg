use bevy::{
    ecs::{event::EventReader, system::Resource},
    log::info,
};

use rpg_network_protocol::protocol::*;

#[derive(Default, Resource)]
pub(crate) struct Chat;

pub(crate) fn receive_join_success(mut join_events: EventReader<ServerMessage>) {
    for event in join_events.read() {
        let ServerMessage::SCChatJoinSuccess(_) = event else {
            continue;
        };

        info!("chat join success");
    }
}

pub(crate) fn receive_join_error(mut join_events: EventReader<ServerMessage>) {
    for event in join_events.read() {
        let ServerMessage::SCChatJoinError(_) = event else {
            continue;
        };

        info!("chat join error");
    }
}

pub(crate) fn receive_channel_join_success(mut join_events: EventReader<ServerMessage>) {
    for event in join_events.read() {
        let ServerMessage::SCChatChannelJoinSuccess(_) = event else {
            continue;
        };

        info!("chat channel join success");
    }
}

pub(crate) fn receive_channel_join_error(mut join_events: EventReader<ServerMessage>) {
    for event in join_events.read() {
        let ServerMessage::SCChatChannelJoinError(_) = event else {
            continue;
        };

        info!("chat channel join error");
    }
}

pub(crate) fn receive_chat_message(mut message_events: EventReader<ServerMessage>) {
    for event in message_events.read() {
        let ServerMessage::SCChatMessage(msg) = event else {
            continue;
        };

        info!("chat message {msg:?}");
    }
}
