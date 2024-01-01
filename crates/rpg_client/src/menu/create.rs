use crate::{
    assets::TextureAssets,
    game::plugin::{GameState, PlayerOptions},
    menu::main::MainRoot,
    state::AppState,
};

use ui_util::{
    style::{UiRoot, UiTheme},
    widgets::EditText,
};

use rpg_core::class::Class;

use bevy::{
    ecs::{
        component::Component,
        query::{Changed, With},
        schedule::NextState,
        system::{ParamSet, Query, ResMut, Resource},
    },
    hierarchy::{BuildChildren, ChildBuilder},
    prelude::{Deref, DerefMut},
    text::Text,
    ui::{
        node_bundles::{ButtonBundle, ImageBundle, NodeBundle, TextBundle},
        AlignItems, AlignSelf, Display, FocusPolicy, Interaction, JustifyContent, Style, UiImage,
        UiRect, Val,
    },
    utils::default,
};

#[derive(Component)]
pub struct CreatePlayerName;

#[derive(Component)]
pub struct CreateRoot;

#[derive(Component)]
pub struct CreateButton;

#[derive(Component)]
pub struct CreatePlayerClass(Class);

#[derive(Component)]
pub struct CancelCreateButton;

#[derive(Deref, DerefMut, Resource)]
pub struct SelectedClass(pub Class);

pub fn spawn(
    textures: &TextureAssets,
    builder: &mut ChildBuilder,
    ui_theme: &UiTheme,
    button: &ButtonBundle,
    frame: &Style,
) {
    let mut row_centered = ui_theme.row_style.clone();
    row_centered.align_self = AlignSelf::Center;

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
            p.spawn(NodeBundle {
                style: ui_theme.col_style.clone(),
                ..default()
            })
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: row_centered.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(
                        TextBundle::from_section(
                            "Select a Class",
                            ui_theme.text_style_regular.clone(),
                        )
                        .with_style(ui_theme.row_style.clone()),
                    );
                });

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
                                p.spawn(TextBundle::from_section(
                                    name,
                                    ui_theme.text_style_small.clone(),
                                ));
                            });
                        }
                    });

                    p.spawn(NodeBundle {
                        style: ui_theme.col_style.clone(),
                        ..default()
                    })
                    .with_children(|p| {
                        p.spawn(NodeBundle {
                            style: ui_theme.frame_row_style.clone(),
                            ..default()
                        })
                        .with_children(|p| {
                            p.spawn((TextBundle::from_section(
                                "Name:",
                                ui_theme.text_style_regular.clone(),
                            ),));

                            let mut edit_style = ui_theme.frame_row_style.clone();

                            edit_style.border = UiRect::all(ui_theme.border);
                            //edit_style.padding = UiRect::all(ui_theme.padding);
                            //edit_style.height = Val::Px(ui_theme.font_size_regular + 12.);

                            //edit_style.align_items = AlignItems::Center;
                            //edit_style.align_self = AlignSelf::Center;

                            p.spawn(NodeBundle {
                                style: edit_style.clone(),
                                border_color: ui_theme.frame_border_color,
                                background_color: ui_theme.menu_background_color,
                                ..default()
                            })
                            .with_children(|p| {
                                p.spawn((
                                    CreatePlayerName,
                                    EditText::default(),
                                    Interaction::None,
                                    TextBundle {
                                        text: Text::from_section(
                                            " ",
                                            ui_theme.text_style_regular.clone(),
                                        ),
                                        style: Style {
                                            height: Val::Px(ui_theme.font_size_regular + 12.),
                                            width: Val::Px(128.0),
                                            ..default()
                                        },
                                        focus_policy: FocusPolicy::Pass,
                                        ..default()
                                    },
                                ));
                            });
                        });
                        p.spawn(NodeBundle {
                            style: ui_theme.frame_row_style.clone(),
                            ..default()
                        })
                        .with_children(|p| {
                            p.spawn(TextBundle {
                                text: Text::from_section(
                                    "Hardcore:",
                                    ui_theme.text_style_regular.clone(),
                                ),
                                ..default()
                            });

                            let mut button_style = Style {
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                min_width: Val::Px(28.),
                                min_height: Val::Px(28.),
                                max_width: Val::Px(28.),
                                max_height: Val::Px(28.),
                                padding: UiRect::all(ui_theme.padding),
                                border: UiRect::all(ui_theme.border),
                                ..default()
                            };

                            p.spawn(ButtonBundle {
                                style: button_style.clone(),
                                background_color: ui_theme.button_theme.normal_background_color,
                                border_color: ui_theme.border_color,
                                ..default()
                            })
                            .with_children(|p| {
                                p.spawn(ImageBundle {
                                    image: UiImage {
                                        texture: textures.icons["checkmark"].clone_weak(),

                                        ..default()
                                    },
                                    style: Style {
                                        max_width: Val::Px(22.),
                                        min_height: Val::Px(22.),
                                        //display: Display::None,
                                        ..default()
                                    },
                                    //transform: Transform::from_scale(Vec3::splat(0.5)),
                                    ..default()
                                });
                            });
                        });
                    });
                });
            });

            p.spawn(NodeBundle {
                style: ui_theme.row_style.clone(),
                ..default()
            })
            .with_children(|p| {
                p.spawn((button.clone(), CancelCreateButton))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Cancel",
                            ui_theme.text_style_regular.clone(),
                        ));
                    });
            });
        });
}

pub fn cancel_button(
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

pub fn create_class(
    mut state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<GameState>,
    interaction_q: Query<
        (&Interaction, &CreatePlayerClass),
        (Changed<Interaction>, With<CreatePlayerClass>),
    >,
    mut menu_root_q: Query<&mut Style, With<UiRoot>>,
    player_name_text_q: Query<&Text, With<CreatePlayerName>>,
) {
    let player_name_text = player_name_text_q.single();
    if player_name_text.sections[0].value.is_empty() {
        return;
    }

    if let Ok((Interaction::Pressed, create_class)) = interaction_q.get_single() {
        game_state.player_config = Some(PlayerOptions {
            name: "Player".to_string(),
            class: create_class.0,
        });

        menu_root_q.single_mut().display = Display::None;
        state.set(AppState::GameSpawn);
    }
}
