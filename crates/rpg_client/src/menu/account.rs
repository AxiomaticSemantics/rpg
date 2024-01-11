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

use rpg_core::{class::Class, unit::HeroGameMode};
use rpg_network_protocol::protocol::*;

use lightyear::prelude::*;

use bevy::{
    ecs::{
        component::Component,
        query::{Changed, With},
        schedule::NextState,
        system::{ParamSet, Query, Res, ResMut, Resource},
    },
    hierarchy::{BuildChildren, ChildBuilder},
    log::*,
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
pub struct AccountRoot;

#[derive(Component)]
pub struct AccountCreateName;

#[derive(Component)]
pub struct AccountCreateEmail;

#[derive(Component)]
pub struct AccountCreatePassword;

#[derive(Component)]
pub struct AccountLoginName;

#[derive(Component)]
pub struct AccountLoginPassword;

#[derive(Component)]
pub struct AccountCreateButton;

#[derive(Component)]
pub struct AccountLoginButton;

#[derive(Component)]
pub struct CancelButton;

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
            AccountRoot,
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
                            "Create Account",
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
                    /*p.spawn(NodeBundle {
                        style: ui_theme.col_style.clone(),
                        ..default()
                    })
                    .with_children(|p| {
                    });*/

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
                                    AccountCreateName,
                                    EditText::default(),
                                    Interaction::None,
                                    TextBundle {
                                        text: Text::from_section(
                                            "",
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
                                "Email:",
                                ui_theme.text_style_regular.clone(),
                            ),));

                            let mut edit_style = ui_theme.frame_row_style.clone();

                            edit_style.border = UiRect::all(ui_theme.border);

                            p.spawn(NodeBundle {
                                style: edit_style.clone(),
                                border_color: ui_theme.frame_border_color,
                                background_color: ui_theme.menu_background_color,
                                ..default()
                            })
                            .with_children(|p| {
                                p.spawn((
                                    AccountCreateEmail,
                                    EditText::default(),
                                    Interaction::None,
                                    TextBundle {
                                        text: Text::from_section(
                                            "",
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
                                    "Password:",
                                    ui_theme.text_style_regular.clone(),
                                ),));

                                let mut edit_style = ui_theme.frame_row_style.clone();

                                edit_style.border = UiRect::all(ui_theme.border);

                                p.spawn(NodeBundle {
                                    style: edit_style.clone(),
                                    border_color: ui_theme.frame_border_color,
                                    background_color: ui_theme.menu_background_color,
                                    ..default()
                                })
                                .with_children(|p| {
                                    p.spawn((
                                        AccountCreatePassword,
                                        EditText::default(),
                                        Interaction::None,
                                        TextBundle {
                                            text: Text::from_section(
                                                "",
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
                        });
                        /*
                            p.spawn(NodeBundle {
                                style: ui_theme.frame_row_style.clone(),
                                ..default()
                            })
                            .with_children(|p| {
                                p.spawn(TextBundle::from_section(
                                    "Email:",
                                    ui_theme.text_style_regular.clone(),
                                ));

                                /*
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

                                p.spawn(NodeBundle {
                                    style: button_style.clone(),
                                    background_color: ui_theme.button_theme.normal_background_color,
                                    border_color: ui_theme.border_color,
                                    ..default()
                                })
                                .with_children(|p| {
                                    p.spawn((
                                        //CreateMode(HeroGameMode::Normal),
                                        Interaction::None,
                                        ImageBundle {
                                            image: UiImage {
                                                texture: textures.icons["transparent"].clone_weak(),
                                                ..default()
                                            },
                                            style: Style {
                                                max_width: Val::Px(24.),
                                                min_height: Val::Px(24.),
                                                ..default()
                                            },
                                            ..default()
                                        },
                                    ));
                                });
                                */
                            });
                        */
                    });
                });
                p.spawn(NodeBundle {
                    style: ui_theme.row_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn((button.clone(), AccountCreateButton))
                        .with_children(|p| {
                            p.spawn(TextBundle::from_section(
                                "Create",
                                ui_theme.text_style_regular.clone(),
                            ));
                        });
                    // });
                });
            });

            p.spawn(NodeBundle {
                style: ui_theme.row_style.clone(),
                ..default()
            })
            .with_children(|p| {
                p.spawn((button.clone(), CancelButton)).with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Cancel",
                        ui_theme.text_style_regular.clone(),
                    ));
                });
            });
        });
}

pub fn create_button(
    mut net_client: ResMut<Client>,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<AccountCreateButton>)>,
    mut account_text_set: ParamSet<(
        Query<&Text, With<AccountCreateName>>,
        Query<&Text, With<AccountCreateEmail>>,
        Query<&Text, With<AccountCreatePassword>>,
    )>,
) {
    for interaction in &interaction_q {
        //
        if *interaction != Interaction::Pressed {
            continue;
        }

        let name = account_text_set.p0().single().sections[0].value.clone();
        let email = account_text_set.p1().single().sections[0].value.clone();
        let password = account_text_set.p2().single().sections[0].value.clone();

        if name.is_empty() {
            info!("account create: no name provided, skipping");
            continue;
        }

        if email.is_empty() {
            info!("account create: no email provided, skipping");
            continue;
        }

        if password.is_empty() {
            info!("account create: no password provided skipping");
            continue;
        }

        // TODO some basic validation of input
        let create_msg = CSCreateAccount {
            name,
            email,
            password,
        };

        net_client.send_message::<Channel1, _>(create_msg);
        info!("sending create account message");
    }
}

pub fn login_button(
    net_client: Res<Client>,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<AccountLoginButton>)>,
    account_text_set: ParamSet<(
        Query<&Text, With<AccountLoginName>>,
        Query<&Text, With<AccountLoginPassword>>,
    )>,
) {
    for interaction in &interaction_q {
        //
        if *interaction != Interaction::Pressed {
            continue;
        }
    }
}

pub fn cancel_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<AccountRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

/*
pub fn set_game_mode(
    textures: Res<TextureAssets>,
    mut game_mode_q: Query<(&mut CreateMode, &mut UiImage, &Interaction), Changed<Interaction>>,
) {
    let Ok((mut game_mode, mut ui_image, interaction)) = game_mode_q.get_single_mut() else {
        return;
    };

    if interaction == &Interaction::Pressed {
        if game_mode.0 == HeroGameMode::Normal {
            ui_image.texture = textures.icons["checkmark"].clone_weak();
            game_mode.0 = HeroGameMode::Hardcore;
        } else {
            ui_image.texture = textures.icons["transparent"].clone_weak();
            game_mode.0 = HeroGameMode::Normal;
        }
    }
}

pub fn create_class(
    mut state: ResMut<NextState<AppState>>,
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
        menu_root_q.single_mut().display = Display::None;
        state.set(AppState::GameSpawn);
    }
}
*/
