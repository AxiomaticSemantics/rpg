#![allow(clippy::too_many_arguments)]

use crate::assets::TextureAssets;

use crate::game::plugin::{build_stats_string, GameSessionCleanup, GameState};

use ui_util::style::UiTheme;
use util::cleanup::CleanupStrategy;

use bevy::{
    ecs::{
        component::Component,
        query::{Changed, With},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::BuildChildren,
    input::{keyboard::KeyCode, ButtonInput},
    render::color::Color,
    text::Text,
    ui::{
        node_bundles::{ButtonBundle, ImageBundle, NodeBundle, TextBundle},
        AlignSelf, BackgroundColor, BorderColor, Display, FlexDirection, Interaction, Style,
    },
    utils::default,
};

use rpg_network_protocol::protocol::*;

#[derive(Component)]
pub struct MenuView;

#[derive(Component)]
pub struct GameStats;

#[derive(Component)]
pub struct ExitButton;

#[derive(Component)]
pub struct CancelButton;

pub(crate) fn toggle_menu(
    input: Res<ButtonInput<KeyCode>>,
    mut menu_q: Query<&mut Style, With<MenuView>>,
    mut stats_q: Query<&mut Text, With<GameStats>>,
) {
    let mut style = menu_q.single_mut();
    if input.just_pressed(KeyCode::Escape) {
        if style.display == Display::None {
            style.display = Display::Flex;

            /* FIXME
            let mut stats = stats_q.single_mut();
            stats.sections[0].value = build_stats_string(&game_state.session_stats);
            */
        } else {
            style.display = Display::None;
        }
    }
}

pub(crate) fn exit_button(
    mut net_client: ResMut<Client>,
    mut menu_q: Query<&mut Style, With<MenuView>>,
    exit_button_q: Query<&Interaction, (With<ExitButton>, Changed<Interaction>)>,
) {
    let Ok(interaction) = exit_button_q.get_single() else {
        return;
    };

    if interaction == &Interaction::Pressed {
        menu_q.single_mut().display = Display::None;
        net_client.send_message::<Channel1, _>(CSPlayerLeave);
    }
}

pub(crate) fn cancel_button(
    mut menu_q: Query<&mut Style, With<MenuView>>,
    cancel_button_q: Query<&Interaction, (With<CancelButton>, Changed<Interaction>)>,
) {
    let Ok(interaction) = cancel_button_q.get_single() else {
        return;
    };

    if interaction == &Interaction::Pressed {
        menu_q.single_mut().display = Display::None;
    }
}

pub(crate) fn setup(mut commands: Commands, ui_theme: Res<UiTheme>, _textures: Res<TextureAssets>) {
    let mut container_hidden_style = ui_theme.container_absolute_max.clone();
    container_hidden_style.display = Display::None;

    let row_node = NodeBundle {
        style: ui_theme.row_style.clone(),
        ..default()
    };

    let col_node = NodeBundle {
        style: ui_theme.col_style.clone(),
        ..default()
    };

    let frame_col_node = NodeBundle {
        style: ui_theme.frame_col_style.clone(),
        border_color: ui_theme.border_color,
        background_color: ui_theme.frame_background_color,
        ..default()
    };

    /*
    let frame_row_node = NodeBundle {
        style: ui_theme.frame_row_style.clone(),
        background_color: ui_theme.menu_background_color,
        ..default()
    };*/

    // Pause view
    commands
        .spawn((
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            MenuView,
            NodeBundle {
                style: container_hidden_style.clone(),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(frame_col_node.clone()).with_children(|p| {
                p.spawn(col_node.clone()).with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Stats",
                        ui_theme.text_style_regular.clone(),
                    ));
                });

                p.spawn((
                    GameStats,
                    TextBundle {
                        text: Text::from_section("", ui_theme.text_style_regular.clone()),
                        style: Style {
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        background_color: ui_theme.frame_background_color,
                        ..default()
                    },
                ));

                p.spawn((
                    ExitButton,
                    ButtonBundle {
                        style: ui_theme.button_theme.style.clone(),
                        border_color: BorderColor(Color::rgb(0.3, 0.3, 0.3)),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Exit",
                        ui_theme.text_style_regular.clone(),
                    ));
                });

                p.spawn((
                    CancelButton,
                    ButtonBundle {
                        style: ui_theme.button_theme.style.clone(),
                        border_color: BorderColor(Color::rgb(0.3, 0.3, 0.3)),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Cancel",
                        ui_theme.text_style_regular.clone(),
                    ));
                });
            });
        });
}
