use crate::{
    state::AppState,
    ui::menu::{
        account::{AccountCreateRoot, AccountListRoot, AccountLoginRoot},
        create::CreateRoot,
    },
};

use bevy::{
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        schedule::NextState,
        system::{ParamSet, Query, Res, ResMut},
    },
    log::info,
    ui::{Display, Style},
};

use rpg_account::{
    account::{Account, AccountInfo},
    character::{Character, CharacterInfo, CharacterRecord},
};
use rpg_network_protocol::protocol::*;

use lightyear::client::events::MessageEvent;

#[derive(Default, Component)]
pub(crate) struct RpgAccount(pub(crate) Account);

pub(crate) fn receive_account_create_success(
    mut style_set: ParamSet<(
        Query<&mut Style, With<AccountCreateRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
    mut account_events: EventReader<MessageEvent<SCCreateAccountSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in account_events.read() {
        info!("account creation success");

        let account_msg = event.message();

        let mut account = account_q.single_mut();
        account.0 = account_msg.0.clone();

        style_set.p0().single_mut().display = Display::None;
        style_set.p1().single_mut().display = Display::Flex;

        account_events.clear();
        return;
    }
}

pub(crate) fn receive_account_create_error(
    mut account_events: EventReader<MessageEvent<SCCreateAccountError>>,
) {
    for _ in account_events.read() {
        info!("account creation error");

        account_events.clear();
        return;
    }
}

pub(crate) fn receive_account_login_success(
    mut style_set: ParamSet<(
        Query<&mut Style, With<AccountLoginRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
    mut login_reader: EventReader<MessageEvent<SCLoginAccountSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in login_reader.read() {
        info!("login success");

        let account_msg = event.message();
        let mut account = account_q.single_mut();
        account.0 = account_msg.0.clone();

        style_set.p0().single_mut().display = Display::None;
        style_set.p1().single_mut().display = Display::Flex;

        login_reader.clear();
        return;
    }
}

pub(crate) fn receive_account_login_error(
    mut login_reader: EventReader<MessageEvent<SCLoginAccountError>>,
) {
    for _ in login_reader.read() {
        info!("login error");

        login_reader.clear();
        return;
    }
}

pub(crate) fn receive_character_create_success(
    mut create_reader: EventReader<MessageEvent<SCCreateCharacterSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
    mut style_set: ParamSet<(
        Query<&mut Style, With<CreateRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
) {
    for event in create_reader.read() {
        let create_msg = event.message();

        info!("character creation success {:?}", create_msg.0.info);

        let mut account = account_q.single_mut();
        let character_record = account
            .0
            .characters
            .iter_mut()
            .find(|c| c.info.slot == create_msg.0.info.slot);

        if let Some(character_record) = character_record {
            character_record.character = create_msg.0.character.clone();
            character_record.info = create_msg.0.info.clone();
        } else {
            let character_record = CharacterRecord {
                info: create_msg.0.info.clone(),
                character: create_msg.0.character.clone(),
            };

            account.0.characters.push(character_record);
        }

        style_set.p0().single_mut().display = Display::None;
        style_set.p1().single_mut().display = Display::Flex;

        create_reader.clear();
        return;
    }
}

pub(crate) fn receive_character_create_error(
    mut create_reader: EventReader<MessageEvent<SCCreateCharacterError>>,
) {
    for _ in create_reader.read() {
        info!("character creation error");

        create_reader.clear();
        return;
    }
}

pub(crate) fn receive_game_join_success(
    mut state: ResMut<NextState<AppState>>,
    mut join_events: EventReader<MessageEvent<SCGameJoinSuccess>>,
) {
    for event in join_events.read() {
        let join_msg = event.message();
        info!("game join success {join_msg:?}");

        state.set(AppState::GameJoin);

        join_events.clear();
        return;
    }
}

pub(crate) fn receive_game_join_error(mut join_events: EventReader<MessageEvent<SCGameJoinError>>) {
    for _ in join_events.read() {
        info!("game join error");

        join_events.clear();
        return;
    }
}

pub(crate) fn receive_game_create_success(
    mut state: ResMut<NextState<AppState>>,
    mut create_events: EventReader<MessageEvent<SCGameCreateSuccess>>,
) {
    for event in create_events.read() {
        let create_msg = event.message();
        info!("game create success {create_msg:?}");

        state.set(AppState::GameSpawn);

        create_events.clear();
        return;
    }
}

pub(crate) fn receive_game_create_error(
    mut create_events: EventReader<MessageEvent<SCGameCreateError>>,
) {
    for _ in create_events.read() {
        info!("game create error");

        create_events.clear();
        return;
    }
}
