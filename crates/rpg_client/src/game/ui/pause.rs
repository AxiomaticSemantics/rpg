#![allow(clippy::too_many_arguments)]

use crate::assets::TextureAssets;

use crate::game::plugin::{GameSessionCleanup, GameState, PlayState, SessionStats};

use ui_util::style::UiTheme;
use util::cleanup::CleanupStrategy;

use bevy::{
    ecs::{
        component::Component,
        event::EventWriter,
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

#[derive(Component)]
pub struct PauseView;

#[derive(Component)]
pub struct PauseStats;

#[derive(Component)]
pub struct SaveButton;

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

// FIXME pausing no longer exists, this is temporary
pub(crate) fn user_pause(
    mut game_state: ResMut<GameState>,
    input: Res<ButtonInput<KeyCode>>,
    mut pause_view_q: Query<&mut Style, With<PauseView>>,
    mut pause_stats_q: Query<&mut Text, With<PauseStats>>,
) {
    let mut pause_style = pause_view_q.single_mut();
    if input.just_pressed(KeyCode::Escape) {
        if pause_style.display == Display::None {
            let mut pause_stats = pause_stats_q.single_mut();
            pause_stats.sections[0].value = build_stats_string(&game_state.session_stats);
            pause_style.display = Display::Flex;
        } else {
            pause_style.display = Display::None;
        }
    }
}

pub(crate) fn game_exit_button_pressed(
    //mut exit_writer: EventWriter<ExitGame>,
    mut pause_view_q: Query<&mut Style, With<PauseView>>,
    button_q: Query<&Interaction, (With<SaveButton>, Changed<Interaction>)>,
) {
    let Ok(interaction) = button_q.get_single() else {
        return;
    };

    if interaction == &Interaction::Pressed {
        pause_view_q.single_mut().display = Display::None;
        // TODO send event to trigger sending exit game packet to server
    }
}

pub(crate) fn setup(
    mut commands: Commands,
    game_state: Res<GameState>,
    ui_theme: Res<UiTheme>,
    _textures: Res<TextureAssets>,
) {
    println!("setup game::ui::pause");

    let mut container_hidden_style = ui_theme.container_absolute_max.clone();
    container_hidden_style.display = Display::None;

    let vertical_spacing = NodeBundle {
        style: ui_theme.vertical_spacer.clone(),
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
            PauseView,
            NodeBundle {
                style: container_hidden_style.clone(),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(frame_col_node.clone()).with_children(|p| {
                p.spawn(col_node.clone()).with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Paused",
                        ui_theme.text_style_regular.clone(),
                    ));
                });

                p.spawn((
                    PauseStats,
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
                    SaveButton,
                    ButtonBundle {
                        style: ui_theme.button_theme.style.clone(),
                        border_color: BorderColor(Color::rgb(0.3, 0.3, 0.3)),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Save",
                        ui_theme.text_style_regular.clone(),
                    ));
                });
            });
        });
}
