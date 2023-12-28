use crate::{
    game::plugin::{GameConfig, PlayerOptions},
    menu::main::MainRoot,
    state::AppState,
};

use ui_util::{
    style::{UiRoot, UiTheme},
    widgets::{EditText, List, ListPosition},
};

use rpg_core::class::Class;

use bevy::{
    ecs::prelude::*,
    hierarchy::prelude::*,
    prelude::*,
    text::prelude::*,
    ui::{prelude::*, BorderColor},
};

#[derive(Component)]
pub(crate) struct CreateRoot;

#[derive(Component)]
pub(crate) struct CreateButton;

#[derive(Component)]
pub(crate) struct CreatePlayerClass(Class);

#[derive(Component)]
pub(crate) struct CancelCreateButton;

pub(crate) fn spawn_create(
    builder: &mut ChildBuilder,
    ui_theme: &UiTheme,
    button: &ButtonBundle,
    frame: &Style,
    text_node_style: &Style,
    text_style: &TextStyle,
) {
    builder
        .spawn((
            CreateRoot,
            NodeBundle {
                style: frame.clone(),
                background_color: ui_theme.frame_background_color,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(
                TextBundle::from_section("Select a Class", text_style.clone())
                    .with_style(text_node_style.clone()),
            );

            for (class, name) in [
                (CreatePlayerClass(Class::Str), "Warrior"),
                (CreatePlayerClass(Class::Dex), "Ranger"),
                (CreatePlayerClass(Class::Int), "Wizard"),
                (CreatePlayerClass(Class::StrDex), "Duelist"),
                (CreatePlayerClass(Class::DexInt), "Necromancer"),
                (CreatePlayerClass(Class::IntStr), "Cleric"),
                (CreatePlayerClass(Class::StrDexInt), "Rogue"),
            ] {
                p.spawn((button.clone(), class)).with_children(|p| {
                    p.spawn(TextBundle::from_section(name, text_style.clone()));
                });
            }

            p.spawn((button.clone(), CancelCreateButton))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section("Cancel", text_style.clone()));
                });
        });
}

pub(crate) fn cancel_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelCreateButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<CreateRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub(crate) fn create_class(
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
