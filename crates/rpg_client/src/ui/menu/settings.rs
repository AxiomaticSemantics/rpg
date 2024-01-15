use crate::ui::menu::main::MainRoot;

use ui_util::style::UiTheme;

use bevy::{
    ecs::prelude::*,
    hierarchy::{BuildChildren, ChildBuilder},
    ui::{
        node_bundles::{ButtonBundle, NodeBundle, TextBundle},
        BorderColor, Display, Interaction, Style,
    },
    utils::default,
};

#[derive(Component)]
pub struct SettingsRoot;

#[derive(Component)]
pub struct CancelButton;

#[derive(Component)]
pub struct ControlsButton;

#[derive(Component)]
pub struct AudioButton;

#[derive(Component)]
pub struct VideoButton;

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

                    p.spawn((button.clone(), CancelButton)).with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Cancel",
                            ui_theme.text_style_small.clone(),
                        ));
                    });
                });

                p.spawn(NodeBundle {
                    style: ui_theme.row_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(
                        TextBundle::from_section("Foo", ui_theme.text_style_regular.clone())
                            .with_style(ui_theme.row_style.clone()),
                    );
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

pub fn controls_button(
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

pub fn audio_button(
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
