use super::{account, chat, game, lobby};
use crate::state::AppState;

use bevy::{
    app::{App, FixedPreUpdate, FixedUpdate, Plugin, Startup, Update},
    ecs::{
        event::{EventReader, EventWriter},
        schedule::{common_conditions::*, Condition, IntoSystemConfigs},
        system::{Res, ResMut, Resource},
        world::{FromWorld, World},
    },
    prelude::{Deref, DerefMut},
    time::{Fixed, Time, Timer, TimerMode},
};

use rpg_network_protocol::{protocol::*, KEY, PROTOCOL_ID};

use bevy_renet::{
    client_connected,
    renet::{
        transport::{ClientAuthentication, NetcodeClientTransport},
        ClientId, ConnectionConfig, RenetClient,
    },
    transport::NetcodeClientPlugin,
    RenetClientPlugin,
};

use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::SystemTime;

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

        app.add_plugins(NetcodeClientPlugin)
            .add_plugins(RenetClientPlugin);

        let connection_config = ConnectionConfig {
            available_bytes_per_tick: 1024 * 1024,
            client_channels_config: ClientChannel::channels_config(),
            server_channels_config: ServerChannel::channels_config(),
        };

        let client = RenetClient::new(connection_config);

        let server_addr = "127.0.0.1:4269".parse().unwrap();
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let current_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let client_id = current_time.as_millis() as u64;
        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            server_addr,
            user_data: None,
        };

        let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

        app.add_event::<ServerMessage>()
            .insert_resource(client)
            .insert_resource(transport)
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.))
            .init_resource::<ConnectionTimer>()
            .add_systems(Startup, connect)
            .add_systems(
                Update,
                (
                    connect,
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
                    )
                        .run_if(in_state(AppState::Menu)),
                    (
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
                    )
                        .run_if(in_state(AppState::Menu).or_else(in_state(AppState::Game))),
                    (
                        game::receive_player_spawn,
                        game::receive_player_join_success,
                        game::receive_player_join_error,
                        game::receive_zone_load,
                        game::receive_zone_unload,
                    )
                        .run_if(in_state(AppState::Game).or_else(
                            in_state(AppState::GameJoin).or_else(in_state(AppState::GameSpawn)),
                        )),
                ),
            )
            .add_systems(
                FixedPreUpdate,
                (
                    sync_client,
                    (
                        game::receive_despawn_corpse,
                        game::receive_despawn_item,
                        game::receive_despawn_skill,
                        game::receive_spawn_item,
                        game::receive_spawn_items,
                        game::receive_spawn_villain,
                        game::receive_spawn_hero,
                        game::receive_spawn_skill,
                        game::receive_player_revive,
                        game::receive_hero_revive,
                        game::receive_item_pickup,
                        game::receive_item_drop,
                        game::receive_item_store,
                    ),
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    game::receive_damage,
                    game::receive_stat_updates,
                    game::receive_stat_update,
                    (
                        game::receive_player_rotation,
                        game::receive_player_move,
                        game::receive_player_move_end.after(game::receive_player_move),
                        game::receive_unit_rotation,
                        game::receive_unit_move,
                        game::receive_unit_move_end.after(game::receive_unit_move),
                    ),
                    (
                        game::receive_unit_attack,
                        game::receive_unit_anim,
                        game::receive_combat_result,
                        game::receive_hero_death,
                        game::receive_villain_death,
                    )
                        .chain()
                        .after(game::receive_damage),
                ),
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

fn sync_client(
    mut net_client: ResMut<RenetClient>,
    mut message_writer: EventWriter<ServerMessage>,
) {
    while let Some(message) = net_client.receive_message(ServerChannel::Message) {
        let server_message: ServerMessage = bincode::deserialize(&message).unwrap();
        message_writer.send(server_message);
    }
}

fn connect(
    time: Res<Time>,
    mut net_client: ResMut<RenetClient>,
    mut connection_timer: ResMut<ConnectionTimer>,
) {
    if net_client.is_connected() {
        if !connection_timer.paused() {
            connection_timer.pause();
            connection_timer.reset();
        }
        return;
    }

    let dt = time.delta();
    if connection_timer.paused() {
        connection_timer.reset();
        connection_timer.unpause();
        return;
    }

    connection_timer.tick(dt);

    if connection_timer.just_finished() {
        connection_timer.reset();
    }
}

fn receive_server_hello(
    _net_client: Res<RenetClient>,
    mut hello_events: EventReader<ServerMessage>,
) {
    // TODO use this to disallow login/creation?
}
