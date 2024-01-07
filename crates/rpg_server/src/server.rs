use crate::Transports;

use bevy::{
    app::{
        App, FixedUpdate, Plugin, PluginGroup, PreUpdate, ScheduleRunnerPlugin, Startup, Update,
    },
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
    math::Vec3,
    transform::{components::Transform, TransformBundle},
    utils::default,
    MinimalPlugins,
};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use rpg_network_protocol::{protocol::*, *};

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

pub(crate) struct NetworkServerPlugin {
    pub(crate) port: u16,
    pub(crate) transport: Transports,
}

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), self.port);
        let netcode_config = NetcodeConfig::default()
            .with_protocol_id(PROTOCOL_ID)
            .with_key(KEY);

        /*let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(10),
            incoming_jitter: Duration::from_millis(20),
            incoming_loss: 0.05,
        };*/

        let transport = TransportConfig::UdpSocket(server_addr);
        let io = Io::from_config(IoConfig::from_transport(transport));
        //.with_conditioner(link_conditioner));

        let config = ServerConfig {
            shared: shared_config(),
            netcode: netcode_config,
            ping: PingConfig::default(),
        };
        let plugin_config = PluginConfig::new(config, io, protocol());

        app.add_plugins(server::ServerPlugin::new(plugin_config))
            .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                Duration::from_secs_f64(1.0 / 60.0),
            )))
            .init_resource::<NetworkContext>()
            .add_systems(
                FixedUpdate,
                (rotation_request, movement_request)
                    .chain()
                    .in_set(FixedUpdateSet::Main),
            )
            .add_systems(PreUpdate, (handle_connections, handle_disconnections))
            .add_systems(Update, connect_player);
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum ClientType {
    #[default]
    Unknown,
    Player(ClientId),
    Admin(ClientId),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Client {
    pub(crate) entity: Entity,
    pub(crate) client_type: ClientType,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            client_type: ClientType::Unknown,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct NetworkContext {
    pub clients: HashMap<ClientId, Client>,
}

pub(crate) fn handle_disconnections(
    mut disconnections: EventReader<DisconnectEvent>,
    mut context: ResMut<NetworkContext>,
    mut commands: Commands,
) {
    for disconnection in disconnections.read() {
        let client_id = disconnection.context();
        info!("Removing {client_id} from global map");
        if let Some(client) = context.clients.remove(client_id) {
            if client.entity != Entity::PLACEHOLDER {
                commands.entity(client.entity).despawn_recursive();
            }
        }
    }
}

pub(crate) fn handle_connections(
    mut connections: EventReader<ConnectEvent>,
    mut context: ResMut<NetworkContext>,
    mut server: ResMut<Server>,
) {
    for connection in connections.read() {
        let client_id = connection.context();

        context.clients.insert(
            *client_id,
            Client {
                entity: Entity::PLACEHOLDER,
                client_type: ClientType::Unknown,
            },
        );

        server
            .send_message_to_target::<Channel1, SCHello>(
                SCHello,
                NetworkTarget::Only(vec![*client_id]),
            )
            .unwrap();
    }
}

pub(crate) fn connect_player(
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
