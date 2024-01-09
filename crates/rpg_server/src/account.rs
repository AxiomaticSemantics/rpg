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
    character::{Character, CharacterInfo},
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
        if client.is_authenticated_admin() {}

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
                    name: event.message().name.clone(),
                    uid: net_params.state.next_uid.get(),
                    character_info: vec![],
                },
                characters: vec![],
            };

            serde_json::to_writer(file, &account).unwrap();

            net_params.state.next_uid.next();

            info!("writing account file to {file_path}");
            // Write account data
        }
    }
}

pub(crate) fn receive_account_load(
    mut commands: Commands,
    mut account_load_reader: EventReader<MessageEvent<CSLoadAccount>>,
    mut net_params: NetworkParamsRW,
) {
    for event in account_load_reader.read() {
        let client = net_params.context.clients.get_mut(event.context()).unwrap();
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
                info!("spawning RpgAccount for {client:?}");

                commands.spawn(RpgAccountBundle {
                    account: RpgAccount(account),
                });
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
                .info
                .character_info
                .iter()
                .any(|c| c.name == event.message().name)
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

                account.0.info.character_info.push(CharacterInfo {
                    name: create_msg.name.clone(),
                    uid: unit.uid,
                    hero_mode: create_msg.game_mode,
                });
                account.0.characters.push(Character {
                    unit,
                    passive_tree: PassiveSkillGraph::new(create_msg.class),
                    storage: UnitStorage::default(),
                });

                net_params.state.next_uid.next();

                net_params
                    .server
                    .send_message_to_target::<Channel1, SCCreateAccountSuccess>(
                        SCCreateAccountSuccess(account.0.info.clone()),
                        NetworkTarget::Only(vec![*client_id]),
                    )
                    .unwrap();
            }
        }
    }
}

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
    }
}

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
