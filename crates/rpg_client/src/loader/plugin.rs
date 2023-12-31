use crate::{
    assets::{AudioAssets, FontAssets, JsonAssets, TextureAssets},
    game::{self, plugin::GamePlugin, state_saver},
    menu::plugin::MenuPlugin,
    splash::plugin::SplashScreenPlugin,
    state::AppState,
};

//use console_plugin::plugin::ConsolePlugin;
use ui_util::{plugin::UiUtilPlugin, style::UiTheme};
use util::plugin::UtilityPlugin;

#[cfg(all(debug_assertions, feature = "bevy_diagnostic"))]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::{
    app::{App, AppExit, Plugin, PluginGroup, Startup, Update},
    audio::GlobalVolume,
    core_pipeline::core_2d::{Camera2d, Camera2dBundle},
    ecs::{
        component::Component,
        entity::Entity,
        event::EventWriter,
        query::With,
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter, OnExit},
        system::{Commands, NonSend, Query, ResMut},
    },
    log::LogPlugin,
    render::{
        camera::{Camera, ClearColor},
        prelude::*,
    },
    utils::default,
    window::{PrimaryWindow, Window, WindowPlugin, WindowResolution},
    winit::WinitWindows,
    DefaultPlugins,
};

use audio_manager::plugin::AudioManagerPlugin;
use bevy_tweening::TweeningPlugin;
use winit::window::Icon;

#[derive(Component)]
pub struct OutOfGameCamera;

pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        println!("Initializing loader plugin.");

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
                        filter: "wgpu=warn,naga=warn,client3d=info".into(),
                        ..default()
                    })
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            resolution: WindowResolution::new(1280., 720.)
                                .with_scale_factor_override(1.),
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
            .init_resource::<FontAssets>()
            .init_resource::<TextureAssets>()
            .init_resource::<AudioAssets>()
            .init_resource::<UiTheme>()
            .init_resource::<state_saver::SaveSlots>()
            // External plugins
            .add_plugins(TweeningPlugin)
            .add_plugins(UiUtilPlugin)
            //.add_plugins(ConsolePlugin)
            // Internal plugins
            .add_plugins(AudioManagerPlugin)
            .add_plugins(SplashScreenPlugin)
            .add_plugins(MenuPlugin)
            .add_plugins(GamePlugin)
            .add_systems(
                OnEnter(AppState::LoadGameAssets),
                (game::plugin::load_assets, state_saver::load_save_slots),
            )
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
    println!("transition to `AppState::Splash`");

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
    println!("transition to `AppState::LoadGameAssets`");
    state.set(AppState::LoadGameAssets);
}

// Set the window icon on supported platforms
fn set_window_icon(windows: NonSend<WinitWindows>, window_q: Query<Entity, With<PrimaryWindow>>) {
    let entity = window_q.single();
    let primary = windows.get_window(entity).unwrap();
    let path = format!(
        "{}/textures/app_icon.png",
        std::env::var("BEVY_ASSET_ROOT").unwrap()
    );

    let icon_buf = std::io::Cursor::new(path.as_str());
    let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) else {
        println!("Failed to set window icon");
        return;
    };

    let image = image.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon = Icon::from_rgba(rgba, width, height).unwrap();
    primary.set_window_icon(Some(icon));
}
