//! Run with
//! - `cargo run --bin rpg_admin_client`
mod client;

use crate::client::{NetworkClientConfig, NetworkClientPlugin};

use rpg_network_protocol::*;

use bevy::app::{App, DefaultPlugins};

use clap::Parser;
use lightyear::netcode::ClientId;

use std::net::Ipv4Addr;

fn main() {
    let cli = Cli::parse();

    let client_plugin = NetworkClientPlugin {
        client_id: cli.client_id as ClientId,
        config: NetworkClientConfig {
            client_port: cli.client_port,
            server_addr: cli.server_addr,
            server_port: cli.server_port,
        },
    };
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(client_plugin)
        .run();
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
}
