//! Run with
//! - `cargo run`
//! - `cargo run -- --port 4269 --addr 127.0.0.1`

mod assets;
mod server_state;
mod state;

mod net;

mod account;
mod chat;
mod game;
mod lobby;

mod world;

use crate::{
    assets::{load_metadata, JsonAssets},
    chat::ChatManager,
    game::plugin::GamePlugin,
    lobby::LobbyManager,
    net::server::NetworkServerPlugin,
    server_state::ServerMetadataResource,
    state::AppState,
};

use rpg_network_protocol::*;
use util::plugin::UtilityPlugin;

use bevy::{
    app::{App, PluginGroup, ScheduleRunnerPlugin, Startup, Update},
    asset::AssetPlugin,
    core::TaskPoolPlugin,
    ecs::schedule::{common_conditions::in_state, IntoSystemConfigs},
    log::LogPlugin,
    time::{Fixed, Time},
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

use std::io::Error;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time;

use signal_hook::consts::signal::*;
use signal_hook::consts::TERM_SIGNALS;
use signal_hook::flag;
use signal_hook::iterator::Signals;

fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let term_now = Arc::new(AtomicBool::new(false));
    for sig in TERM_SIGNALS {
        // When terminated by a second term signal, exit with exit code 1.
        // This will do nothing the first time (because term_now is false).
        flag::register_conditional_shutdown(*sig, 1, Arc::clone(&term_now))?;
        flag::register(*sig, Arc::clone(&term_now))?;
    }

    let mut signals = Signals::new(TERM_SIGNALS)?;

    let t = thread::spawn(move || {
        App::new()
            .init_state::<AppState>()
            .add_plugins(
                MinimalPlugins
                    .set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
                        1.0 / 60.0,
                    )))
                    .set(TaskPoolPlugin::default()),
            )
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 60.))
            .add_plugins(LogPlugin::default())
            .add_plugins(AssetPlugin::default())
            .add_plugins(UtilityPlugin)
            .init_resource::<JsonAssets>()
            .init_resource::<ServerMetadataResource>()
            .init_resource::<ChatManager>()
            .init_resource::<LobbyManager>()
            .add_systems(Startup, chat::setup)
            .add_systems(Update, load_metadata.run_if(in_state(AppState::Loading)))
            .add_plugins(NetworkServerPlugin { port: cli.port })
            .add_plugins(GamePlugin)
            .run();
    });

    'outer: loop {
        let mut caught = false;
        for signal in signals.pending() {
            match signal {
                SIGINT => {
                    caught = true;
                    eprintln!("\nCaught: SIGINT; joining app thread");
                    break 'outer;
                }
                SIGTERM => {
                    caught = true;
                    eprintln!("\nCaught: SIGTERM; joining app thread");
                    break 'outer;
                }
                term_sig => {
                    caught = true;
                    eprintln!("\nCaught: {:?}; joining app thread", term_sig);
                    break 'outer;
                }
            }
        }
        thread::sleep(time::Duration::from_millis(1));
    }

    eprintln!("\nEnter Ctrl+C again to exit immediately.");

    t.join().unwrap();

    Ok(())
}
