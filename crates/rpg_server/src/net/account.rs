use crate::{
    account::{
        AccountInstance, AccountInstanceBundle, AdminAccountInstance, AdminAccountInstanceBundle,
    },
    assets::MetadataResources,
    game::plugin::{GameState, PlayerIdInfo},
    server_state::ServerMetadataResource,
    state::AppState,
};

use super::{client::ClientType, server::NetworkParamsRW};

use bevy::{
    ecs::{
        event::EventReader,
        schedule::NextState,
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
    math::Vec3,
    transform::components::Transform,
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
    unit::{HeroInfo, Unit as RpgUnit, UnitInfo, UnitKind},
};
use rpg_network_protocol::protocol::*;
use rpg_util::unit::Unit;

use util::fs::{open_read, open_write};

use serde_json;

use std::{env, path::Path};

pub(crate) fn receive_account_create(
    mut commands: Commands,
    mut server_metadata: ResMut<ServerMetadataResource>,
    mut account_create_reader: EventReader<MessageEvent<CSCreateAccount>>,
    mut net_params: NetworkParamsRW,
) {
    for event in account_create_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get_mut(&client_id).unwrap();
        if client.is_authenticated_player() {
            info!("already authenticated client attempted to create account {client:?}");
            continue;
        }

        // Allow authenticated admins to create accounts
        if client.is_admin() && !client.is_authenticated() {
            panic!("unauthenticated admin: {client:?}");
        }

        let account_file_path = format!(
            "{}/server/accounts/{}.json",
            std::env::var("RPG_SAVE_ROOT").unwrap(),
            event.message().name
        );
        let account_path = Path::new(account_file_path.as_str());
        let account_file = open_read(account_path);

        let meta_file_path = format!(
            "{}/server/meta.json",
            std::env::var("RPG_SAVE_ROOT").unwrap(),
        );
        let meta_path = Path::new(meta_file_path.as_str());
        let meta_file = open_write(meta_path);

        let Ok(meta_file) = meta_file else {
            info!("cannot open {meta_file_path} for writing");
            return;
        };

        if let Ok(_) = account_file {
            info!("account already exists");
        } else {
            let Ok(account_file) = open_write(account_path) else {
                info!("unable to open account file for writing");
                continue;
            };

            let account = Account {
                info: AccountInfo {
                    character_slots: 12,
                    name: event.message().name.clone(),
                    id: server_metadata.0.next_account_id,
                    selected_slot: None,
                },
                characters: vec![],
            };

            // finally update the server metadata
            server_metadata.0.next_account_id.0 += 1;

            info!("writing account file to {account_file_path}");
            serde_json::to_writer(meta_file, &server_metadata.0).unwrap();
            serde_json::to_writer(account_file, &account).unwrap();

            // Set the newly created account to be autenticated
            client.client_type = ClientType::Player;
            client.account_id = Some(account.info.id);
            info!("spawning account for {client:?}");

            let account_entity = commands
                .spawn(AccountInstanceBundle {
                    account: AccountInstance(account.clone()),
                })
                .id();

            client.entity = account_entity;

            net_params.server.send_message_to_target::<Channel1, _>(
                SCCreateAccountSuccess(account.clone()),
                NetworkTarget::Only(vec![client_id]),
            );
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
            env::var("RPG_SAVE_ROOT").unwrap(),
            event.message().name
        );
        let path = Path::new(file_path.as_str());
        let file = open_read(path);

        let Ok(file) = file else {
            info!("account does not exist {client:?}");
            continue;
        };

        let account: Result<AdminAccount, _> = serde_json::from_reader(file);
        if let Ok(account) = account {
            // FIXME assign a client id and send it to the client
            client.client_type = ClientType::Player;
            client.account_id = Some(account.info.id);
            info!("spawning admin account for {client:?}");

            let account_entity = commands
                .spawn(AdminAccountInstanceBundle {
                    account: AdminAccountInstance(account.clone()),
                })
                .id();

            client.entity = account_entity;

            /* TODO disabled for now
            net_params.server.send_message_to_target::<Channel1, _>(
                SCLoginAdminAccountSuccess(account.clone()),
                NetworkTarget::Only(vec![*client_id]),
            );

            commands.spawn(AdminAccountInstance(account));
            */
        } else {
            info!("unable to deserialize account: {file_path}");
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
                info!("spawning account for {client:?}");

                let account_entity = commands
                    .spawn(AccountInstanceBundle {
                        account: AccountInstance(account.clone()),
                    })
                    .id();

                client.entity = account_entity;

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
    mut server_metadata: ResMut<ServerMetadataResource>,
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
                let mut unit = RpgUnit::new(
                    server_metadata.0.next_uid.get(),
                    create_msg.class,
                    UnitKind::Hero,
                    unit_info,
                    1,
                    create_msg.name.clone(),
                    None,
                    &metadata.0,
                );
                unit.add_default_skills(&metadata.0);

                server_metadata.0.next_uid.next();

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

                let meta_file_path = format!(
                    "{}/server/meta.json",
                    std::env::var("RPG_SAVE_ROOT").unwrap(),
                );
                let meta_path = Path::new(meta_file_path.as_str());
                let meta_file = open_write(meta_path);

                let Ok(meta_file) = meta_file else {
                    info!("cannot open {meta_file_path} for writing");
                    return;
                };

                serde_json::to_writer(file, &account.0).unwrap();
                serde_json::to_writer(meta_file, &server_metadata.0).unwrap();
            }
        }
    }
}

