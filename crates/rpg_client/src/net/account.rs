use crate::{
    state::AppState,
    ui::menu::{
        account::{self, AccountCharacter, AccountCreateRoot, AccountListRoot, AccountLoginRoot},
        create::CreateRoot,
    },
};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        schedule::NextState,
        system::{Commands, ParamSet, Query, Res, ResMut},
    },
    hierarchy::Children,
    log::info,
    text::Text,
    ui::{Display, Style},
};

use rpg_account::{
    account::{Account, AccountInfo},
    character::{Character, CharacterInfo, CharacterRecord},
};
use rpg_network_protocol::protocol::*;
use ui_util::style::UiTheme;

use lightyear::client::events::MessageEvent;

#[derive(Default, Component)]
pub(crate) struct RpgAccount(pub(crate) Account);

pub(crate) fn receive_account_create_success(
    ui_theme: Res<UiTheme>,
    mut style_set: ParamSet<(
        Query<&mut Style, With<AccountCreateRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
    mut account_events: EventReader<MessageEvent<SCCreateAccountSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    let mut account = account_q.single_mut();

    for event in account_events.read() {
        info!("account creation success");

        let account_msg = event.message();
        account.0 = account_msg.0.clone();

        style_set.p0().single_mut().display = Display::None;
        style_set.p1().single_mut().display = Display::Flex;

        account_events.clear();
        return;
    }
}

pub(crate) fn receive_account_create_error(
    mut account_events: EventReader<MessageEvent<SCCreateAccountError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in account_events.read() {
        info!("account creation error");

        account_events.clear();
        return;
    }
}

pub(crate) fn receive_account_login_success(
    mut commands: Commands,
    ui_theme: Res<UiTheme>,
    mut style_set: ParamSet<(
        Query<&mut Style, With<AccountLoginRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
    mut account_events: EventReader<MessageEvent<SCLoginAccountSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    let mut account = account_q.single_mut();

    for event in account_events.read() {
        info!("login success");

        let account_msg = event.message();
        account.0 = account_msg.0.clone();

        style_set.p0().single_mut().display = Display::None;
        style_set.p1().single_mut().display = Display::Flex;

        account_events.clear();
        return;
    }
}

pub(crate) fn receive_account_login_error(
    mut account_events: EventReader<MessageEvent<SCLoginAccountError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in account_events.read() {
        info!("login error");

        account_events.clear();
        return;
    }
}

pub(crate) fn receive_character_create_success(
    mut account_events: EventReader<MessageEvent<SCCreateCharacterSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
    mut style_set: ParamSet<(
        Query<&mut Style, With<CreateRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
) {
    let mut account = account_q.single_mut();

    for event in account_events.read() {
        let create_msg = event.message();

        info!("character creation success {:?}", create_msg.0.info);

        let mut character_record = account
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

        account_events.clear();
        return;
    }
}

pub(crate) fn receive_character_create_error(
    mut account_events: EventReader<MessageEvent<SCCreateCharacterError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in account_events.read() {
        info!("character creation error");

        account_events.clear();
        return;
    }
}

pub(crate) fn receive_game_join_success(
    mut state: ResMut<NextState<AppState>>,
    mut join_events: EventReader<MessageEvent<SCGameJoinSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in join_events.read() {
        info!("game join success");

        let join_msg = event.message();

        state.set(AppState::GameJoin);

        join_events.clear();
        return;
    }
}

pub(crate) fn receive_game_join_error(
    mut join_events: EventReader<MessageEvent<SCGameJoinError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in join_events.read() {
        info!("game join error");

        join_events.clear();
        return;
    }
}

pub(crate) fn receive_game_create_success(
    mut state: ResMut<NextState<AppState>>,
    mut create_events: EventReader<MessageEvent<SCGameCreateSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in create_events.read() {
        info!("game create success");

        state.set(AppState::GameJoin);

        create_events.clear();
        return;
    }
}

pub(crate) fn receive_game_create_error(
    mut create_events: EventReader<MessageEvent<SCGameCreateError>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    for event in create_events.read() {
        info!("game create error");

        create_events.clear();
        return;
    }
}
