use crate::ui::menu::main::MainRoot;

use ui_util::style::UiTheme;

use bevy::{
    ecs::{
        component::Component,
        query::{Changed, With},
        system::{ParamSet, Query},
    },
    hierarchy::{BuildChildren, ChildBuilder},
    ui::{
        node_bundles::{ButtonBundle, NodeBundle, TextBundle},
        Display, Interaction, Style,
    },
    utils::default,
};

#[derive(Component)]
pub struct SettingsRoot;

#[derive(Component)]
pub struct ControlSettingsRoot;

#[derive(Component)]
pub struct AudioSettingsRoot;

#[derive(Component)]
pub struct VideoSettingsRoot;

#[derive(Component)]
pub struct CancelButton;

#[derive(Component)]
pub struct CancelControlButton;

#[derive(Component)]
pub struct CancelAudioButton;

#[derive(Component)]
pub struct CancelVideoButton;

#[derive(Component)]
pub struct ControlButton;

#[derive(Component)]
pub struct AudioButton;

#[derive(Component)]
pub struct VideoButton;

pub fn spawn_control_settings(
    builder: &mut ChildBuilder,
    ui_theme: &UiTheme,
    button: &ButtonBundle,
    frame: &Style,
) {
    builder
        .spawn((
            ControlSettingsRoot,
            NodeBundle {
                style: frame.clone(),
                ..default()
            },
        ))
        .with_children(|b| {
            b.spawn(TextBundle::from_section(
                "Move",
                ui_theme.text_style_regular.clone(),
            ));

            b.spawn(TextBundle::from_section(
                "LMB",
                ui_theme.text_style_regular.clone(),
            ));

            b.spawn((button.clone(), CancelControlButton))
                .with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "Cancel",
                        ui_theme.text_style_small.clone(),
                    ));
                });
        });
}

pub fn spawn_audio_settings(
    builder: &mut ChildBuilder,
    ui_theme: &UiTheme,
    button: &ButtonBundle,
    frame: &Style,
) {
    builder
        .spawn((
            AudioSettingsRoot,
            NodeBundle {
                style: frame.clone(),
                ..default()
            },
        ))
        .with_children(|b| {
            b.spawn(TextBundle::from_section(
                "Volume",
                ui_theme.text_style_regular.clone(),
            ));

            b.spawn(TextBundle::from_section(
                "Slider",
                ui_theme.text_style_regular.clone(),
            ));

            b.spawn((button.clone(), CancelAudioButton))
                .with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "Cancel",
                        ui_theme.text_style_small.clone(),
                    ));
                });
        });
}

pub fn spawn_video_settings(
    builder: &mut ChildBuilder,
    ui_theme: &UiTheme,
    button: &ButtonBundle,
    frame: &Style,
) {
    builder
        .spawn((
            VideoSettingsRoot,
            NodeBundle {
                style: frame.clone(),
                ..default()
            },
        ))
        .with_children(|b| {
            b.spawn(TextBundle::from_section(
                "Resolution",
                ui_theme.text_style_regular.clone(),
            ));

            b.spawn(TextBundle::from_section(
                "ListSelect",
                ui_theme.text_style_regular.clone(),
            ));

            b.spawn((button.clone(), CancelVideoButton))
                .with_children(|b| {
                    b.spawn(TextBundle::from_section(
                        "Cancel",
                        ui_theme.text_style_small.clone(),
                    ));
                });
        });
}

pub fn spawn(builder: &mut ChildBuilder, ui_theme: &UiTheme, button: &ButtonBundle, frame: &Style) {
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
                TextBundle::from_section("Settings", ui_theme.text_style_regular.clone())
                    .with_style(ui_theme.row_style.clone()),
            );

            p.spawn(NodeBundle {
                style: ui_theme.col_style.clone(),
                ..default()
            })
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: ui_theme.col_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((button.clone(), ControlButton)).with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Controls",
                            ui_theme.text_style_small.clone(),
                        ));
                    });

                    p.spawn((button.clone(), VideoButton)).with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Video",
                            ui_theme.text_style_small.clone(),
                        ));
                    });

                    p.spawn((button.clone(), AudioButton)).with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Audio",
                            ui_theme.text_style_small.clone(),
                        ));
                    });
                });

                p.spawn(NodeBundle {
                    style: ui_theme.col_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((button.clone(), CancelButton)).with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Cancel",
                            ui_theme.text_style_small.clone(),
                        ));
                    });
                });
            });
        });
}

pub fn cancel_button(
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

pub fn cancel_controls_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelControlButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<SettingsRoot>>,
        Query<&mut Style, With<ControlSettingsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub fn cancel_audio_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelAudioButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<SettingsRoot>>,
        Query<&mut Style, With<AudioSettingsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub fn cancel_video_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelVideoButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<SettingsRoot>>,
        Query<&mut Style, With<VideoSettingsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub fn controls_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ControlButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<SettingsRoot>>,
        Query<&mut Style, With<ControlSettingsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::Flex;
    }
}

pub fn audio_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<AudioButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<SettingsRoot>>,
        Query<&mut Style, With<AudioSettingsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::Flex;
    }
}

pub fn video_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<VideoButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<SettingsRoot>>,
        Query<&mut Style, With<VideoSettingsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::Flex;
    }
}
