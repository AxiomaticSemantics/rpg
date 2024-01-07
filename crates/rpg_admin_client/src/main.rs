//! Run with
//! - `cargo run --bin rpg_admin_client`
mod client;

use rpg_network_protocol::*;

use std::net::Ipv4Addr;

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::DefaultPlugins;
use clap::{Parser, ValueEnum};

use crate::client::{NetworkClientPlugin, NetworkClientPluginConfig};
use lightyear::netcode::ClientId;
use lightyear::prelude::TransportConfig;

fn main() {
    let cli = Cli::parse();
    let mut app = App::new();
    setup(&mut app, cli);

    app.run();
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Transports {
    Udp,
}

#[derive(Parser, PartialEq, Debug)]
pub(crate) struct Cli {
    #[arg(short, long, default_value_t = 0)]
    client_id: u16,

    #[arg(long, default_value_t = CLIENT_PORT)]
    client_port: u16,

    #[arg(long, default_value_t = Ipv4Addr::LOCALHOST)]
    server_addr: Ipv4Addr,

    #[arg(short, long, default_value_t = SERVER_PORT)]
    server_port: u16,

    #[arg(short, long, value_enum, default_value_t = Transports::Udp)]
    transport: Transports,
}

fn setup(app: &mut App, cli: Cli) {
    let client_plugin = NetworkClientPlugin {
        client_id: cli.client_id as ClientId,
        config: NetworkClientPluginConfig {
            client_port: cli.client_port,
            server_addr: cli.server_addr,
            server_port: cli.server_port,
            transport: cli.transport,
        },
    };
    app.add_plugins(DefaultPlugins).add_plugins(client_plugin);
}
