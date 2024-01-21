use super::{account, chat, game, lobby};
use crate::{game::plugin::GameState, state::AppState};

use bevy::{
    app::{App, FixedUpdate, Plugin, PreUpdate, Update},
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::{common_conditions::*, Condition, IntoSystemConfigs},
        system::{Commands, Res, ResMut, Resource, SystemParam},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use rpg_account::account::AccountId;
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
            .add_systems(PreUpdate, (handle_connections, handle_disconnections))
            .add_systems(
                FixedUpdate,
                (
                    game::receive_rotation,
                    game::receive_skill_use_direct,
                    game::receive_skill_use_targeted,
                    game::receive_movement,
                )
                    .chain()
                    //.in_set(FixedUpdateSet::Main)
                    .run_if(in_state(AppState::Simulation)),
            )
            .add_systems(
                Update,
                (
                    (
                        (
                            account::receive_account_create,
                            account::receive_account_login,
                            account::receive_admin_login,
                            account::receive_character_create,
                            account::receive_game_create,
                        ),
                        (
                            lobby::receive_lobby_create,
                            lobby::receive_lobby_join,
                            lobby::receive_lobby_leave,
                            lobby::receive_lobby_message,
                        ),
                    )
                        .run_if(in_state(AppState::Lobby)),
                    (
                        account::receive_game_join,
                        chat::receive_chat_channel_message,
                        chat::receive_chat_join,
                        chat::receive_chat_leave,
                    )
                        .run_if(
                            in_state(AppState::Lobby)
                                .or_else(in_state(AppState::SpawnSimulation))
                                .or_else(in_state(AppState::Simulation)),
                        ),
                    (game::receive_player_ready).run_if(in_state(AppState::Simulation)),
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
    Player,
    Admin,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Client {
    pub(crate) entity: Entity,
    pub(crate) client_type: ClientType,
    pub(crate) account_id: Option<AccountId>,
}

impl Client {
    pub(crate) fn new(entity: Entity, client_type: ClientType) -> Self {
        Self {
            entity,
            account_id: None,
            client_type,
        }
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.account_id.is_some() && (self.is_player() || self.is_admin())
    }

    pub(crate) fn is_player(&self) -> bool {
        ClientType::Player == self.client_type
    }

    pub(crate) fn is_admin(&self) -> bool {
        ClientType::Admin == self.client_type
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
            account_id: None,
        }
    }
}

#[derive(Resource, Default)]
pub(crate) struct NetworkContext {
    pub clients: HashMap<ClientId, Client>,
}

impl NetworkContext {
    pub fn get_client_from_client_id(&self, id: ClientId) -> Option<&Client> {
        self.clients.get(&id)
    }

    pub fn get_client_from_account_id(&self, id: AccountId) -> Option<&Client> {
        self.clients.values().find(|a| {
            if let Some(aid) = a.account_id {
                aid == id
            } else {
                false
            }
        })
    }

    pub fn is_client_authenticated(&self, id: ClientId) -> bool {
        if let Some(client) = self.clients.get(&id) {
            client.is_authenticated()
        } else {
            false
        }
    }
}

pub(crate) fn handle_disconnections(
    mut game_state: ResMut<GameState>,
    mut disconnect_reader: EventReader<DisconnectEvent>,
    mut net_params: NetworkParamsRW,
    mut commands: Commands,
) {
    for event in disconnect_reader.read() {
        let client_id = *event.context();
        game_state.players.retain(|p| p.client_id != client_id);

        if let Some(client) = net_params.context.clients.remove(&client_id) {
            info!("Removing {client_id} from global map");

            if client.entity != Entity::PLACEHOLDER {
                info!("despawning client entity");
                commands.entity(client.entity).despawn_recursive();
            }
        }
    }
}

pub(crate) fn handle_connections(
    mut connect_reader: EventReader<ConnectEvent>,
    mut net_params: NetworkParamsRW,
) {
    for event in connect_reader.read() {
        let client_id = *event.context();

        net_params
            .context
            .clients
            .insert(client_id, Client::default());

        net_params
            .server
            .send_message_to_target::<Channel1, SCHello>(
                SCHello(client_id),
                NetworkTarget::Only(vec![client_id]),
            )
            .unwrap();

        info!("sending hello to {client_id}");
    }
}
