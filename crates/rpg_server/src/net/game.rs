use super::server::{NetworkParamsRO, NetworkParamsRW};
use crate::{
    account::AccountInstance,
    assets::MetadataResources,
    game::plugin::{AabbResources, GameState},
};

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
    actions::{Action, ActionData, Actions, AttackData, State},
    skill::{get_skill_origin, SkillSlots, Skills},
    unit::{Hero, HeroBundle, Unit, UnitBundle},
};
use util::math::AabbComponent;

pub(crate) fn receive_player_join(
    mut join_reader: EventReader<MessageEvent<CSPlayerJoin>>,
    net_params: NetworkParamsRO,
) {
    for event in join_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        info!("player joined");
    }
}

pub(crate) fn receive_player_leave(
    mut leave_reader: EventReader<MessageEvent<CSPlayerLeave>>,
    mut net_params: NetworkParamsRW,
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
    aabbs: Res<AabbResources>,
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
        let aabb = aabbs.aabbs["hero"];

        net_params.server.send_message_to_target::<Channel1, _>(
            SCPlayerSpawn {
                position: Vec3::ZERO,
            },
            NetworkTarget::Only(vec![client_id]),
        );

        net_params.server.send_message_to_target::<Channel1, _>(
            SCSpawnHero {
                uid: unit.uid,
                position: Vec3::ZERO,
                name: unit.name.clone(),
                class: unit.class,
                level: unit.level,
                skills: character.character.skills.clone(),
                skill_slots: character.character.skill_slots.clone(),
            },
            NetworkTarget::AllExcept(vec![client_id]),
        );

        commands.entity(client.entity).insert((
            AabbComponent(aabb),
            Transform::from_translation(Vec3::ZERO),
            HeroBundle {
                unit: UnitBundle::new(
                    Unit(unit),
                    Skills(character.character.skills.clone()),
                    SkillSlots::new(character.character.skill_slots.clone()),
                ),
                hero: Hero,
            },
        ));
        // TODO ensure the player is spawned in a town
    }
}

/// Move player
// TODO
// - FIXME this needs to be sent to all characters in range of each player
//   that moves for now a naive approach will be taken
// - FIXME add action request, move message send to action handler
pub(crate) fn receive_movement(
    mut movement_reader: EventReader<MessageEvent<CSMovePlayer>>,
    net_params: NetworkParamsRO,
    mut player_q: Query<&mut Actions, With<Hero>>,
) {
    for event in movement_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let mut actions = player_q.get_mut(client.entity).unwrap();
        actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
        //info!("player move request");
    }
}

pub(crate) fn receive_movement_end(
    mut movement_reader: EventReader<MessageEvent<CSMovePlayerEnd>>,
    net_params: NetworkParamsRO,
    mut player_q: Query<&mut Actions, With<Hero>>,
) {
    for event in movement_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let mut actions = player_q.get_mut(client.entity).unwrap();
        if let Some(action) = &mut actions.movement {
            action.state = State::Finalize;
            //info!("end player move request");
        }
    }
}

/// Rotate player
pub(crate) fn receive_rotation(
    mut rotation_reader: EventReader<MessageEvent<CSRotPlayer>>,
    net_params: NetworkParamsRO,
    mut player_q: Query<&mut Actions, With<Hero>>,
) {
    for event in rotation_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let rot_msg = event.message();

        let mut actions = player_q.get_mut(client.entity).unwrap();
        actions.request(Action::new(ActionData::LookDir(rot_msg.0), None, true));
    }
}

pub(crate) fn receive_skill_use_direct(
    mut skill_use_reader: EventReader<MessageEvent<CSSkillUseDirect>>,
    net_params: NetworkParamsRO,
    metadata: Res<MetadataResources>,
    mut player_q: Query<(&Transform, &mut Actions), With<Hero>>,
) {
    for event in skill_use_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let (transform, mut actions) = player_q.get_mut(client.entity).unwrap();
        let skill_msg = event.message();
        // info!("skill use direct: {skill_msg:?}");

        let skill_target = get_skill_origin(
            &metadata.0,
            &transform,
            transform.translation, // FIXMEcursor_position.ground,
            skill_msg.0,
        );

        if actions.attack.is_none() && actions.knockback.is_none() {
            actions.request(Action::new(
                ActionData::Attack(AttackData {
                    skill_id: skill_msg.0,
                    user: transform.translation,
                    skill_target,
                }),
                None,
                true,
            ));
        }
    }
}

pub(crate) fn receive_skill_use_targeted(
    mut skill_use_reader: EventReader<MessageEvent<CSSkillUseTargeted>>,
    net_params: NetworkParamsRO,
    metadata: Res<MetadataResources>,
    mut player_q: Query<(&Transform, &mut Actions), With<Hero>>,
) {
    for event in skill_use_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let (transform, mut actions) = player_q.get_mut(client.entity).unwrap();
        let skill_msg = event.message();
        // info!("skill use targeted: {skill_msg:?}");

        let skill_target = get_skill_origin(
            &metadata.0,
            &transform,
            transform.translation, // FIXMEcursor_position.ground,
            skill_msg.skill_id,
        );

        if actions.attack.is_none() && actions.knockback.is_none() {
            actions.request(Action::new(
                ActionData::Attack(AttackData {
                    skill_id: skill_msg.skill_id,
                    user: transform.translation,
                    skill_target,
                }),
                None,
                true,
            ));
        }
    }
}

pub(crate) fn receive_item_drop(
    mut drop_reader: EventReader<MessageEvent<CSItemDrop>>,
    mut net_params: NetworkParamsRW,
    mut player_q: Query<(&Transform, &Unit), With<Hero>>,
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
    mut player_q: Query<(&Transform, &Unit), With<Hero>>,
) {
    for event in pickup_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };
    }
}
