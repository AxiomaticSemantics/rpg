use crate::{
    assets::TextureAssets,
    loader::plugin::OutOfGameCamera,
    state::AppState,
    ui::{
        chat::{self},
        lobby::{self},
        menu::{self, main::MainRoot},
    },
};

/*
use console_plugin::{
    console::{Console, HistoryIndex},
    plugin::{ConsoleHistoryItem, ConsoleInput},
};*/

use ui_util::style::{UiRoot, UiTheme};
use util::cleanup::{self, CleanupStrategy};

use bevy::{
    app::{App, Plugin, Update},
    core_pipeline::core_2d::Camera2d,
    ecs::{
        query::With,
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter},
        system::{Commands, ParamSet, Query, Res, ResMut},
    },
    hierarchy::BuildChildren,
    render::{camera::Camera, color::Color},
    text::TextStyle,
    ui::{
        node_bundles::{ButtonBundle, NodeBundle},
        AlignItems, AlignSelf, Display, JustifyContent, Style, UiRect,
    },
    utils::default,
};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        println!("Initializing menu plugin.");
        app.init_resource::<menu::account::SelectedCharacter>()
            .add_systems(OnEnter(AppState::MenuLoad), spawn)
            .add_systems(OnEnter(AppState::Menu), display_menu)
            .add_systems(
                Update,
                (
                    (
                        menu::main::exit_button,
                        menu::main::account_create_button,
                        menu::main::account_login_button,
                        menu::main::settings_button,
                        menu::main::credits_button,
                    ),
                    (
                        menu::account::cancel_create_button,
                        menu::account::cancel_login_button,
                        menu::account::create_button,
                        menu::account::login_button,
                        menu::account::lobby_create_button,
                        menu::account::lobby_join_button,
                        menu::account::list_create_character_button,
                        menu::account::list_create_game_button,
                        menu::account::list_cancel_button,
                        menu::account::list_select_slot,
                        menu::account::update_character_list,
                    ),
                    (
                        menu::create::cancel_button,
                        menu::create::create_class,
                        menu::create::set_game_mode,
                    ),
                    (
                        menu::settings::cancel_button,
                        menu::settings::controls_button,
                        menu::settings::audio_button,
                    ),
                    menu::credits::cancel_button,
                    (
                        lobby::game_create_button,
                        lobby::lobby_send_message,
                        lobby::leave_button,
                        lobby::update_lobby_messages,
                        lobby::update_players_container,
                    ),
                )
                    .run_if(in_state(AppState::Menu)),
            )
            .add_systems(OnEnter(AppState::Shutdown), cleanup::cleanup::<UiRoot>);
    }
}

fn display_menu(
    mut menu_set: ParamSet<(
        Query<&mut Style, With<UiRoot>>,
        Query<&mut Style, With<MainRoot>>,
    )>,
    mut camera_q: Query<&mut Camera, (With<Camera2d>, With<OutOfGameCamera>)>,
) {
    let mut camera = camera_q.single_mut();
    if !camera.is_active {
        camera.is_active = true;
    }

    menu_set.p0().single_mut().display = Display::Flex;
    menu_set.p1().single_mut().display = Display::Flex;
}

fn spawn(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    ui_theme: Res<UiTheme>,
    textures: Res<TextureAssets>,
    //mut console: ResMut<Console>,
    /*console_set: ParamSet<(
        Query<(&mut EditText, &mut Text, &mut HistoryIndex), With<ConsoleInput>>,
        Query<(&mut Text, &mut HistoryIndex), With<ConsoleHistoryItem>>,
    )>,*/
) {
    println!("spawning main menu");

    /*
    let message = "spawning main menu".to_string();

    if console.ui_root.is_some() {
        console.update_history(message.clone(), false);
    }*/

    let text_style = TextStyle {
        font: ui_theme.font.clone(),
        font_size: ui_theme.font_size_regular,
        color: ui_theme.button_theme.normal_text_color,
    };

    let text_node_style = Style {
        margin: UiRect::all(ui_theme.margin),
        ..default()
    };

    let button_bundle = ButtonBundle {
        style: Style {
            margin: UiRect::all(ui_theme.margin),
            justify_content: JustifyContent::Center,
            align_self: AlignSelf::Center,
            align_items: AlignItems::Center,
            ..default()
        },

        background_color: Color::NONE.into(),
        ..default()
    };

    let mut ui_container_style = ui_theme.container_absolute_max.clone();
    ui_container_style.display = Display::None;

    let mut frame_hidden = ui_theme.frame_col_style.clone();
    frame_hidden.display = Display::None;

    // root node
    commands
        .spawn((
            CleanupStrategy::DespawnRecursive,
            NodeBundle {
                style: ui_container_style,
                ..default()
            },
            UiRoot,
        ))
        .with_children(|p| {
            menu::main::spawn(
                &textures,
                p,
                &ui_theme,
                &button_bundle,
                &ui_theme.frame_col_style,
            );
            menu::account::spawn_create(&textures, p, &ui_theme, &button_bundle, &frame_hidden);
            menu::account::spawn_login(&textures, p, &ui_theme, &button_bundle, &frame_hidden);
            menu::account::spawn_list(&textures, p, &ui_theme, &button_bundle, &frame_hidden);
            menu::create::spawn(&textures, p, &ui_theme, &button_bundle, &frame_hidden);
            menu::settings::spawn(p, &ui_theme, &button_bundle, &frame_hidden);
            menu::credits::spawn(p, &ui_theme, &button_bundle, &frame_hidden);

            // FIXME these are here temporarily
            chat::spawn(&textures, p, &ui_theme, &button_bundle, &frame_hidden);
            lobby::spawn(&textures, p, &ui_theme, &button_bundle, &frame_hidden);
        });

    state.set(AppState::Menu);
}
