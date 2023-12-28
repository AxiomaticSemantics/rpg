use crate::{
    game::plugin::{GameConfig, PlayerOptions},
    menu::main::MainRoot,
};

use ui_util::style::UiTheme;

use bevy::{
    ecs::prelude::*,
    hierarchy::prelude::*,
    prelude::*,
    text::prelude::*,
    ui::{prelude::*, BorderColor},
};

#[derive(Component)]
pub(crate) struct LoadRoot;

#[derive(Component)]
pub(crate) struct CancelLoadButton;

pub(crate) fn spawn_load(
    builder: &mut ChildBuilder,
    ui_theme: &UiTheme,
    button: &ButtonBundle,
    frame: &Style,
    text_node_style: &Style,
    text_style: &TextStyle,
) {
    builder
        .spawn((
            LoadRoot,
            NodeBundle {
                style: frame.clone(),
                background_color: ui_theme.frame_background_color,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(
                TextBundle::from_section("Load Character", text_style.clone())
                    .with_style(text_node_style.clone()),
            );

            p.spawn((button.clone(), CancelLoadButton))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section("Cancel", text_style.clone()));
                });
        });
}

pub(crate) fn cancel_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelLoadButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<LoadRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

/*
fn create_class(
    mut state: ResMut<NextState<AppState>>,
    mut game_config: ResMut<GameConfig>,
    interaction_q: Query<
        (&Interaction, &CreatePlayerClass),
        (Changed<Interaction>, With<CreatePlayerClass>),
    >,
    mut menu_root_q: Query<&mut Style, With<UiRoot>>,
) {
    if let Ok((Interaction::Pressed, create_class)) = interaction_q.get_single() {
        game_config.player_config = Some(PlayerOptions {
            name: "Player".to_string(),
            class: create_class.0,
        });

        menu_root_q.single_mut().display = Display::None;
        state.set(AppState::GameSpawn);
    }
}
*/
