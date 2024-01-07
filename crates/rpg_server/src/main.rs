//! Run with
//! - `cargo run`
//! - `cargo run -- port 2320`
mod server;

use rpg_network_protocol::{protocol::*, *};

use bevy::log::LogPlugin;
use bevy::prelude::*;

use clap::{Parser, ValueEnum};

use crate::server::NetworkServerPlugin;
use lightyear::netcode::{ClientId, Key};
use lightyear::prelude::TransportConfig;

fn main() {
    let cli = Cli::parse();
    let mut app = App::new();
    setup(&mut app, cli);

    app.run();
}

/*
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Transports {
    Udp,
}
*/

#[derive(Parser, PartialEq, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = SERVER_PORT)]
    port: u16,
    //#[arg(short, long, value_enum, default_value_t = Transports::Udp)]
    //transport: Transports,
}

fn setup(app: &mut App, cli: Cli) {
    let server_plugin = NetworkServerPlugin {
        port: cli.port,
        transport: Transports::Udp,
    };

    app.add_plugins(LogPlugin::default())
        .add_plugins(server_plugin);
}
