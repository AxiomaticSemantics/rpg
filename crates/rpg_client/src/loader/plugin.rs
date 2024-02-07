use crate::{
    assets::{AudioAssets, JsonAssets, TextureAssets},
    game::{self, plugin::GamePlugin, state_saver},
    net::{
        chat::Chat,
        lobby::Lobby,
        plugin::{NetworkClientConfig, NetworkClientPlugin},
    },
    splash::plugin::SplashScreenPlugin,
    state::AppState,
    ui::menu::plugin::MenuPlugin,
};

//use console_plugin::plugin::ConsolePlugin;
use ui_util::{plugin::UiUtilPlugin, style::UiTheme};
use util::{plugin::UtilityPlugin, random::Rng};

#[cfg(all(debug_assertions, feature = "bevy_diagnostic"))]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::{
    app::{App, AppExit, Plugin, PluginGroup, Startup, Update},
    audio::GlobalVolume,
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        event::EventWriter,
        query::With,
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter},
        system::{Commands, NonSend, Query, ResMut},
    },
    log::{info, warn, LogPlugin},
    render::{
        camera::{Camera, ClearColorConfig},
        color::Color,
        texture::ImagePlugin,
        view::Msaa,
    },
    utils::default,
    window::{PrimaryWindow, Window, WindowPlugin, WindowResolution},
    winit::WinitWindows,
    DefaultPlugins,
};

use rpg_network_protocol::SERVER_PORT;

use audio_manager::plugin::AudioManagerPlugin;
use bevy_tweening::TweeningPlugin;
use winit::window::Icon;

use clap::Parser;

use std::net::Ipv4Addr;

#[derive(Parser, PartialEq, Debug)]
struct Cli {
    #[arg(short, long, default_value_t = Ipv4Addr::UNSPECIFIED)]
    pub addr: Ipv4Addr,
    #[arg(short, long, default_value_t = SERVER_PORT)]
    pub port: u16,
}

#[derive(Component)]
pub struct OutOfGameCamera;

pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        println!("Initializing loader plugin.");

        let cli = Cli::parse();

        #[cfg(all(debug_assertions, feature = "diagnostics"))]
        {
            use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};

            app.edit_schedule(Main, |schedule| {
                println!("enabling ambiguity detection");
                schedule.set_build_settings(ScheduleBuildSettings {
                    ambiguity_detection: LogLevel::Warn,
                    ..default()
                });
            })
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_plugins(LogDiagnosticsPlugin::default());
        }

        app.init_state::<AppState>()
            .insert_resource(Msaa::Sample4)
            .add_plugins(
                DefaultPlugins
                    .set(LogPlugin {
                        filter: "wgpu=warn,naga=warn,rpg_client=debug".into(),
                        ..default()
                    })
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            resolution: WindowResolution::new(1280., 720.),
                            title: "Untitled RPG".into(),
                            resizable: false,
                            ..default()
                        }),
                        ..default()
                    })
                    .set(bevy::audio::AudioPlugin {
                        global_volume: GlobalVolume::new(0.25),
                        ..default()
                    })
                    .set(ImagePlugin::default_nearest()),
            )
            .add_systems(Startup, set_window_icon)
            .add_plugins(UtilityPlugin)
            .init_resource::<JsonAssets>()
            .init_resource::<TextureAssets>()
            .init_resource::<AudioAssets>()
            .init_resource::<UiTheme>()
            .init_resource::<Lobby>()
            .init_resource::<Chat>()
            .add_plugins(NetworkClientPlugin {
                config: generate_network_config(&cli),
            })
            .add_plugins(TweeningPlugin)
            .add_plugins(UiUtilPlugin)
            //.add_plugins(ConsolePlugin)
            .add_plugins(AudioManagerPlugin)
            .add_plugins(SplashScreenPlugin)
            .add_plugins(MenuPlugin)
            .add_plugins(GamePlugin)
            .add_systems(OnEnter(AppState::LoadGameAssets), game::plugin::load_assets)
            .add_systems(
                Update,
                transition_splash.run_if(in_state(AppState::LoadGameAssets)),
            )
            .add_systems(
                Update,
                transition_game_asset_load.run_if(in_state(AppState::LoadAssets)),
            )
            .add_systems(
                OnEnter(AppState::Shutdown),
                |mut exit: EventWriter<AppExit>| {
                    exit.send(AppExit);
                },
            );
    }
}

fn transition_splash(mut commands: Commands, mut state: ResMut<NextState<AppState>>) {
    info!("transition to `AppState::Splash`");

    commands.spawn((
        OutOfGameCamera,
        Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            ..default()
        },
    ));

    state.set(AppState::Splash);
}

fn transition_game_asset_load(mut state: ResMut<NextState<AppState>>) {
    info!("transition to `AppState::LoadGameAssets`");
    state.set(AppState::LoadGameAssets);
}

fn generate_network_config(cli: &Cli) -> NetworkClientConfig {
    let mut rng = Rng::new();
    let client_seed = rng.u64(0..u64::MAX);

    NetworkClientConfig {
        client_port: 0,
        client_seed,
        server_port: cli.port,
        server_addr: if cli.addr != Ipv4Addr::UNSPECIFIED {
            cli.addr
        } else {
            Ipv4Addr::new(192, 168, 0, 102)
        },
    }
}

// Set the window icon on supported platforms
fn set_window_icon(windows: NonSend<WinitWindows>, window_q: Query<Entity, With<PrimaryWindow>>) {
    let entity = window_q.single();
    let primary = windows.get_window(entity).unwrap();
    let path = format!(
        "{}/assets/icon/app_icon.png",
        std::env::var("BEVY_ASSET_ROOT").unwrap()
    );

    let icon_buf = std::io::Cursor::new(path.as_str());
    let image = match image::load(icon_buf, image::ImageFormat::Png) {
        Ok(icon) => icon,
        Err(err) => {
            warn!("Failed to load window icon: {err:?}");
            return;
        }
    };

    let image = image.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon = Icon::from_rgba(rgba, width, height).unwrap();
    primary.set_window_icon(Some(icon));
}
