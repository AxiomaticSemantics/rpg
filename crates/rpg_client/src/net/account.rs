use bevy::ecs::{component::Component, event::EventReader, system::Query};

use rpg_account::account::Account;
use rpg_network_protocol::protocol::*;

use lightyear::client::{
    components::{ComponentSyncMode, SyncComponent},
    events::MessageEvent,
};

#[derive(Component)]
pub(crate) struct RpgAccount(pub Account);

pub(crate) fn receive_account_create_success(
    mut account_events: EventReader<MessageEvent<SCCreateAccountSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    if PlayerPosition::mode() != ComponentSyncMode::Full {
        return;
    }

    for event in account_events.read() {
        //
    }
}

pub(crate) fn receive_account_create_error(
    mut account_events: EventReader<MessageEvent<SCCreateAccountError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    if PlayerPosition::mode() != ComponentSyncMode::Full {
        return;
    }

    for event in account_events.read() {
        //
    }
}

pub(crate) fn receive_account_login_success(
    mut account_events: EventReader<MessageEvent<SCLoginAccountSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    if PlayerPosition::mode() != ComponentSyncMode::Full {
        return;
    }

    for event in account_events.read() {
        //
    }
}

pub(crate) fn receive_account_login_error(
    mut account_events: EventReader<MessageEvent<SCLoginAccountError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    if PlayerPosition::mode() != ComponentSyncMode::Full {
        return;
    }

    for event in account_events.read() {
        //
    }
}
