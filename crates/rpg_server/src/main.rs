//! Run with
//! - `cargo run`
//! - `cargo run -- port 42069`
mod game;
mod server;
mod world;

use rpg_network_protocol::*;

use bevy::log::LogPlugin;
use bevy::prelude::*;

use clap::Parser;

use crate::{server::NetworkServerPlugin, world::ServerWorldPlugin};

fn main() {
    let cli = Cli::parse();

    let mut app = App::new();
    app.add_plugins(LogPlugin::default())
        .add_plugins(NetworkServerPlugin { port: cli.port })
        .add_plugins(ServerWorldPlugin)
        .run();
}

#[derive(Parser, PartialEq, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = SERVER_PORT)]
    port: u16,
}
