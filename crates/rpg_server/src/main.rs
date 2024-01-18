//! Run with
//! - `cargo run`
//! - `cargo run -- --port 42069`

mod assets;
mod server_state;
mod state;

mod net;

mod chat;
mod lobby;

mod world;

use crate::{
    assets::{load_metadata, JsonAssets},
    chat::ChatManager,
    lobby::LobbyManager,
    net::server::NetworkServerPlugin,
    server_state::ServerMetadataResource,
    state::AppState,
    world::ServerWorldPlugin,
};

use rpg_network_protocol::*;
use util::plugin::UtilityPlugin;

use bevy::{
    app::{App, PluginGroup, ScheduleRunnerPlugin, Startup, Update},
    asset::AssetPlugin,
    core::TaskPoolPlugin,
    ecs::schedule::{common_conditions::in_state, IntoSystemConfigs},
    log::LogPlugin,
    MinimalPlugins,
};

use clap::Parser;

use std::{net::Ipv4Addr, time::Duration};

#[derive(Parser, PartialEq, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = SERVER_PORT)]
    port: u16,
    #[arg(short, long, default_value_t = Ipv4Addr::UNSPECIFIED)]
    addr: Ipv4Addr,
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
        .init_resource::<ServerMetadataResource>()
        .init_resource::<ChatManager>()
        .init_resource::<LobbyManager>()
        .add_systems(Startup, chat::setup)
        .add_systems(Update, load_metadata.run_if(in_state(AppState::LoadAssets)))
        .add_plugins(NetworkServerPlugin { port: cli.port })
        .add_plugins(ServerWorldPlugin)
        .run();
}
