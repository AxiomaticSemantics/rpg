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
pub struct CreditsRoot;

#[derive(Component)]
pub struct CancelButton;

pub fn spawn(builder: &mut ChildBuilder, ui_theme: &UiTheme, button: &ButtonBundle, frame: &Style) {
    builder
        .spawn((
            CreditsRoot,
            NodeBundle {
                style: frame.clone(),
                background_color: ui_theme.frame_background_color,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(
                TextBundle::from_section("Credits", ui_theme.text_style_regular.clone())
                    .with_style(ui_theme.row_style.clone()),
            );

            p.spawn(TextBundle::from_section(
                "UnnamedRPG\nCopyright (c) 2023-2024\n\n\
                Powered by Bevy Engine - https://bevyengine.org\n",
                ui_theme.text_style_small.clone(),
            ));

            p.spawn((button.clone(), CancelButton)).with_children(|p| {
                p.spawn(TextBundle::from_section(
                    "Cancel",
                    ui_theme.text_style_regular.clone(),
                ));
            });
        });
}

pub fn cancel_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<CreditsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}
