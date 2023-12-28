use crate::menu::main::MainRoot;

use ui_util::style::UiTheme;

use bevy::{
    ecs::prelude::*,
    hierarchy::{BuildChildren, ChildBuilder},
    text::prelude::*,
    ui::{prelude::*, BorderColor},
    utils::default,
};

use util::cleanup::CleanupStrategy;

#[derive(Component)]
pub(crate) struct SettingsRoot;

#[derive(Component)]
pub(crate) struct CancelButton;

#[derive(Component)]
pub(crate) struct ControlsButton;

#[derive(Component)]
pub(crate) struct AudioButton;

#[derive(Component)]
pub(crate) struct VideoButton;

pub(crate) fn spawn_settings(
    builder: &mut ChildBuilder,
    ui_theme: &UiTheme,
    button: &ButtonBundle,
    frame: &Style,
    text_node_style: &Style,
    text_style: &TextStyle,
) {
    builder
        .spawn((
            SettingsRoot,
            NodeBundle {
                style: frame.clone(),
                background_color: ui_theme.frame_background_color,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(
                TextBundle::from_section("Settings", text_style.clone())
                    .with_style(text_node_style.clone()),
            );

            p.spawn(NodeBundle {
                style: ui_theme.row_style.clone(),
                ..default()
            })
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: ui_theme.col_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((button.clone(), ControlsButton))
                        .with_children(|p| {
                            p.spawn(TextBundle::from_section("Controls", text_style.clone()));
                        });

                    p.spawn((button.clone(), VideoButton)).with_children(|p| {
                        p.spawn(TextBundle::from_section("Video", text_style.clone()));
                    });

                    p.spawn((button.clone(), AudioButton)).with_children(|p| {
                        p.spawn(TextBundle::from_section("Audio", text_style.clone()));
                    });

                    p.spawn((button.clone(), CancelButton)).with_children(|p| {
                        p.spawn(TextBundle::from_section("Cancel", text_style.clone()));
                    });
                });

                p.spawn(NodeBundle {
                    style: ui_theme.row_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(
                        TextBundle::from_section("Foo", text_style.clone())
                            .with_style(text_node_style.clone()),
                    );
                });
            });
        });
}

pub(crate) fn cancel_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<SettingsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub(crate) fn controls_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ControlsButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<SettingsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub(crate) fn audio_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<AudioButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<SettingsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}
