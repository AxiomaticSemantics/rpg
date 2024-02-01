#![allow(clippy::too_many_arguments)]

use crate::{assets::TextureAssets, state::AppState};

use crate::game::plugin::{
    build_stats_string, GameOverState, GameSessionCleanup, GameState, PlayState,
};

use ui_util::style::UiTheme;
use util::cleanup::CleanupStrategy;

use bevy::{
    ecs::{
        component::Component,
        query::{Changed, With},
        schedule::NextState,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::BuildChildren,
    log::info,
    text::Text,
    ui::{
        node_bundles::{ButtonBundle, NodeBundle, TextBundle},
        AlignSelf, BackgroundColor, Display, Interaction, JustifyContent, Style,
    },
    utils::default,
};

#[derive(Component)]
pub struct GameOverView;

#[derive(Component)]
pub struct GameOverStats;

#[derive(Component)]
pub struct RestartGame;

#[derive(Component)]
pub struct ReturnToMenu;

#[derive(Component)]
pub(crate) struct PlayerName;

pub(crate) fn game_over_transition(
    game_state: ResMut<GameState>,
    mut state: ResMut<NextState<AppState>>,
    mut gameover_view_q: Query<&mut Style, With<GameOverView>>,
    mut gameover_stats_q: Query<&mut Text, With<GameOverStats>>,
) {
    let mut view = gameover_view_q.single_mut();

    if let PlayState::Death(GameOverState::Pending) = &game_state.state {
        if view.display == Display::None {
            view.display = Display::Flex;

            //} else if let PlayState::Death(GameOverState::Exit) = &game_state.state {
            let mut stats = gameover_stats_q.single_mut();
            // FIXME stats.sections[0].value = build_stats_string(&game_state.session_stats);
        }
    } else if let PlayState::Death(GameOverState::Exit) = &game_state.state {
        state.set(AppState::GameCleanup);
    }
}

pub(crate) fn restart_button(
    mut state: ResMut<NextState<AppState>>,
    ui_theme: Res<UiTheme>,
    mut game_state: ResMut<GameState>,
    mut restart_q: Query<
        (&Interaction, &mut BackgroundColor),
        (With<RestartGame>, Changed<Interaction>),
    >,
) {
    if let Ok((interaction, mut bg_color)) = restart_q.get_single_mut() {
        match interaction {
            Interaction::Pressed => {
                game_state.state = PlayState::Death(GameOverState::Restart);
                state.set(AppState::GameSpawn);
                info!("game_over: respawn request");
            }
            Interaction::Hovered => *bg_color = ui_theme.button_theme.hovered_background_color,
            Interaction::None => *bg_color = ui_theme.button_theme.normal_background_color,
        }
    }
}

pub(crate) fn exit_button(
    mut state: ResMut<NextState<AppState>>,
    ui_theme: Res<UiTheme>,
    mut game_state: ResMut<GameState>,
    mut menu_q: Query<
        (&Interaction, &mut BackgroundColor),
        (With<ReturnToMenu>, Changed<Interaction>),
    >,
) {
    if let Ok((interaction, mut bg_color)) = menu_q.get_single_mut() {
        match interaction {
            Interaction::Pressed => {
                println!("game_over: returning to menu");
                game_state.state = PlayState::Death(GameOverState::Exit);

                state.set(AppState::GameCleanup);
            }
            Interaction::Hovered => *bg_color = ui_theme.button_theme.hovered_background_color,
            Interaction::None => *bg_color = ui_theme.button_theme.normal_background_color,
        }
    }
}

pub(crate) fn setup(mut commands: Commands, ui_theme: Res<UiTheme>, _textures: Res<TextureAssets>) {
    let mut container_hidden_style = ui_theme.container_absolute_max.clone();
    container_hidden_style.display = Display::None;

    let vertical_spacing = NodeBundle {
        style: ui_theme.vertical_spacer.clone(),
        ..default()
    };

    let horizontal_spacing = NodeBundle {
        style: ui_theme.horizontal_spacer.clone(),
        ..default()
    };

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

    let frame_row_node = NodeBundle {
        style: ui_theme.frame_row_style.clone(),
        border_color: ui_theme.border_color,
        background_color: ui_theme.frame_background_color,
        ..default()
    };

    // Game Over view
    commands
        .spawn((
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            GameOverView,
            NodeBundle {
                style: container_hidden_style.clone(),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(frame_col_node.clone()).with_children(|p| {
                p.spawn(TextBundle {
                    text: Text::from_section("Game Over", ui_theme.text_style_regular.clone()),
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_self: AlignSelf::Center,
                        ..default()
                    },
                    ..default()
                });

                p.spawn(vertical_spacing.clone());

                p.spawn((
                    GameOverStats,
                    TextBundle {
                        text: Text::from_section("", ui_theme.text_style_regular.clone()),
                        ..default()
                    },
                ));

                p.spawn(vertical_spacing.clone());

                p.spawn(row_node.clone()).with_children(|p| {
                    p.spawn(frame_row_node.clone()).with_children(|p| {
                        p.spawn((
                            RestartGame,
                            ButtonBundle {
                                style: ui_theme.button_theme.style.clone(),
                                background_color: ui_theme.button_theme.normal_background_color,
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            p.spawn(TextBundle {
                                text: Text::from_section(
                                    "Restart",
                                    ui_theme.text_style_regular.clone(),
                                ),
                                ..default()
                            });
                        });
                        //});

                        p.spawn(horizontal_spacing.clone());

                        //p.spawn(frame_row_node.clone()).with_children(|p| {
                        p.spawn((
                            ReturnToMenu,
                            ButtonBundle {
                                style: ui_theme.button_theme.style.clone(),
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            p.spawn(TextBundle {
                                text: Text::from_section(
                                    "Return to Menu",
                                    ui_theme.text_style_regular.clone(),
                                ),
                                ..default()
                            });
                        });
                    });
                });

                p.spawn(vertical_spacing.clone());
            });
        });
}
