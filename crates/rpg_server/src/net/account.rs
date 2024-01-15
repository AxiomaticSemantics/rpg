use crate::assets::MetadataResources;

use super::server::{ClientType, NetworkParamsRO, NetworkParamsRW};

use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        event::EventReader,
        system::{Commands, Query, Res},
    },
    log::info,
    prelude::{Deref, DerefMut},
};

use lightyear::prelude::server::*;
use lightyear::prelude::NetworkTarget;

use rpg_account::{
    account::{Account, AccountInfo, AdminAccount, AdminAccountInfo},
    character::{Character, CharacterInfo, CharacterRecord},
};
use rpg_core::{
    passive_tree::PassiveSkillGraph,
    storage::UnitStorage,
    unit::{HeroInfo, Unit, UnitInfo, UnitKind},
};
use rpg_network_protocol::protocol::*;

use util::fs::{open_read, open_write};

use serde_json;

use std::{env, path::Path};

#[derive(Debug, Deref, DerefMut, Component)]
pub(crate) struct AccountInstance(pub(crate) Account);

#[derive(Debug, Deref, DerefMut, Component)]
pub(crate) struct AdminAccountInstance(pub(crate) AdminAccount);

#[derive(Bundle)]
pub(crate) struct AccountInstanceBundle {
    pub account: AccountInstance,
}

#[derive(Bundle)]
pub(crate) struct AdminAccountInstanceBundle {
    pub account: AdminAccountInstance,
}

pub(crate) fn receive_account_create(
    mut commands: Commands,
    mut account_create_reader: EventReader<MessageEvent<CSCreateAccount>>,
    mut net_params: NetworkParamsRW,
) {
    for event in account_create_reader.read() {
        let client = net_params.context.clients.get(event.context()).unwrap();
        if client.is_authenticated_player() {
            info!("already authenticated client attempted to create account {client:?}");
            continue;
        }

        // Allow authenticated admins to create accounts
        if client.is_admin() && !client.is_authenticated() {
            panic!("unauthenticated admin: {client:?}");
        }

        let file_path = format!(
            "{}/server/accounts/{}.json",
            std::env::var("RPG_SAVE_ROOT").unwrap(),
            event.message().name
        );
        let path = Path::new(file_path.as_str());
        let file = open_read(path);

        if let Ok(_) = file {
            info!("account already exists");
        } else {
            let Ok(file) = open_write(path) else {
                info!("unable to open account file for writing");
                continue;
            };

            let account = Account {
                info: AccountInfo {
                    character_slots: 12,
                    name: event.message().name.clone(),
                    id: net_params.state.next_account_id,
                },
                characters: vec![],
            };

            serde_json::to_writer(file, &account).unwrap();

            commands.spawn(AccountInstance(account));

            net_params.state.next_uid.next();

            info!("writing account file to {file_path}");
        }
    }
}

pub(crate) fn receive_admin_login(
    mut commands: Commands,
    mut login_reader: EventReader<MessageEvent<CSLoadAdminAccount>>,
    mut net_params: NetworkParamsRW,
) {
    for event in login_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get_mut(client_id).unwrap();
        if client.is_authenticated_player() {
            info!("authenticated player attempted to login to account {client:?}");
            continue;
        } else if client.is_authenticated_admin() {
            info!("authenticated admin attempted to login to account {client:?}");
            continue;
        }

        let file_path = format!(
            "{}/server/admin_accounts/{}.json",
            std::env::var("RPG_SAVE_ROOT").unwrap(),
            event.message().name
        );
        let path = Path::new(file_path.as_str());
        let file = open_read(path);

        if let Ok(_) = file {
            info!("admin account already exists");
        } else {
            let Ok(file) = open_write(path) else {
                info!("unable to open admin account file for writing");
                continue;
            };

            let account = AdminAccount {
                info: AdminAccountInfo {
                    name: event.message().name.clone(),
                    id: net_params.state.next_account_id,
                },
            };

            serde_json::to_writer(file, &account).unwrap();

            commands.spawn(AdminAccountInstance(account));

            net_params.state.next_uid.next();

            info!("writing account file to {file_path}");
        }
    }
}

