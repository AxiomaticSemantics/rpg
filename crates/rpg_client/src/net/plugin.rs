use bevy::app::{App, Plugin};

use lightyear::{
    client::{
        config::ClientConfig,
        interpolation::plugin::{InterpolationConfig, InterpolationDelay},
        plugin::{ClientPlugin, PluginConfig},
        prediction::plugin::PredictionConfig,
        resource::Authentication,
        sync::SyncConfig,
    },
    netcode::ClientId,
    shared::ping::manager::PingConfig,
    transport::io::*,
};
use rpg_network_protocol::{protocol::protocol, KEY, PROTOCOL_ID};

use std::net::{Ipv4Addr, SocketAddr};

#[derive(Debug, Clone)]
pub struct NetworkClientConfig {
    pub client_port: u16,
    pub server_addr: Ipv4Addr,
    pub server_port: u16,
}

pub struct NetworkClientPlugin {
    pub config: NetworkClientConfig,
    // pub client_id: ClientId,
}

impl Plugin for NetworkClientPlugin {
    fn build(&self, app: &mut App) {
        let server_addr = SocketAddr::new(self.config.server_addr.into(), self.config.server_port);
        let auth = Authentication::Manual {
            server_addr,
            client_id: 0,
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
            // we are sending updates every frame (60fps), let's add a delay of 6 network-ticks
            interpolation: InterpolationConfig::default()
                .with_delay(InterpolationDelay::default().with_send_interval_ratio(2.0)),
        };
        let plugin_config = PluginConfig::new(config, io, protocol(), auth);

        app.add_plugins(ClientPlugin::new(plugin_config));
    }
}
