use super::{account, chat, client::Client, context::NetworkContext, game, lobby};
use crate::{game::plugin::GameState, state::AppState};

use bevy::{
    app::{App, FixedUpdate, Plugin, PreUpdate, Update},
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::{common_conditions::*, Condition, IntoSystemConfigs, NextState},
        system::{Commands, Res, ResMut, Resource, SystemParam},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use rpg_account::account::AccountId;
use rpg_network_protocol::{protocol::*, *};

use rpg_util::unit::collide_units;

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
                    collide_units,
                    game::receive_rotation,
                    game::receive_skill_use_direct,
                    game::receive_skill_use_targeted,
                    game::receive_item_drop,
                    game::receive_item_pickup,
                    game::receive_movement,
                    game::receive_movement_end,
                )
                    .chain()
                    .run_if(in_state(AppState::Simulation))
                    .after(FixedUpdateSet::Main),
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
                    )
                        .run_if(in_state(AppState::Simulation)),
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

fn handle_disconnections(
    mut state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    mut disconnect_reader: EventReader<DisconnectEvent>,
    mut net_params: NetworkParamsRW,
    mut commands: Commands,
) {
    for event in disconnect_reader.read() {
        let client_id = *event.context();
        game_state.players.retain(|p| p.client_id != client_id);

        net_params.context.remove_client(&mut commands, client_id);

        if game_state.players.is_empty() {
            state.set(AppState::Lobby);
            return;
        }
    }
}

fn handle_connections(
    mut connect_reader: EventReader<ConnectEvent>,
    mut net_params: NetworkParamsRW,
) {
    for event in connect_reader.read() {
        let client_id = *event.context();

        net_params.context.add_client(client_id);

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
