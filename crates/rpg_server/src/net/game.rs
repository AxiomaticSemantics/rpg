use super::server::{ClientType, NetworkContext, NetworkParamsRO, NetworkParamsRW};
use crate::{account::AccountInstance, game::plugin::GameState};

use bevy::{
    ecs::{
        event::EventReader,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    log::info,
    math::Vec3,
    transform::components::Transform,
};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use rpg_network_protocol::protocol::*;
use rpg_util::unit::{Hero, Unit, UnitBundle};

pub(crate) fn receive_player_ready(
    mut commands: Commands,
    mut ready_reader: EventReader<MessageEvent<CSPlayerReady>>,
    mut net_params: NetworkParamsRW,
    game_state: Res<GameState>,
    mut account_q: Query<&AccountInstance>,
) {
    for event in ready_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        info!("spawning player");

        let account = account_q.get(client.entity).unwrap();

        let id_info = game_state
            .get_id_info_from_account_id(account.0.info.id)
            .unwrap();

        let character = account
            .get_character_from_uid(id_info.character_id)
            .unwrap();
        let unit = character.character.unit.clone();

        commands.entity(client.entity).insert((
            Transform::from_translation(Vec3::ZERO).looking_to(Vec3::NEG_Z, Vec3::Y),
            UnitBundle { unit: Unit(unit) },
            Hero,
        ));

        net_params.server.send_message_to_target::<Channel1, _>(
            SCPlayerSpawn {
                position: Vec3::ZERO,
                direction: Vec3::NEG_Z,
            },
            NetworkTarget::Only(vec![client_id]),
        );
    }
}

/// Move player
pub(crate) fn receive_movement(
    mut movement_reader: EventReader<MessageEvent<CSMovePlayer>>,
    mut net_params: NetworkParamsRW,
    mut player_q: Query<(&mut Transform, &Unit, &AccountInstance), With<Hero>>,
) {
    for event in movement_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        for (mut transform, player, account) in &mut player_q {
            if client.account_id.unwrap() != account.info.id {
                continue;
            }

            transform.translation = transform.translation + transform.forward() * 0.01;
            //info!("move player to {}", transform.translation);

            net_params
                .server
                .send_message_to_target::<Channel1, SCMovePlayer>(
                    SCMovePlayer(transform.translation),
                    NetworkTarget::Only(vec![client_id]),
                )
                .unwrap();
        }
    }
}

/// Rotate player
pub(crate) fn receive_rotation(
    mut rotation_reader: EventReader<MessageEvent<CSRotPlayer>>,
    mut net_params: NetworkParamsRW,
    mut player_q: Query<(&mut Transform, &Unit, &AccountInstance), With<Hero>>,
) {
    for event in rotation_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        for (mut transform, player, account) in &mut player_q {
            if client.account_id.unwrap() != account.info.id {
                continue;
            }

            let rot_msg = event.message();
            transform.look_to(rot_msg.0, Vec3::Y);

            net_params
                .server
                .send_message_to_target::<Channel1, SCRotPlayer>(
                    SCRotPlayer(rot_msg.0),
                    NetworkTarget::Only(vec![client_id]),
                )
                .unwrap();
        }
    }
}

pub(crate) fn receive_skill_use_direct(
    mut skill_use_reader: EventReader<MessageEvent<CSSkillUseDirect>>,
    mut net_params: NetworkParamsRW,
    mut player_q: Query<(&mut Transform, &Unit, &AccountInstance), With<Hero>>,
) {
    for event in skill_use_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        for (mut transform, player, account) in &mut player_q {
            if client.account_id.unwrap() != account.info.id {
                continue;
            }

            let skill_use_msg = event.message();
            info!("{skill_use_msg:?}");
        }
    }
}

pub(crate) fn receive_skill_use_targeted(
    mut skill_use_reader: EventReader<MessageEvent<CSSkillUseTargeted>>,
    mut net_params: NetworkParamsRW,
    mut player_q: Query<(&mut Transform, &Unit, &AccountInstance), With<Hero>>,
) {
    for event in skill_use_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        for (mut transform, player, account) in &mut player_q {
            if client.account_id.unwrap() != account.info.id {
                continue;
            }
            let skill_use_msg = event.message();
        }
    }
}