pub(crate) fn receive_game_create(
    mut state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    mut net_params: NetworkParamsRW,
    mut create_events: EventReader<MessageEvent<CSCreateGame>>,
    mut account_q: Query<&mut AccountInstance>,
) {
    for create in create_events.read() {
        let client_id = *create.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let create_msg = create.message();
        info!("create game {create_msg:?}");

        let mut account = account_q.get_mut(client.entity).unwrap();

        let Some(character) = account.get_character_from_slot(create_msg.slot) else {
            info!("no character in slot");
            continue;
        };

        // FIXME temporarily clear, this will be handled properly later
        game_state.players.clear();
        game_state.players.push(PlayerIdInfo {
            slot: create_msg.slot,
            account_id: account.0.info.id,
            character_id: character.info.uid,
            client_id,
            entity: client.entity,
        });
        game_state.options.mode = create_msg.game_mode;

        net_params.server.send_message_to_target::<Channel1, _>(
            SCGameCreateSuccess,
            NetworkTarget::Only(vec![client_id]),
        );

        account.info.selected_slot = Some(create_msg.slot);

        // FIXME turn this into an event
        state.set(AppState::SpawnSimulation);
    }
}

// FIXME move to net/game.rs
pub(crate) fn receive_game_join(
    mut game_state: ResMut<GameState>,
    mut net_params: NetworkParamsRW,
    mut join_events: EventReader<MessageEvent<CSJoinGame>>,
    account_q: Query<&AccountInstance>,
    unit_q: Query<(&Unit, &Transform)>,
) {
    for join in join_events.read() {
        let client_id = *join.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let join_msg = join.message();
        info!("join game {join_msg:?}");

        let account = account_q.get(client.entity).unwrap();
        if game_state
            .players
            .iter()
            .any(|a| a.account_id == account.info.id)
        {
            info!("client attempted to join a game while in a game");
            net_params.server.send_message_to_target::<Channel1, _>(
                SCGameJoinError,
                NetworkTarget::Only(vec![client_id]),
            );

            continue;
        }

        let Some(character) = account.get_character_from_slot(join_msg.slot) else {
            info!("no character exists in slot");
            continue;
        };

        /*let clients = game_state
        .players
        .iter()
        .map(|p| p.client_id)
        .collect::<Vec<_>>();*/

        for player in game_state.players.iter() {
            let (unit, unit_transform) = unit_q.get(player.entity).unwrap();

            net_params.server.send_message_to_target::<Channel1, _>(
                SCSpawnHero {
                    position: unit_transform.translation,
                    uid: unit.uid,
                    name: unit.name.clone(),
                    level: unit.level,
                    class: unit.class,
                },
                NetworkTarget::Only(vec![client.id]),
            );
        }

        game_state.players.push(PlayerIdInfo {
            slot: join_msg.slot,
            account_id: account.0.info.id,
            character_id: character.info.uid,
            client_id,
            entity: client.entity,
        });

        net_params.server.send_message_to_target::<Channel1, _>(
            SCGameJoinSuccess,
            NetworkTarget::Only(vec![client_id]),
        );

        net_params.server.send_message_to_target::<Channel1, _>(
            SCSpawnHero {
                position: Vec3::ZERO,
                uid: character.info.uid,
                class: character.character.unit.class,
                level: character.character.unit.level,
                name: character.character.unit.name.clone(),
            },
            NetworkTarget::AllExcept(vec![client_id]),
        );
    }
}
