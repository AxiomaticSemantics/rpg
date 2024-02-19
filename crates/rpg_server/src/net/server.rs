use super::{account, chat, context::NetworkContext, game, lobby};
use crate::{game::plugin::GameState, state::AppState};

use bevy::{
    app::{App, FixedUpdate, Plugin, PreUpdate, Update},
    ecs::{
        event::{Event, EventReader, EventWriter},
        schedule::{common_conditions::*, Condition, IntoSystemConfigs, NextState},
        system::{Commands, Res, ResMut, SystemParam},
    },
    log::info,
};

use bevy_renet::renet::{ClientId, RenetServer, ServerEvent};
use bevy_renet::{transport::NetcodeServerPlugin, RenetServerPlugin};

use bevy_renet::renet::{
    transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    ConnectionConfig,
};

use rpg_network_protocol::{protocol::*, *};

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::SystemTime;

#[derive(Event)]
pub(crate) struct ClientMessageEvent {
    pub(crate) client_id: ClientId,
    pub(crate) message: ClientMessage,
}

pub(crate) struct NetworkServerPlugin {
    pub(crate) addr: Ipv4Addr,
    pub(crate) port: u16,
}

impl Plugin for NetworkServerPlugin {
    fn build(&self, app: &mut App) {
        let connection_config = ConnectionConfig {
            available_bytes_per_tick: 1024 * 1024,
            client_channels_config: ClientChannel::channels_config(),
            server_channels_config: ServerChannel::channels_config(),
        };

        let server = RenetServer::new(connection_config);

        let listen_addr = SocketAddr::new(self.addr.into(), self.port);
        info!("listening on {listen_addr:?}");

        let socket = UdpSocket::bind(listen_addr).unwrap();
        let current_time: std::time::Duration = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let server_config = ServerConfig {
            current_time,
            max_clients: 16,
            protocol_id: PROTOCOL_ID,
            public_addresses: vec![listen_addr],
            authentication: ServerAuthentication::Unsecure,
        };

        let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

        app.add_event::<ClientMessageEvent>()
            .add_plugins(NetcodeServerPlugin)
            .add_plugins(RenetServerPlugin)
            .insert_resource(server)
            .insert_resource(transport)
            .init_resource::<NetworkContext>()
            .add_systems(PreUpdate, (handle_connections, handle_messages).chain())
            .add_systems(
                FixedUpdate,
                (
                    game::receive_rotation,
                    game::receive_skill_use_direct,
                    game::receive_skill_use_targeted,
                    game::receive_item_drop,
                    game::receive_item_pickup,
                    game::receive_movement,
                    game::receive_movement_end,
                )
                    .chain()
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
                    (
                        game::receive_player_leave,
                        game::receive_player_join,
                        game::receive_player_loaded,
                        game::receive_player_revive,
                    )
                        .run_if(in_state(AppState::Simulation)),
                ),
            );
    }
}

#[derive(SystemParam)]
#[allow(dead_code)]
pub(crate) struct NetworkParamsRO<'w> {
    pub(crate) server: Res<'w, RenetServer>,
    pub(crate) context: Res<'w, NetworkContext>,
}

#[derive(SystemParam)]
pub(crate) struct NetworkParamsRW<'w> {
    pub(crate) server: ResMut<'w, RenetServer>,
    pub(crate) context: ResMut<'w, NetworkContext>,
}

fn handle_connections(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    mut connect_reader: EventReader<ServerEvent>,
    mut net_params: NetworkParamsRW,
) {
    for event in connect_reader.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("client joined: {client_id}");

                net_params.context.add_client(*client_id);

                let message = bincode::serialize(&ServerMessage::SCHello(SCHello)).unwrap();
                net_params
                    .server
                    .send_message(*client_id, ServerChannel::Message, message);
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("client disconnected: {reason:?}");
                game_state.players.retain(|p| p.client_id != *client_id);

                net_params.context.remove_client(&mut commands, *client_id);

                if game_state.players.is_empty() {
                    state.set(AppState::Lobby);
                }
            }
        }
    }
}

fn handle_messages(
    mut net_params: NetworkParamsRW,
    mut message_writer: EventWriter<ClientMessageEvent>,
) {
    for client_id in net_params.server.clients_id() {
        while let Some(message) = net_params
            .server
            .receive_message(client_id, ClientChannel::Message)
        {
            let message: ClientMessage = bincode::deserialize(&message).unwrap();

            message_writer.send(ClientMessageEvent { client_id, message });
        }
    }
}
