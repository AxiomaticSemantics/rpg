use crate::server::{ClientType, NetworkContext};

use bevy::{
    ecs::{
        event::EventReader,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::info,
    math::Vec3,
    transform::{components::Transform, TransformBundle},
    utils::default,
};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use rpg_network_protocol::{protocol::*, *};

pub(crate) fn receive_connect_player(
    mut commands: Commands,
    mut connect_reader: EventReader<MessageEvent<CSConnectPlayer>>,
    mut context: ResMut<NetworkContext>,
) {
    for player in connect_reader.read() {
        let client_id = player.context();
        let Some(client) = context.clients.get_mut(client_id) else {
            continue;
        };

        if client.client_type != ClientType::Unknown {
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

//// Read client inputs and move players
pub(crate) fn movement_request(
    mut player_q: Query<(&mut Transform, &NetworkClientId)>,
    mut movement_events: EventReader<MessageEvent<CSMovePlayer>>,
    context: Res<NetworkContext>,
    mut server: ResMut<Server>,
) {
    for movement in movement_events.read() {
        let client_id = movement.context();
        let Some(client) = context.clients.get(client_id) else {
            println!("client not found");
            continue;
        };

        let ClientType::Player(id) = client.client_type else {
            println!("client not a player");
            continue;
        };

        for (mut transform, player) in &mut player_q {
            if id != player.0 {
                continue;
            }

            transform.translation = transform.translation + transform.forward() * 0.01;
            //println!("move player to {}", transform.translation);

            server
                .send_message_to_target::<Channel1, SCMovePlayer>(
                    SCMovePlayer(transform.translation),
                    NetworkTarget::Only(vec![*client_id]),
                )
                .unwrap();
        }
    }
}

//// Read client inputs and move players
pub(crate) fn rotation_request(
    mut player_q: Query<(&mut Transform, &NetworkClientId, &mut PlayerDirection)>,
    mut rotation_events: EventReader<MessageEvent<CSRotPlayer>>,
    context: Res<NetworkContext>,
    mut server: ResMut<Server>,
) {
    for rotation in rotation_events.read() {
        let client_id = rotation.context();
        let Some(client) = context.clients.get(client_id) else {
            println!("client not found");
            continue;
        };

        let ClientType::Player(id) = client.client_type else {
            println!("client not a player");
            continue;
        };

        for (mut transform, player, mut direction) in &mut player_q {
            if player.0 != id {
                continue;
            }

            direction.0 = rotation.message().0;
            transform.look_to(direction.0, Vec3::Y);

            server
                .send_message_to_target::<Channel1, SCRotPlayer>(
                    SCRotPlayer(direction.0),
                    NetworkTarget::Only(vec![*client_id]),
                )
                .unwrap();
        }
    }
}

/*
server.send_message_to_target::<Channel1, _>(message, NetworkTarget::All)
    .unwrap_or_else(|e| {
        error!("Failed to send message: {:?}", e);
    });
}
*/
