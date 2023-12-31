use bevy::ecs::schedule::States;

use clap::ValueEnum;

use std::fmt;

#[derive(ValueEnum, States, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    LoadAssets,
    LoadGameAssets,
    MenuLoad,
    Splash,
    SplashCleanup,
    Menu,
    GameSpawn,
    Game,
    GameOver,
    GameCleanup,
    Shutdown,
}

impl fmt::Display for AppState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LoadAssets => write!(f, "AppState::LoadAssets"),
            Self::LoadGameAssets => write!(f, "AppState::LoadGameAssets"),
            Self::MenuLoad => write!(f, "AppState:MenuLoad"),
            Self::Splash => write!(f, "AppState::Splash"),
            Self::SplashCleanup => write!(f, "AppState::SplashCleanup"),
            Self::Menu => write!(f, "AppState::Menu"),
            Self::GameSpawn => write!(f, "AppState::GameSpawn"),
            Self::Game => write!(f, "AppState::Game"),
            Self::GameOver => write!(f, "AppState::GameOver"),
            Self::GameCleanup => write!(f, "AppState::GameCleanup"),
            Self::Shutdown => write!(f, "AppState::Shutdown"),
        }
    }
}
