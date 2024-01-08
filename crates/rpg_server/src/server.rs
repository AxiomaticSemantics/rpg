use crate::game;

use bevy::{
    app::{App, FixedUpdate, Plugin, PreUpdate, Startup, Update},
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::IntoSystemConfigs,
        system::{Commands, Res, ResMut, Resource},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use rpg_network_protocol::{protocol::*, *};
use util::assets::json::JsonSource;

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};

pub(crate) struct NetworkServerPlugin {
    pub(crate) port: u16,
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
            .init_resource::<NetworkContext>()
            .init_resource::<ServerState>()
            .add_systems(
                FixedUpdate,
                (game::rotation_request, game::movement_request)
                    .chain()
                    .in_set(FixedUpdateSet::Main),
            )
            .add_systems(PreUpdate, (handle_connections, handle_disconnections))
            .add_systems(Update, game::receive_connect_player);
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum ClientType {
    #[default]
    Unknown,
    Player(ClientId),
    Admin(ClientId),
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub(crate) enum ServerMode {
    #[default]
    Idle,
    Lobby,
    Game,
}

#[derive(Default, Resource)]
pub(crate) struct ServerState {
    pub(crate) mode: ServerMode,
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
    mut server_state: ResMut<ServerState>,
) {
    if connections.len() > 0 && server_state.mode == ServerMode::Idle {
        server_state.mode = ServerMode::Lobby;
    }

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

/*
server.send_message_to_target::<Channel1, _>(message, NetworkTarget::All)
    .unwrap_or_else(|e| {
        error!("Failed to send message: {:?}", e);
    });
}
*/
