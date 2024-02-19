use super::{
    client::ClientType,
    server::{ClientMessageEvent, NetworkParamsRW},
};
use crate::{
    account::{
        AccountInstance, AccountInstanceBundle, AdminAccountInstance, AdminAccountInstanceBundle,
    },
    assets::MetadataResources,
    game::plugin::{GameState, PlayerIdInfo},
    server_state::ServerMetadataResource,
    state::AppState,
    world::LoadZone,
};

use bevy::{
    ecs::{
        event::{EventReader, EventWriter},
        schedule::NextState,
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
    math::Vec3,
};

use rpg_account::{
    account::{Account, AccountInfo, AdminAccount, AdminAccountInfo},
    account_statistics::AccountStatistics,
    character::{Character, CharacterInfo, CharacterRecord},
    character_statistics::CharacterStatistics,
};
use rpg_core::{
    passive_tree::UnitPassiveSkills,
    skill::{SkillSlot, SkillSlotId},
    storage::UnitStorage,
    unit::{HeroInfo, Unit as RpgUnit, UnitInfo, UnitKind},
};
use rpg_network_protocol::protocol::*;
use rpg_world::zone::ZoneId;

use util::fs::{open_read, open_write};

use std::{env, path::Path};

pub(crate) fn receive_account_create(
    mut commands: Commands,
    mut server_metadata: ResMut<ServerMetadataResource>,
    mut account_create_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
) {
    for event in account_create_reader.read() {
        let ClientMessage::CSCreateAccount(msg) = &event.message else {
            continue;
        };
        let client = net_params
            .context
            .clients
            .get_mut(&event.client_id)
            .unwrap();
        if client.is_authenticated_player() {
            info!("already authenticated client attempted to create account {client:?}");
            continue;
        }

        // Allow authenticated admins to create accounts
        if client.is_admin() && !client.is_authenticated() {
            panic!("unauthenticated admin: {client:?}");
        }

        let account_file_path = format!(
            "{}/server/accounts/{}.bin",
            std::env::var("RPG_SAVE_ROOT").unwrap(),
            msg.name
        );
        let account_path = Path::new(account_file_path.as_str());
        let account_file = open_read(account_path);

        let meta_file_path = format!(
            "{}/server/meta.bin",
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
                    name: msg.name.clone(),
                    id: server_metadata.0.next_account_id,
                    selected_slot: None,
                },
                statistics: AccountStatistics::default(),
                characters: vec![],
            };

            // finally update the server metadata
            server_metadata.0.next_account_id.0 += 1;

            info!("writing account file to {account_file_path}");

            bincode::serialize_into(meta_file, &server_metadata.0).unwrap();
            bincode::serialize_into(account_file, &account).unwrap();

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

            let message = bincode::serialize(&ServerMessage::SCCreateAccountSuccess(
                SCCreateAccountSuccess(account.clone()),
            ))
            .unwrap();

            net_params
                .server
                .send_message(event.client_id, ServerChannel::Message, message);
        }
    }
}

