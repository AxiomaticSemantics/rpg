use crate::menu::account::{
    self, AccountCharacterSlot, AccountCreateRoot, AccountListRoot, AccountLoginRoot,
};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        system::{Commands, ParamSet, Query, Res},
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

use lightyear::client::{
    components::{ComponentSyncMode, SyncComponent},
    events::MessageEvent,
};

#[derive(Default, Component)]
pub(crate) struct RpgAccount(pub(crate) Account);

pub(crate) fn receive_account_create_success(
    ui_theme: Res<UiTheme>,
    mut style_set: ParamSet<(
        Query<&mut Style, With<AccountCreateRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
        Query<(&mut Text, &mut Style, &AccountCharacterSlot)>,
    )>,
    mut account_events: EventReader<MessageEvent<SCCreateAccountSuccess>>,
) {
    for event in account_events.read() {
        info!("account creation success");

        let account_msg = event.message();

        for character_record in account_msg.0.characters.iter() {
            for (mut slot_text, mut slot_style, slot) in &mut style_set.p2() {
                if slot.0 != character_record.info.slot {
                    continue;
                }

                let slot_string = format!(
                    "{} level {} {}",
                    character_record.character.unit.name,
                    character_record.character.unit.level,
                    character_record.character.unit.class
                );
                slot_text.sections[0].value = slot_string;
            }
        }

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
        Query<(&mut Text, &mut Style, &AccountCharacterSlot)>,
    )>,
    mut account_events: EventReader<MessageEvent<SCLoginAccountSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
    let mut account = account_q.single_mut();

    for event in account_events.read() {
        info!("login success");

        let account_msg = event.message();

        for character_record in account_msg.0.characters.iter() {
            for (mut slot_text, mut slot_style, slot) in &mut style_set.p2() {
                if slot.0 != character_record.info.slot {
                    info!("{slot:?} {:?}", character_record.info.slot);
                    continue;
                }

                let slot_string = format!(
                    "{} level {} {}",
                    character_record.character.unit.name,
                    character_record.character.unit.level,
                    character_record.character.unit.class
                );
                slot_text.sections[0].value = slot_string;
            }
        }

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
