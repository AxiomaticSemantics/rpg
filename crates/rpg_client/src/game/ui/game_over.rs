#![allow(clippy::too_many_arguments)]

use crate::state::AppState;

use crate::game::plugin::{GameOverState, GameState, PlayState};

use ui_util::style::UiTheme;

use bevy::{
    ecs::{
        component::Component,
        query::{Changed, With},
        schedule::NextState,
        system::{Query, Res, ResMut},
    },
    log::info,
    text::Text,
    ui::{BackgroundColor, Display, Interaction, Style},
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
