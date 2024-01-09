//! Run with
//! - `cargo run`
//! - `cargo run -- port 42069`
mod account;
mod assets;
mod game;
mod server;
mod state;
mod world;

use crate::assets::{load_metadata, JsonAssets};
use crate::state::AppState;
use crate::{server::NetworkServerPlugin, world::ServerWorldPlugin};

use rpg_network_protocol::*;
use util::plugin::UtilityPlugin;

use bevy::{
    app::ScheduleRunnerPlugin, asset::AssetPlugin, ecs::schedule::common_conditions::in_state,
    log::LogPlugin, prelude::*,
};

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
    app.init_state::<AppState>()
        .add_plugins(
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
        .add_systems(Update, load_metadata.run_if(in_state(AppState::LoadAssets)))
        .add_plugins(NetworkServerPlugin { port: cli.port })
        .add_plugins(ServerWorldPlugin)
        .run();
}
