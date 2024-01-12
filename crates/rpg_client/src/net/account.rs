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
    character::{Character, CharacterInfo},
};
use rpg_network_protocol::protocol::*;
use ui_util::style::UiTheme;

use lightyear::client::{
    components::{ComponentSyncMode, SyncComponent},
    events::MessageEvent,
};

#[derive(Component)]
pub(crate) struct RpgAccount(pub(crate) Account);

pub(crate) fn receive_account_create_success(
    mut commands: Commands,
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

        commands.spawn(RpgAccount(account_msg.0.clone()));

        style_set.p0().single_mut().display = Display::None;
        style_set.p1().single_mut().display = Display::Flex;
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
    ui_theme: Res<UiTheme>,
    mut style_set: ParamSet<(
        Query<&mut Style, With<AccountLoginRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
        Query<(&mut Text, &mut Style, &AccountCharacterSlot)>,
    )>,
    mut account_events: EventReader<MessageEvent<SCLoginAccountSuccess>>,
    mut account_q: Query<&mut RpgAccount>,
) {
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

        commands.spawn(RpgAccount(account_msg.0.clone()));

        style_set.p0().single_mut().display = Display::None;
        style_set.p1().single_mut().display = Display::Flex;
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
