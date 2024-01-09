use crate::server::{ClientType, NetworkContext};

use bevy::{
    ecs::{
        event::EventReader,
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
    math::Vec3,
    transform::{components::Transform, TransformBundle},
};

use lightyear::prelude::server::*;

use rpg_network_protocol::{protocol::*, *};

use util::fs::{open_read, open_write};

use std::{fs::Metadata, path::Path};

pub(crate) fn receive_account_create(
    mut commands: Commands,
    mut account_create_reader: EventReader<MessageEvent<CSCreateAccount>>,
    mut context: ResMut<NetworkContext>,
) {
    for event in account_create_reader.read() {
        let file_path = format!("save/server/accounts/{}", event.message().name);
        let path = Path::new(file_path.as_str());
        let file = open_read(path);

        if let Ok(file) = file {
            let fs_meta = file.metadata().unwrap();
        } else {
            let Ok(file) = open_write(path) else {
                info!("unable to open account file for writing");
                continue;
            };

            info!("writing account file to {file_path}");
            // Write account data
        }
    }
}

pub(crate) fn receive_account_load(
    mut commands: Commands,
    mut account_load_reader: EventReader<MessageEvent<CSLoadAccount>>,
    mut context: ResMut<NetworkContext>,
) {
    for event in account_load_reader.read() {
        //
    }
}

pub(crate) fn receive_character_create(
    mut commands: Commands,
    mut character_create_reader: EventReader<MessageEvent<CSCreateCharacter>>,
    mut context: ResMut<NetworkContext>,
) {
    for event in character_create_reader.read() {
        //
    }
}

pub(crate) fn receive_character_load(
    mut commands: Commands,
    mut character_load_reader: EventReader<MessageEvent<CSLoadCharacter>>,
    mut context: ResMut<NetworkContext>,
) {
    for event in character_load_reader.read() {
        //
    }
}

pub(crate) fn receive_connect_player(
    mut commands: Commands,
    mut connect_reader: EventReader<MessageEvent<CSConnectPlayer>>,
    mut context: ResMut<NetworkContext>,
) {
    for player in connect_reader.read() {
        let client_id = player.context();
        let client = context.clients.get_mut(client_id).unwrap();
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