pub(crate) fn receive_admin_login(
    mut commands: Commands,
    mut login_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
) {
    for event in login_reader.read() {
        let ClientMessage::CSLoadAdminAccount(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get_mut(&client_id).unwrap();
        if client.is_authenticated_player() {
            info!("authenticated player attempted to login to account {client:?}");
            continue;
        } else if client.is_authenticated_admin() {
            info!("authenticated admin attempted to login to account {client:?}");
            continue;
        }

        let file_path = format!(
            "{}/server/admin_accounts/{}.bin",
            env::var("RPG_SAVE_ROOT").unwrap(),
            msg.name
        );
        let path = Path::new(file_path.as_str());
        let file = open_read(path);

        let Ok(file) = file else {
            info!("account does not exist {client:?}");
            continue;
        };

        let account: Result<AdminAccount, _> = bincode::deserialize_from(file);
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
    mut login_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
) {
    for event in login_reader.read() {
        let ClientMessage::CSLoadAccount(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get_mut(&client_id).unwrap();
        if client.is_authenticated_player() {
            info!("authenticated player attempted to load account {client:?}");
            continue;
        }

        let file_path = format!(
            "{}/server/accounts/{}.bin",
            env::var("RPG_SAVE_ROOT").unwrap(),
            msg.name
        );
        let path = Path::new(file_path.as_str());
        let file = open_read(path);

        if let Ok(file) = file {
            let account: Result<Account, _> = bincode::deserialize_from(file);
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

                let message = bincode::serialize(&ServerMessage::SCLoginAccountSuccess(
                    SCLoginAccountSuccess(account.clone()),
                ))
                .unwrap();
                net_params
                    .server
                    .send_message(client_id, ServerChannel::Message, message);
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
    mut character_create_reader: EventReader<ClientMessageEvent>,
    mut net_params: NetworkParamsRW,
    mut account_q: Query<&mut AccountInstance>,
) {
    for event in character_create_reader.read() {
        let ClientMessage::CSCreateCharacter(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.get_client_from_id(client_id).unwrap();
        if !client.is_authenticated_player() {
            info!("unauthenticated client attempted to create character {client:?}");
            continue;
        }

        for mut account in &mut account_q {
            if account.0.characters.iter().any(|c| c.info.slot == msg.slot) {
                info!("character already exists");

                let message =
                    bincode::serialize(&ServerMessage::SCCreateAccountError(SCCreateAccountError))
                        .unwrap();
                net_params
                    .server
                    .send_message(client_id, ServerChannel::Message, message);
            } else {
                let unit_info = UnitInfo::Hero(HeroInfo::new(&metadata.rpg, msg.game_mode));
                let mut unit = RpgUnit::new(
                    server_metadata.0.next_uid.get(),
                    msg.class,
                    UnitKind::Hero,
                    unit_info,
                    1,
                    msg.name.clone(),
                    &metadata.rpg,
                );
                let mut skills = Vec::new();
                unit.add_default_skills(&mut skills, &metadata.rpg);
                let skill_slots = vec![SkillSlot::new(SkillSlotId(0), Some(skills[0].id))];

                server_metadata.0.next_uid.next();

                let character_info = CharacterInfo {
                    name: msg.name.clone(),
                    slot: msg.slot,
                    uid: unit.uid,
                    game_mode: msg.game_mode,
                };

                let character = CharacterRecord {
                    info: character_info,
                    statistics: CharacterStatistics::default(),
                    character: Character {
                        unit,
                        skills,
                        skill_slots,
                        passive_tree: UnitPassiveSkills::new(msg.class),
                        storage: UnitStorage::default(),
                        waypoints: vec![ZoneId(0)],
                    },
                };

                let message = bincode::serialize(&ServerMessage::SCCreateCharacterSuccess(
                    SCCreateCharacterSuccess(character.clone()),
                ))
                .unwrap();

                net_params
                    .server
                    .send_message(client_id, ServerChannel::Message, message);

                account.0.characters.push(character);

                let file_path = format!(
                    "{}/server/accounts/{}.bin",
                    env::var("RPG_SAVE_ROOT").unwrap(),
                    account.0.info.name
                );
                let path = Path::new(file_path.as_str());
                let Ok(file) = open_write(path) else {
                    info!("unable to open account file for writing");
                    continue;
                };

                let meta_file_path = format!(
                    "{}/server/meta.bin",
                    std::env::var("RPG_SAVE_ROOT").unwrap(),
                );
                let meta_path = Path::new(meta_file_path.as_str());
                let meta_file = open_write(meta_path);

                let Ok(meta_file) = meta_file else {
                    info!("cannot open {meta_file_path} for writing");
                    return;
                };

                bincode::serialize_into(file, &account.0).unwrap();
                bincode::serialize_into(meta_file, &server_metadata.0).unwrap();
            }
        }
    }
}

pub(crate) fn receive_game_create(
    mut state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    mut load_writer: EventWriter<LoadZone>,
    mut net_params: NetworkParamsRW,
    mut create_events: EventReader<ClientMessageEvent>,
    mut account_q: Query<&mut AccountInstance>,
) {
    for event in create_events.read() {
        let ClientMessage::CSCreateGame(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        if !game_state.players.is_empty() {
            // TODO only a single game instance is currently supported
            info!("game already active");
            continue;
        }

        info!("create game {msg:?}");

        let mut account = account_q.get_mut(client.entity).unwrap();

        let Some(character) = account.get_character_from_slot(msg.slot) else {
            info!("no character in slot");
            continue;
        };

        let hero_info = character.character.unit.info.hero();
        if hero_info.game_mode != msg.game_mode {
            info!(
                "a {:?} character cannot create a {:?} game",
                hero_info.game_mode, msg.game_mode
            );
            continue;
        }

        // add the creator to the player list
        game_state.players.push(PlayerIdInfo {
            slot: msg.slot,
            account_id: account.0.info.id,
            character_id: character.info.uid,
            client_id,
            entity: client.entity,
        });
        game_state.options.max_players = 8;
        game_state.options.mode = msg.game_mode;

        let message = bincode::serialize(&ServerMessage::SCGameCreateSuccess(SCGameCreateSuccess(
            msg.game_mode,
        )))
        .unwrap();
        net_params
            .server
            .send_message(client_id, ServerChannel::Message, message);

        account.info.selected_slot = Some(msg.slot);

        load_writer.send(LoadZone(ZoneId(0)));

        state.set(AppState::SpawnSimulation); // TODO NEEDED?
    }
}

// FIXME move to net/game.rs
pub(crate) fn receive_game_join(
    mut game_state: ResMut<GameState>,
    mut net_params: NetworkParamsRW,
    mut join_events: EventReader<ClientMessageEvent>,
    account_q: Query<&AccountInstance>,
) {
    for event in join_events.read() {
        let ClientMessage::CSJoinGame(msg) = &event.message else {
            continue;
        };

        let client_id = event.client_id;
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        // If the game is full reject new joins
        if game_state.players.len() >= game_state.options.max_players as usize {
            info!("game full");
            join_events.clear();
            return;
        }

        info!("join game {msg:?}");

        let account = account_q.get(client.entity).unwrap();
        if game_state
            .players
            .iter()
            .any(|a| a.account_id == account.info.id)
        {
            info!("client attempted to join a game while in a game");
            let message =
                bincode::serialize(&ServerMessage::SCGameJoinError(SCGameJoinError)).unwrap();
            net_params
                .server
                .send_message(client_id, ServerChannel::Message, message);

            continue;
        }

        let Some(character) = account.get_character_from_slot(msg.slot) else {
            info!("no character exists in slot");
            continue;
        };

        // Ensure the player is of the correct type to join the game
        if character.info.game_mode != game_state.options.mode {
            info!(
                "a {:?} player attemped to join a {:?} game",
                character.info.game_mode, game_state.options.mode
            );
        }

        // TODO optimize
        // spawn the new player on all connected players
        let message = bincode::serialize(&ServerMessage::SCSpawnHero(SCSpawnHero {
            position: Vec3::ZERO,
            uid: character.info.uid,
            class: character.character.unit.class,
            level: character.character.unit.level,
            name: character.character.unit.name.clone(),
            skills: character.character.skills.clone(),
            deaths: None,
            skill_slots: character.character.skill_slots.clone(),
        }))
        .unwrap();

        net_params
            .server
            .broadcast_message_except(client_id, ServerChannel::Message, message);

        game_state.players.push(PlayerIdInfo {
            slot: msg.slot,
            account_id: account.0.info.id,
            character_id: character.info.uid,
            client_id,
            entity: client.entity,
        });

        let message = bincode::serialize(&ServerMessage::SCGameJoinSuccess(SCGameJoinSuccess(
            game_state.options.mode,
        )))
        .unwrap();
        net_params
            .server
            .send_message(client_id, ServerChannel::Message, message);
    }
}
