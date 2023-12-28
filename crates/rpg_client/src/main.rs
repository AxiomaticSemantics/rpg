#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod assets;
mod game;
mod loader;
mod menu;
mod random;
mod splash;
mod state;

use bevy::app::App;

use loader::plugin::LoaderPlugin;

fn main() {
    App::new().add_plugins(LoaderPlugin).run();
}
