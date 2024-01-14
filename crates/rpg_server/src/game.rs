use crate::server::{ClientType, NetworkContext, NetworkParamsRO, NetworkParamsRW};

use bevy::{
    ecs::{
        event::EventReader,
        system::{Query, Res, ResMut},
    },
    log::info,
    math::Vec3,
    transform::components::Transform,
};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use rpg_network_protocol::protocol::*;

/// Move player
pub(crate) fn movement_request(
    mut player_q: Query<(&mut Transform, &NetworkClientId)>,
    mut movement_events: EventReader<MessageEvent<CSMovePlayer>>,
    mut net_params: NetworkParamsRW,
) {
    for movement in movement_events.read() {
        let client_id = movement.context();
        let client = net_params.context.clients.get(client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        for (mut transform, player) in &mut player_q {
            if *client_id != player.0 {
                continue;
            }

            transform.translation = transform.translation + transform.forward() * 0.01;
            //println!("move player to {}", transform.translation);

            net_params
                .server
                .send_message_to_target::<Channel1, SCMovePlayer>(
                    SCMovePlayer(transform.translation),
                    NetworkTarget::Only(vec![*client_id]),
                )
                .unwrap();
        }
    }
}

/// Rotate player
pub(crate) fn rotation_request(
    mut player_q: Query<(&mut Transform, &NetworkClientId, &mut PlayerDirection)>,
    mut rotation_events: EventReader<MessageEvent<CSRotPlayer>>,
    context: Res<NetworkContext>,
    mut server: ResMut<Server>,
) {
    for rotation in rotation_events.read() {
        let client_id = rotation.context();
        let client = context.clients.get(client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        for (mut transform, player, mut direction) in &mut player_q {
            if *client_id != player.0 {
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
