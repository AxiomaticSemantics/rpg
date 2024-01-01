#![allow(clippy::too_many_arguments)]

use crate::{assets::TextureAssets, state::AppState};

use crate::game::plugin::{GameOverState, GameSessionCleanup, GameState, PlayState, SessionStats};

use ui_util::style::UiTheme;
use util::cleanup::CleanupStrategy;

use bevy::{
    ecs::{
        component::Component,
        query::{Changed, With},
        schedule::NextState,
        system::{Commands, ParamSet, Query, Res, ResMut},
    },
    hierarchy::BuildChildren,
    render::color::Color,
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

fn build_stats_string(stats: &SessionStats) -> String {
    format!(
        "Villain Stats:\nSpawned: {} Killed: {} Attacks: {} Hits: {}\n\nPlayer Stats:\nAttacks: {} Hits: {}\nBlocks: {} Dodges: {} Times Blocked: {} Times Dodged: {}\n\nItems Stats:\nSpawned: {} Looted: {}",
        stats.spawned,
        stats.kills,
        stats.villain_attacks,
        stats.villain_hits,
        stats.attacks,
        stats.hits,
        stats.blocks,
        stats.dodges,
        stats.times_blocked,
        stats.times_dodged,
        stats.items_spawned,
        stats.items_looted,
    )
}

pub(crate) fn game_over_transition(
    game_state: ResMut<GameState>,
    mut state: ResMut<NextState<AppState>>,
    mut gameover_view_q: Query<&mut Style, With<GameOverView>>,
    mut gameover_stats_q: Query<&mut Text, With<GameOverStats>>,
) {
    if let PlayState::GameOver(GameOverState::Exit) = game_state.state {
        println!("game_over_transition");
        let mut stats = gameover_stats_q.single_mut();
        stats.sections[0].value = build_stats_string(&game_state.session_stats);
        let mut view = gameover_view_q.single_mut();
        view.display = Display::Flex;

        state.set(AppState::GameCleanup);
    } else if let PlayState::GameOver(GameOverState::Saved) = game_state.state {
        state.set(AppState::GameCleanup);
    }
}

pub(crate) fn game_over(
    mut state: ResMut<NextState<AppState>>,
    ui_theme: Res<UiTheme>,
    mut game_state: ResMut<GameState>,
    mut interaction_set: ParamSet<(
        Query<(&Interaction, &mut BackgroundColor), (With<RestartGame>, Changed<Interaction>)>,
        Query<(&Interaction, &mut BackgroundColor), (With<ReturnToMenu>, Changed<Interaction>)>,
    )>,
) {
    let restart = &mut interaction_set.p0();
    for (interaction, mut bg_color) in restart.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                game_state.state = PlayState::GameOver(GameOverState::Restart);
                game_state.session_stats = SessionStats::default();
                state.set(AppState::GameSpawn);
                println!("game_over: restarting with current character");
            }
            Interaction::Hovered => *bg_color = ui_theme.button_theme.hovered_background_color,
            Interaction::None => *bg_color = ui_theme.button_theme.normal_background_color,
        }
    }

    let menu = &mut interaction_set.p1();
    for (interaction, mut bg_color) in menu.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                println!("game_over: returning to menu");
                game_state.state = PlayState::GameOver(GameOverState::Exit);

                state.set(AppState::Menu);
            }
            Interaction::Hovered => *bg_color = ui_theme.button_theme.hovered_background_color,
            Interaction::None => *bg_color = ui_theme.button_theme.normal_background_color,
        }
    }
}

pub(crate) fn setup(
    mut commands: Commands,
    game_state: Res<GameState>,
    ui_theme: Res<UiTheme>,
    _textures: Res<TextureAssets>,
) {
    println!("setup game::ui::game_over");

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
                    });

                    p.spawn(horizontal_spacing.clone());

                    p.spawn(frame_row_node.clone()).with_children(|p| {
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
