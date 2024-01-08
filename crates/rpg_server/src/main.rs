//! Run with
//! - `cargo run`
//! - `cargo run -- port 42069`
mod assets;
mod game;
mod server;
mod world;

use crate::assets::JsonAssets;
use crate::{server::NetworkServerPlugin, world::ServerWorldPlugin};

use rpg_network_protocol::*;
use util::plugin::UtilityPlugin;

use bevy::app::ScheduleRunnerPlugin;
use bevy::asset::AssetPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;

use clap::Parser;
use std::time::Duration;

#[derive(Parser, PartialEq, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = SERVER_PORT)]
    port: u16,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new();
    app.add_plugins(
        MinimalPlugins
            .set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                1.0 / 60.0,
            )))
            .set(TaskPoolPlugin::default()),
    )
    .add_plugins(LogPlugin::default())
    .add_plugins(AssetPlugin::default())
    .add_plugins(UtilityPlugin)
    .init_resource::<JsonAssets>()
    .add_plugins(NetworkServerPlugin { port: cli.port })
    .add_plugins(ServerWorldPlugin)
    .run();
}
