//! rpg_client
//!
//! Most loading happens in `LoaderPlugin`
//!
//! Run with
//! - `cargo run -p rpg_client`
//! - `cargo run -p rpg_client -- --addr 192.168.0.1 --port 42069`

#![cfg_attr(
    all(feature = "disable_windows_console", target_os = "windows",),
    windows_subsystem = "windows"
)]

mod assets;
mod game;
mod loader;
mod net;
mod splash;
mod state;
mod ui;

use bevy::app::App;

use loader::plugin::LoaderPlugin;

fn main() {
    App::new().add_plugins(LoaderPlugin).run();
}
