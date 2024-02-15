use crate::{
    game::plugin::GameState,
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
        system::{Commands, ParamSet, Query, ResMut},
    },
    log::info,
    ui::{Display, Style},
};

use rpg_account::account::Account;
use rpg_network_protocol::protocol::*;

#[derive(Component)]
pub(crate) struct RpgAccount(pub(crate) Account);

pub(crate) fn receive_account_create_success(
    mut commands: Commands,
    mut style_set: ParamSet<(
        Query<&mut Style, With<AccountCreateRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
    mut account_events: EventReader<ServerMessage>,
) {
    for event in account_events.read() {
        let ServerMessage::SCCreateAccountSuccess(msg) = event else {
            continue;
        };

        info!("account creation success");

        commands.spawn(RpgAccount(msg.0.clone()));

        style_set.p0().single_mut().display = Display::None;
        style_set.p1().single_mut().display = Display::Flex;
    }
}

pub(crate) fn receive_account_create_error(mut account_events: EventReader<ServerMessage>) {
    for event in account_events.read() {
        let ServerMessage::SCCreateAccountError(_) = event else {
            continue;
        };

        info!("account creation error");
    }
}

pub(crate) fn receive_account_login_success(
    mut commands: Commands,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<AccountLoginRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
    mut login_reader: EventReader<ServerMessage>,
) {
    for event in login_reader.read() {
        let ServerMessage::SCLoginAccountSuccess(msg) = event else {
            continue;
        };

        info!("login success");

        commands.spawn(RpgAccount(msg.0.clone()));

        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::Flex;
    }
}

pub(crate) fn receive_account_login_error(mut login_reader: EventReader<ServerMessage>) {
    for event in login_reader.read() {
        let ServerMessage::SCLoginAccountError(_) = event else {
            continue;
        };

        info!("login error");
    }
}

pub(crate) fn receive_character_create_success(
    mut create_reader: EventReader<ServerMessage>,
    mut account_q: Query<&mut RpgAccount>,
    mut style_set: ParamSet<(
        Query<&mut Style, With<CreateRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
) {
    for event in create_reader.read() {
        let ServerMessage::SCCreateCharacterSuccess(msg) = event else {
            continue;
        };

        info!("character creation success {:?}", msg.0.info);

        let mut account = account_q.single_mut();
        let character_record = account
            .0
            .characters
            .iter_mut()
            .find(|c| c.info.slot == msg.0.info.slot);

        if let Some(character_record) = character_record {
            *character_record = msg.0.clone();
        } else {
            account.0.characters.push(msg.0.clone());
        }

        style_set.p0().single_mut().display = Display::None;
        style_set.p1().single_mut().display = Display::Flex;
    }
}

pub(crate) fn receive_character_create_error(mut create_reader: EventReader<ServerMessage>) {
    for event in create_reader.read() {
        let ServerMessage::SCCreateCharacterError(_) = event else {
            continue;
        };

        info!("character creation error");
    }
}

pub(crate) fn receive_game_join_success(
    mut state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    mut join_events: EventReader<ServerMessage>,
) {
    for event in join_events.read() {
        let ServerMessage::SCGameJoinSuccess(msg) = event else {
            continue;
        };

        info!("game join success {msg:?}");

        game_state.mode = msg.0;
        state.set(AppState::GameSpawn);

        return;
    }
}

pub(crate) fn receive_game_join_error(mut join_events: EventReader<ServerMessage>) {
    for event in join_events.read() {
        let ServerMessage::SCGameJoinError(_) = event else {
            continue;
        };

        info!("game join error");
    }
}

pub(crate) fn receive_game_create_success(
    mut state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    mut create_events: EventReader<ServerMessage>,
) {
    for event in create_events.read() {
        let ServerMessage::SCGameCreateSuccess(msg) = event else {
            continue;
        };

        info!("game create success {msg:?}");

        game_state.mode = msg.0;
        state.set(AppState::GameSpawn);

        create_events.clear();
        return;
    }
}

pub(crate) fn receive_game_create_error(mut create_events: EventReader<ServerMessage>) {
    for event in create_events.read() {
        let ServerMessage::SCGameCreateError(_) = event else {
            continue;
        };

        info!("game create error");
    }
}