pub(crate) fn receive_account_login(
    mut commands: Commands,
    mut login_reader: EventReader<MessageEvent<CSLoadAccount>>,
    mut net_params: NetworkParamsRW,
) {
    for event in login_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get_mut(client_id).unwrap();
        if client.is_authenticated_player() {
            info!("authenticated player attempted to load account {client:?}");
            continue;
        }

        let file_path = format!(
            "{}/server/accounts/{}.json",
            env::var("RPG_SAVE_ROOT").unwrap(),
            event.message().name
        );
        let path = Path::new(file_path.as_str());
        let file = open_read(path);

        if let Ok(file) = file {
            let account: Result<Account, _> = serde_json::from_reader(file);
            if let Ok(account) = account {
                // FIXME assign a client id and send it to the client
                client.client_type = ClientType::Player;
                client.account_id = Some(account.info.id);
                info!("spawning RpgAccount for {client:?}");

                commands.spawn(AccountInstanceBundle {
                    account: AccountInstance(account.clone()),
                });

                /*
                client.entity = commands
                    .spawn((
                        protocol::NetworkPlayerBundle::new(*client_id, Vec3::ZERO, Vec3::ZERO),
                        TransformBundle::from_transform(
                            Transform::from_translation(Vec3::ZERO).looking_to(Vec3::NEG_Z, Vec3::Y),
                        ),
                    ))
                    .id();
                        */
                net_params.server.send_message_to_target::<Channel1, _>(
                    SCLoginAccountSuccess(account.clone()),
                    NetworkTarget::Only(vec![*client_id]),
                );
            } else {
                info!("unable to deserialize account: {file_path}");
            }
        } else {
            info!("account does not exist {client:?}");
        }
    }
}

pub(crate) fn receive_character_create(
    metadata: Res<MetadataResources>,
    mut character_create_reader: EventReader<MessageEvent<CSCreateCharacter>>,
    mut net_params: NetworkParamsRW,
    mut account_q: Query<&mut AccountInstance>,
) {
    for event in character_create_reader.read() {
        let client_id = event.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated_player() {
            info!("unauthenticated client attempted to create character {client:?}");
            continue;
        }

        let create_msg = event.message();

        for mut account in &mut account_q {
            if account
                .0
                .characters
                .iter()
                .any(|c| c.info.slot == event.message().slot)
            {
                info!("character already exists");

                net_params
                    .server
                    .send_message_to_target::<Channel1, SCCreateAccountError>(
                        SCCreateAccountError,
                        NetworkTarget::Only(vec![*client_id]),
                    )
                    .unwrap();
            } else {
                let unit_info = UnitInfo::Hero(HeroInfo::new(&metadata.0, create_msg.game_mode));
                let mut unit = Unit::new(
                    net_params.state.next_uid.get(),
                    create_msg.class,
                    UnitKind::Hero,
                    unit_info,
                    1,
                    create_msg.name.clone(),
                    None,
                    &metadata.0,
                );
                unit.add_default_skills(&metadata.0);

                net_params.state.next_uid.next();

                let character_info = CharacterInfo {
                    name: create_msg.name.clone(),
                    slot: create_msg.slot,
                    uid: unit.uid,
                    game_mode: create_msg.game_mode,
                };

                let character = CharacterRecord {
                    info: character_info,
                    character: Character {
                        unit,
                        passive_tree: PassiveSkillGraph::new(create_msg.class),
                        storage: UnitStorage::default(),
                    },
                };

                net_params.state.next_uid.next();

                net_params
                    .server
                    .send_message_to_target::<Channel1, SCCreateCharacterSuccess>(
                        SCCreateCharacterSuccess(character.clone()),
                        NetworkTarget::Only(vec![*client_id]),
                    )
                    .unwrap();

                account.0.characters.push(character);

                let file_path = format!(
                    "{}/server/accounts/{}.json",
                    env::var("RPG_SAVE_ROOT").unwrap(),
                    account.0.info.name
                );
                let path = Path::new(file_path.as_str());
                let Ok(file) = open_write(path) else {
                    info!("unable to open account file for writing");
                    continue;
                };
                serde_json::to_writer(file, &account.0).unwrap();
            }
        }
    }
}

pub(crate) fn receive_game_create(
    net_params: NetworkParamsRO,
    mut create_events: EventReader<MessageEvent<CSCreateGame>>,
) {
    for create in create_events.read() {
        let client_id = create.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let create_msg = create.message();
        info!("create game {create_msg:?}");
    }
}

pub(crate) fn receive_game_join(
    net_params: NetworkParamsRO,
    mut join_events: EventReader<MessageEvent<CSJoinGame>>,
) {
    for join in join_events.read() {
        let client_id = join.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let join_msg = join.message();
        info!("join game {join_msg:?}");
    }
}
