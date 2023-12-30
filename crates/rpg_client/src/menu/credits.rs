use crate::menu::main::MainRoot;

use ui_util::{
    style::UiTheme,
    widgets::{EditText, List, ListPosition},
};

use bevy::{
    ecs::prelude::*,
    hierarchy::prelude::*,
    prelude::default,
    text::prelude::*,
    ui::{prelude::*, BorderColor},
};

#[derive(Component)]
pub struct CreditsRoot;

#[derive(Component)]
pub struct CancelButton;

pub fn spawn_credits(
    builder: &mut ChildBuilder,
    ui_theme: &UiTheme,
    button: &ButtonBundle,
    frame: &Style,
    text_node_style: &Style,
    text_style: &TextStyle,
) {
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
                TextBundle::from_section("Credits", text_style.clone())
                    .with_style(text_node_style.clone()),
            );

            p.spawn(TextBundle::from_section(
                "UnknownSurvivalRPG\nCopyright (c) 2023\n\n\
                Powered by Bevy Engine - https://bevyengine.org\n",
                text_style.clone(),
            ));

            p.spawn((button.clone(), CancelButton)).with_children(|p| {
                p.spawn(TextBundle::from_section("Cancel", text_style.clone()));
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
