use crate::{
    assets::TextureAssets,
    game::state_saver::SaveSlots,
    loader::plugin::OutOfGameCamera,
    menu::{
        self, create::CreateRoot, credits::CreditsRoot, load::LoadRoot, main::MainRoot,
        settings::SettingsRoot,
    },
    state::AppState,
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
        app.init_resource::<menu::load::SelectedSaveSlot>()
            .add_systems(OnEnter(AppState::MenuLoad), spawn)
            .add_systems(OnEnter(AppState::Menu), display_menu)
            .add_systems(
                Update,
                (
                    menu::main::exit_button,
                    menu::main::create_button,
                    menu::main::load_button,
                    menu::main::settings_button,
                    menu::main::credits_button,
                    menu::create::cancel_button,
                    menu::create::create_class,
                    menu::create::set_game_mode,
                    menu::load::cancel_button,
                    menu::load::select_save_slot,
                    menu::load::load_button,
                    menu::settings::cancel_button,
                    menu::settings::controls_button,
                    menu::settings::audio_button,
                    menu::credits::cancel_button,
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
        Query<&mut Style, With<CreateRoot>>,
        Query<&mut Style, With<LoadRoot>>,
        Query<&mut Style, With<SettingsRoot>>,
        Query<&mut Style, With<CreditsRoot>>,
    )>,
    mut camera_q: Query<&mut Camera, (With<Camera2d>, With<OutOfGameCamera>)>,
) {
    let mut camera = camera_q.single_mut();
    if !camera.is_active {
        camera.is_active = true;
    }

    menu_set.p0().single_mut().display = Display::Flex;
    menu_set.p1().single_mut().display = Display::Flex;
    menu_set.p2().single_mut().display = Display::None;
    menu_set.p3().single_mut().display = Display::None;
    menu_set.p4().single_mut().display = Display::None;
    menu_set.p5().single_mut().display = Display::None;
}

fn spawn(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    save_slots: Res<SaveSlots>,
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
            //border: UiRect::all(ui_theme.border),
            margin: UiRect::all(ui_theme.margin),
            //padding: UiRect::all(ui_theme.padding),
            justify_content: JustifyContent::Center,
            align_self: AlignSelf::Center,
            align_items: AlignItems::Center,
            ..default()
        },

        background_color: Color::NONE.into(),
        //border_color: ui_theme.border_color,
        //background_color: ui_theme.button_theme.normal_background_color,
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
            super::main::spawn(
                &textures,
                p,
                &ui_theme,
                &button_bundle,
                &ui_theme.frame_col_style,
            );
            super::create::spawn(&textures, p, &ui_theme, &button_bundle, &frame_hidden);
            super::load::spawn(p, &ui_theme, &button_bundle, &frame_hidden, &save_slots);
            super::settings::spawn(p, &ui_theme, &button_bundle, &frame_hidden);
            super::credits::spawn(p, &ui_theme, &button_bundle, &frame_hidden);
        });

    println!("transition `AppState::Menu`");
    state.set(AppState::Menu);
}
