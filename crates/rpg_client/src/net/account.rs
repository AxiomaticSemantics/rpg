use crate::menu::account::{AccountCreateRoot, AccountListContainer, AccountListRoot};

use bevy::{
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        system::{Commands, ParamSet, Query},
    },
    log::info,
    ui::{Display, Style},
};

use rpg_account::{
    account::{Account, AccountInfo},
    character::{Character, CharacterInfo},
};
use rpg_network_protocol::protocol::*;

use lightyear::client::{
    components::{ComponentSyncMode, SyncComponent},
    events::MessageEvent,
};

#[derive(Component)]
pub(crate) struct RpgAccount(pub(crate) Account);

pub(crate) fn receive_account_create_success(
    mut commands: Commands,
    mut account_menu_set: ParamSet<(
        Query<&mut Style, With<AccountCreateRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
    mut account_events: EventReader<MessageEvent<SCCreateAccountSuccess>>,
    mut account_container_q: Query<&mut AccountListContainer>,
) {
    for event in account_events.read() {
        info!("account creation success");

        let account_msg = event.message();

        commands.spawn(RpgAccount(account_msg.0.clone()));

        account_menu_set.p0().single_mut().display = Display::None;
        account_menu_set.p1().single_mut().display = Display::Flex;
        return;
    }
}

pub(crate) fn receive_account_create_error(
    mut account_events: EventReader<MessageEvent<SCCreateAccountError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in account_events.read() {
        info!("account creation error");
    }
}

pub(crate) fn receive_account_login_success(
    mut commands: Commands,
    mut account_menu_set: ParamSet<(
        Query<&mut Style, With<AccountCreateRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
    mut account_events: EventReader<MessageEvent<SCLoginAccountSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
    mut account_container_q: Query<&mut AccountListContainer>,
) {
    for event in account_events.read() {
        info!("login success");

        let account_msg = event.message();

        commands.spawn(RpgAccount(account_msg.0.clone()));

        account_menu_set.p0().single_mut().display = Display::None;
        account_menu_set.p1().single_mut().display = Display::Flex;
        return;
    }
}

pub(crate) fn receive_account_login_error(
    mut account_events: EventReader<MessageEvent<SCLoginAccountError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in account_events.read() {
        info!("login error");
    }
}

pub(crate) fn receive_character_create_success(
    mut account_events: EventReader<MessageEvent<SCCreateCharacterSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in account_events.read() {
        info!("character creation success");
    }
}

pub(crate) fn receive_character_create_error(
    mut account_events: EventReader<MessageEvent<SCCreateCharacterError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in account_events.read() {
        info!("character creation error");
    }
}
