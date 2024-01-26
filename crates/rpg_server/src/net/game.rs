use super::server::{ClientType, NetworkContext, NetworkParamsRW};
use crate::{account::AccountInstance, assets::MetadataResources, game::plugin::GameState};

use bevy::{
    ecs::{
        event::EventReader,
        query::With,
        system::{Commands, Query, Res},
    },
    log::info,
    math::Vec3,
    transform::components::Transform,
};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::{Action, ActionData, Actions, AttackData},
    skill::get_skill_origin,
    unit::{Hero, HeroBundle, Unit, UnitBundle},
};
use util::math::{Aabb, AabbComponent};

pub(crate) fn receive_player_join(
    mut commands: Commands,
    mut join_reader: EventReader<MessageEvent<CSPlayerJoin>>,
    mut net_params: NetworkParamsRW,
    game_state: Res<GameState>,
    mut account_q: Query<&AccountInstance>,
) {
    for event in join_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        info!("play join");
    }
}

pub(crate) fn receive_player_leave(
    mut commands: Commands,
    mut leave_reader: EventReader<MessageEvent<CSPlayerLeave>>,
    mut net_params: NetworkParamsRW,
    game_state: Res<GameState>,
    mut account_q: Query<&AccountInstance>,
) {
    for event in leave_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        info!("player leave");
    }
}

/// This is received when the client has completed loading the world and is ready to be spawned
pub(crate) fn receive_player_loaded(
    mut commands: Commands,
    mut ready_reader: EventReader<MessageEvent<CSClientReady>>,
    mut net_params: NetworkParamsRW,
    game_state: Res<GameState>,
    account_q: Query<&AccountInstance>,
) {
    for event in ready_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let account = account_q.get(client.entity).unwrap();

        let id_info = game_state
            .get_id_info_from_account_id(account.0.info.id)
            .unwrap();

        let character = account
            .get_character_from_uid(id_info.character_id)
            .unwrap();

        info!("spawning player: {:?} {id_info:?}", account.0.info);

        let unit = character.character.unit.clone();
        let aabb = Aabb::from_min_max(Vec3::new(-0.3, 0.0, -0.2), Vec3::new(0.3, 1.2, 0.2));
        commands.entity(client.entity).insert((
            AabbComponent(aabb),
            Transform::from_translation(Vec3::ZERO).looking_to(Vec3::NEG_Z, Vec3::Y),
            HeroBundle {
                unit: UnitBundle::new(Unit(unit)),
                hero: Hero,
            },
        ));

        net_params.server.send_message_to_target::<Channel1, _>(
            SCPlayerSpawn {
                position: Vec3::ZERO,
                direction: Vec3::NEG_Z,
            },
            NetworkTarget::Only(vec![client_id]),
        );

        // TODO ensure the player is spawned in a town

        // TODO send message to all connected players
    }
}

/// Move player
// TODO
// - FIXME this needs to be sent to all characters in range of each player
//   that moves for now a naive approach will be taken
// - FIXME add action request, move message send to action handler
pub(crate) fn receive_movement(
    mut movement_reader: EventReader<MessageEvent<CSMovePlayer>>,
    mut net_params: NetworkParamsRW,
    mut player_q: Query<(&mut Transform, &Unit, &mut Actions, &AccountInstance), With<Hero>>,
) {
    for event in movement_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let Ok((mut transform, player, mut actions, account)) = player_q.get_mut(client.entity)
        else {
            info!("{client:?}");
            continue;
        };

        actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
        info!("move request {}", transform.translation);
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
    metadata: Res<MetadataResources>,
    mut player_q: Query<(&mut Transform, &Unit, &mut Actions, &AccountInstance), With<Hero>>,
) {
    for event in skill_use_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        for (mut transform, player, mut actions, account) in &mut player_q {
            if client.account_id.unwrap() != account.info.id {
                continue;
            }

            let skill_use_msg = event.message();
            info!("{skill_use_msg:?}");

            let (origin, target) = get_skill_origin(
                &metadata.0,
                &transform,
                transform.translation, // FIXMEcursor_position.ground,
                skill_use_msg.0,
            );

            if actions.attack.is_none() && actions.knockback.is_none() {
                actions.request(Action::new(
                    ActionData::Attack(AttackData {
                        skill_id: skill_use_msg.0,
                        user: transform.translation,
                        origin,
                        target,
                    }),
                    None,
                    true,
                ));
                //
            }
        }
    }
}

pub(crate) fn receive_item_drop(
    mut drop_reader: EventReader<MessageEvent<CSItemDrop>>,
    mut net_params: NetworkParamsRW,
    mut player_q: Query<(&mut Transform, &Unit, &AccountInstance), With<Hero>>,
) {
    for event in drop_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };
    }
}

pub(crate) fn receive_item_pickup(
    mut pickup_reader: EventReader<MessageEvent<CSItemPickup>>,
    mut net_params: NetworkParamsRW,
    mut player_q: Query<(&mut Transform, &Unit, &AccountInstance), With<Hero>>,
) {
    for event in pickup_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };
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
            info!("{skill_use_msg:?}");

            //
        }
    }
}
