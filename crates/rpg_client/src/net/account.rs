use bevy::{
    ecs::{component::Component, event::EventReader, system::Query},
    log::info,
};

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
        info!("account creattion success");
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
        info!("account creation error");
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
        info!("login success");
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
        info!("login error");
    }
}

pub(crate) fn receive_character_create_success(
    mut account_events: EventReader<MessageEvent<SCCreateCharacterSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    if PlayerPosition::mode() != ComponentSyncMode::Full {
        return;
    }

    for event in account_events.read() {
        info!("character creation success");
    }
}

pub(crate) fn receive_character_create_error(
    mut account_events: EventReader<MessageEvent<SCCreateCharacterError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    if PlayerPosition::mode() != ComponentSyncMode::Full {
        return;
    }

    for event in account_events.read() {
        info!("character creation error");
    }
}
