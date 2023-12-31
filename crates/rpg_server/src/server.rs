use crate::{account, game};

use bevy::{
    app::{App, FixedUpdate, Plugin, PreUpdate, Update},
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::IntoSystemConfigs,
        system::{Commands, Res, ResMut, Resource, SystemParam},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use rpg_network_protocol::{protocol::*, *};

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

        let transport = TransportConfig::UdpSocket(server_addr);

        #[cfg(feature = "net_debug")]
        let link_conditioner = LinkConditionerConfig {
            incoming_latency: Duration::from_millis(10),
            incoming_jitter: Duration::from_millis(20),
            incoming_loss: 0.05,
        };
        #[cfg(feature = "net_debug")]
        let io =
            Io::from_config(IoConfig::from_transport(transport)).with_conditioner(link_conditioner);

        #[cfg(not(feature = "net_debug"))]
        let io = Io::from_config(IoConfig::from_transport(transport));

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
            .add_systems(
                Update,
                (
                    account::receive_account_create,
                    account::receive_account_load,
                    account::receive_character_create,
                    account::receive_character_load,
                    account::receive_connect_player,
                    account::receive_connect_admin,
                ),
            );
    }
}

#[derive(SystemParam)]
pub(crate) struct NetworkParamsRO<'w> {
    pub(crate) server: Res<'w, Server>,
    pub(crate) context: Res<'w, NetworkContext>,
}

#[derive(SystemParam)]
pub(crate) struct NetworkParamsRW<'w> {
    pub(crate) server: ResMut<'w, Server>,
    pub(crate) context: ResMut<'w, NetworkContext>,
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
    Offline,
    Idle,
    Lobby,
    Game,
}

#[derive(Default, Resource)]
pub(crate) struct ServerState {
    pub(crate) mode: ServerMode,
}

#[derive(Default, Debug, PartialEq, Eq)]
pub enum AuthorizationStatus {
    #[default]
    Unauthenticated,
    Authenticated,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Client {
    pub(crate) entity: Entity,
    pub(crate) client_type: ClientType,
    pub(crate) auth_status: AuthorizationStatus,
}

impl Client {
    pub(crate) fn new(
        entity: Entity,
        client_type: ClientType,
        auth_status: AuthorizationStatus,
    ) -> Self {
        Self {
            entity,
            client_type,
            auth_status,
        }
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.auth_status == AuthorizationStatus::Authenticated
    }

    pub(crate) fn is_player(&self) -> bool {
        if let ClientType::Player(_) = self.client_type {
            true
        } else {
            false
        }
    }

    pub(crate) fn is_admin(&self) -> bool {
        if let ClientType::Admin(_) = self.client_type {
            true
        } else {
            false
        }
    }

    pub(crate) fn is_authenticated_player(&self) -> bool {
        self.is_player() && self.is_authenticated()
    }

    pub(crate) fn is_authenticated_admin(&self) -> bool {
        self.is_admin() && self.is_authenticated()
    }
}

impl Default for Client {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            client_type: ClientType::Unknown,
            auth_status: AuthorizationStatus::Unauthenticated,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct NetworkContext {
    pub clients: HashMap<ClientId, Client>,
}

pub(crate) fn handle_disconnections(
    mut disconnections: EventReader<DisconnectEvent>,
    mut net_params: NetworkParamsRW,
    mut commands: Commands,
) {
    for disconnection in disconnections.read() {
        let client_id = disconnection.context();
        info!("Removing {client_id} from global map");
        if let Some(client) = net_params.context.clients.remove(client_id) {
            if client.entity != Entity::PLACEHOLDER {
                commands.entity(client.entity).despawn_recursive();
            }
        }
    }
}

pub(crate) fn handle_connections(
    mut connections: EventReader<ConnectEvent>,
    mut net_params: NetworkParamsRW,
    mut server_state: ResMut<ServerState>,
) {
    if connections.len() > 0 && server_state.mode == ServerMode::Idle {
        info!("Setting server to Lobby mode");
        server_state.mode = ServerMode::Lobby;
    }

    for connection in connections.read() {
        let client_id = connection.context();

        net_params
            .context
            .clients
            .insert(*client_id, Client::default());

        net_params
            .server
            .send_message_to_target::<Channel1, SCHello>(
                SCHello,
                NetworkTarget::Only(vec![*client_id]),
            )
            .unwrap();
    }
}

/*
server.send_message_to_target::<Channel1, _>(message, NetworkTarget::All)
    .unwrap_or_else(|e| error!("Failed to send message: {:?}", e));
}
*/
