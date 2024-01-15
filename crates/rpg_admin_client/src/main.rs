//! This is currently a testing playground.
//! Run with
//! - `cargo run --bin rpg_admin_client`
//! - `cargo run --bin rpg_admin_client -- --port 5000 --addr 127.0.0.1`

mod client;

use crate::client::{NetworkClientConfig, NetworkClientPlugin};

use rpg_network_protocol::*;

use bevy::{app::App, DefaultPlugins};

use clap::Parser;
use lightyear::netcode::ClientId;

use std::net::Ipv4Addr;

fn main() {
    let cli = Cli::parse();

    let client_plugin = NetworkClientPlugin {
        client_id: cli.client_id as ClientId,
        config: NetworkClientConfig {
            server_addr: cli.addr,
            server_port: cli.port,
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

    #[arg(long, default_value_t = Ipv4Addr::LOCALHOST)]
    addr: Ipv4Addr,

    #[arg(short, long, default_value_t = SERVER_PORT)]
    port: u16,
}
