//! Run with
//! - `cargo run`
//! - `cargo run -- port 42069`
mod assets;
mod net;
mod state;

mod world;

use crate::{
    assets::{load_metadata, JsonAssets},
    net::server::NetworkServerPlugin,
    state::AppState,
    world::ServerWorldPlugin,
};

use rpg_network_protocol::*;
use util::plugin::UtilityPlugin;

use bevy::{
    app::{App, PluginGroup, ScheduleRunnerPlugin, Update},
    asset::AssetPlugin,
    core::TaskPoolPlugin,
    ecs::schedule::{common_conditions::in_state, IntoSystemConfigs},
    log::LogPlugin,
    MinimalPlugins,
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
