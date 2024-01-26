use super::{account, chat, game, lobby};
use crate::state::AppState;

use bevy::{
    app::{App, FixedUpdate, Plugin, Startup, Update},
    ecs::{
        event::EventReader,
        schedule::{common_conditions::*, Condition, IntoSystemConfigs},
        system::{Commands, Res, ResMut, Resource},
        world::{FromWorld, World},
    },
    prelude::{Deref, DerefMut},
    time::{Fixed, Time, Timer, TimerMode},
};

use lightyear::{
    client::{
        config::ClientConfig,
        events::MessageEvent,
        interpolation::plugin::{InterpolationConfig, InterpolationDelay},
        plugin::{ClientPlugin, PluginConfig},
        prediction::plugin::PredictionConfig,
        resource::Authentication,
        sync::SyncConfig,
    },
    netcode::ClientId,
    packet::message::Message,
    shared::{ping::manager::PingConfig, sets::FixedUpdateSet},
    transport::io::*,
};
use rpg_network_protocol::{
    protocol::{protocol, Client, *},
    KEY, PROTOCOL_ID,
};

use std::net::{Ipv4Addr, SocketAddr};

#[derive(Debug, Clone)]
pub struct NetworkClientConfig {
    pub client_seed: u64,
    pub client_port: u16,
    pub server_addr: Ipv4Addr,
    pub server_port: u16,
}

pub struct NetworkClientPlugin {
    pub config: NetworkClientConfig,
}

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        let server_addr = SocketAddr::new(self.config.server_addr.into(), self.config.server_port);
        let auth = Authentication::Manual {
            server_addr,
            client_id: self.config.client_seed,
            private_key: KEY,
            protocol_id: PROTOCOL_ID,
        };
        let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), self.config.client_port);
        let transport = TransportConfig::UdpSocket(client_addr);
        let io = Io::from_config(IoConfig::from_transport(transport));

        let config = ClientConfig {
            shared: rpg_network_protocol::shared_config(),
            netcode: Default::default(),
            ping: PingConfig::default(),
            sync: SyncConfig::default(),
            prediction: PredictionConfig::default(),
            interpolation: InterpolationConfig::default()
                .with_delay(InterpolationDelay::default().with_send_interval_ratio(2.0)),
        };
        let plugin_config = PluginConfig::new(config, io, protocol(), auth);

        app.add_plugins(ClientPlugin::new(plugin_config))
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.))
            .init_resource::<ConnectionTimer>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    reconnect,
                    receive_server_hello,
                    (
                        account::receive_account_create_success,
                        account::receive_account_create_error,
                        account::receive_account_login_success,
                        account::receive_account_login_error,
                        account::receive_character_create_success,
                        account::receive_character_create_error,
                        account::receive_game_create_success,
                        account::receive_game_create_error,
                        account::receive_game_join_success,
                        account::receive_game_join_error,
                    ),
                    (
                        lobby::receive_join_success,
                        lobby::receive_join_error,
                        lobby::receive_create_success,
                        lobby::receive_create_error,
                        lobby::receive_lobby_message,
                    ),
                    (
                        chat::receive_join_success,
                        chat::receive_join_error,
                        chat::receive_channel_join_success,
                        chat::receive_channel_join_error,
                        chat::receive_chat_message,
                    ),
                    (
                        game::receive_player_spawn,
                        game::receive_player_join_success,
                        game::receive_player_join_error,
                    )
                        .run_if(in_state(AppState::Game).or_else(in_state(AppState::GameSpawn))),
                ),
            )
            .add_systems(
                FixedUpdate,
                (
                    game::receive_stat_updates,
                    game::receive_stat_update,
                    game::receive_spawn_villain,
                    game::receive_despawn_villain,
                    game::receive_spawn_item,
                    game::receive_spawn_items,
                    game::receive_player_rotation,
                    game::receive_player_move,
                    game::receive_player_move_end,
                )
                    .after(FixedUpdateSet::Main),
            );
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ConnectionTimer(pub Timer);

impl FromWorld for ConnectionTimer {
    fn from_world(_world: &mut World) -> Self {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}

fn reconnect(
    time: Res<Time>,
    mut net_client: ResMut<Client>,
    mut connection_timer: ResMut<ConnectionTimer>,
) {
    let dt = time.delta();
    connection_timer.tick(dt);

    if net_client.is_connected() {
        if !connection_timer.paused() {
            connection_timer.pause();
        }
    } else {
        if connection_timer.just_finished() {
            net_client.connect();
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(account::RpgAccount::default());
}

fn receive_server_hello(
    net_client: Res<Client>,
    mut hello_events: EventReader<MessageEvent<SCHello>>,
) {
}
