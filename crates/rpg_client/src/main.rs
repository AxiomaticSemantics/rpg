//! Run with
//! - `cargo run`
//! - `cargo run -- --addr 192.168.0.1 --port 42069`

#![cfg_attr(
    all(
        feature = "disable_windows_console",
        target_os = "windows",
        not(debug_assertions)
    ),
    windows_subsystem = "windows"
)]

mod assets;
mod game;
mod loader;
mod menu;
mod net;
mod random;
mod splash;
mod state;

use bevy::app::App;

use loader::plugin::LoaderPlugin;

fn main() {
    App::new().add_plugins(LoaderPlugin).run();
}
