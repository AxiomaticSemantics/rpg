use crate::{
    assets::MetadataResources,
    server::{AuthorizationStatus, ClientType, NetworkContext, NetworkParamsRO, NetworkParamsRW},
};

use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        event::EventReader,
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
    math::Vec3,
    prelude::{Deref, DerefMut},
    transform::{components::Transform, TransformBundle},
};

use lightyear::prelude::server::*;
use lightyear::prelude::NetworkTarget;

use rpg_account::{
    account::{Account, AccountInfo},
    character::{Character, CharacterInfo, CharacterRecord},
};
use rpg_core::{
    passive_tree::PassiveSkillGraph,
    storage::UnitStorage,
    unit::{HeroInfo, Unit, UnitInfo, UnitKind},
};
use rpg_network_protocol::{protocol::*, *};

use util::fs::{open_read, open_write};

use serde_json;

use std::{env, path::Path};

#[derive(Bundle)]
pub(crate) struct RpgAccountBundle {
    pub account: RpgAccount,
}

#[derive(Debug, Deref, DerefMut, Component)]
pub(crate) struct RpgAccount(pub(crate) Account);

// FIXME there should be different message types for admin and player variants
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
            continue;
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
                    uid: net_params.state.next_uid.get(),
                },
                characters: vec![],
            };

            serde_json::to_writer(file, &account).unwrap();

            commands.spawn(RpgAccount(account));

            net_params.state.next_uid.next();

            info!("writing account file to {file_path}");
        }
    }
}

pub(crate) fn receive_account_load(
    mut commands: Commands,
    mut account_load_reader: EventReader<MessageEvent<CSLoadAccount>>,
    mut net_params: NetworkParamsRW,
) {
    for event in account_load_reader.read() {
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
                client.auth_status = AuthorizationStatus::Authenticated;
                // FIXME assign a client id and send it to the client
                client.client_type = ClientType::Player(*client_id);
                info!("spawning RpgAccount for {client:?}");

                commands.spawn(RpgAccountBundle {
                    account: RpgAccount(account.clone()),
                });

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
    mut account_q: Query<&mut RpgAccount>,
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
                let unit_info = UnitInfo::Hero(HeroInfo::new(&metadata.rpg, create_msg.game_mode));
                let mut unit = Unit::new(
                    net_params.state.next_uid.get(),
                    create_msg.class,
                    UnitKind::Hero,
                    unit_info,
                    1,
                    create_msg.name.clone(),
                    None,
                    &metadata.rpg,
                );
                unit.add_default_skills(&metadata.rpg);

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

/*
pub(crate) fn receive_character_load(
    mut character_load_reader: EventReader<MessageEvent<CSLoadCharacter>>,
    net_params: NetworkParamsRO,
) {
    for event in character_load_reader.read() {
        let client = net_params.context.clients.get(event.context()).unwrap();
        if !client.is_authenticated_player() {
            info!("unauthenticated client attempted to load character {client:?}");
            continue;
        }
        let load_msg = event.message();

    }
}*/

pub(crate) fn receive_connect_player(
    mut commands: Commands,
    mut connect_reader: EventReader<MessageEvent<CSConnectPlayer>>,
    mut net_params: NetworkParamsRW,
) {
    for player in connect_reader.read() {
        let client_id = player.context();
        let client = net_params.context.clients.get_mut(client_id).unwrap();
        if client.client_type != ClientType::Unknown {
            info!(
                "client type {:?} attempted to authorize as player while already authorized",
                client.client_type
            );
            continue;
        }

        client.client_type = ClientType::Player(*client_id);

        client.entity = commands
            .spawn((
                protocol::NetworkPlayerBundle::new(*client_id, Vec3::ZERO, Vec3::ZERO),
                TransformBundle::from_transform(
                    Transform::from_translation(Vec3::ZERO).looking_to(Vec3::NEG_Z, Vec3::Y),
                ),
            ))
            .id();

        info!("client type set to player");
    }
}

pub(crate) fn receive_connect_admin(
    mut connect_reader: EventReader<MessageEvent<CSConnectAdmin>>,
    mut context: ResMut<NetworkContext>,
) {
    for admin in connect_reader.read() {
        let client_id = admin.context();
        let client = context.clients.get_mut(client_id).unwrap();
        if client.client_type != ClientType::Unknown {
            info!(
                "client type {:?} attempted to authorize as admin while already authorized",
                client.client_type
            );
            continue;
        }

        client.client_type = ClientType::Admin(*client_id);

        info!("client type set to admin");
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
    mut create_events: EventReader<MessageEvent<CSJoinGame>>,
) {
    for create in create_events.read() {
        let client_id = create.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let create_msg = create.message();
        info!("join game {create_msg:?}");
    }
}
