use crate::{
    assets::TextureAssets,
    game::plugin::{GameState, PlayerOptions},
    menu::main::MainRoot,
    net::account::RpgAccount,
    state::AppState,
};

use ui_util::{
    style::{UiRoot, UiTheme},
    widgets::EditText,
};

use rpg_core::{class::Class, uid::Uid, unit::HeroGameMode};
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
pub struct AccountCreateRoot;

#[derive(Component)]
pub struct AccountLoginRoot;

#[derive(Component)]
pub struct AccountListRoot;

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
pub struct CancelCreateButton;

#[derive(Component)]
pub struct CancelLoginButton;

#[derive(Component)]
pub struct AccountListCancelButton;

#[derive(Component)]
pub struct AccountListCreateGameButton;

#[derive(Component)]
pub struct AccountListContainer;

#[derive(Component, Deref, DerefMut)]
pub struct AccountListSelectedCharacterUid(pub Option<Uid>);

pub fn spawn_create(
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
            AccountCreateRoot,
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

pub fn spawn_login(
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
            AccountLoginRoot,
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
                            "Login to Account",
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

                            p.spawn(NodeBundle {
                                style: edit_style.clone(),
                                border_color: ui_theme.frame_border_color,
                                background_color: ui_theme.menu_background_color,
                                ..default()
                            })
                            .with_children(|p| {
                                p.spawn((
                                    AccountLoginName,
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
                                    AccountLoginPassword,
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
                });
            });

            p.spawn(NodeBundle {
                style: ui_theme.row_style.clone(),
                ..default()
            })
            .with_children(|p| {
                p.spawn((button.clone(), AccountLoginButton))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Login",
                            ui_theme.text_style_regular.clone(),
                        ));
                    });
                p.spawn((button.clone(), CancelLoginButton))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Cancel",
                            ui_theme.text_style_regular.clone(),
                        ));
                    });
            });
        });
}

pub fn spawn_list(
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
            AccountListRoot,
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
                        TextBundle::from_section("Accounts", ui_theme.text_style_regular.clone())
                            .with_style(ui_theme.row_style.clone()),
                    );
                });

                // TODO create a container to place accounts in

                p.spawn((
                    AccountListContainer,
                    NodeBundle {
                        style: ui_theme.row_style.clone(),
                        ..default()
                    },
                ));

                /*
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

                            p.spawn(NodeBundle {
                                style: edit_style.clone(),
                                border_color: ui_theme.frame_border_color,
                                background_color: ui_theme.menu_background_color,
                                ..default()
                            })
                            .with_children(|p| {
                                p.spawn((
                                    AccountLoginName,
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
                });*/
            });

            p.spawn(NodeBundle {
                style: ui_theme.row_style.clone(),
                ..default()
            })
            .with_children(|p| {
                p.spawn((
                    button.clone(),
                    AccountListSelectedCharacterUid(None),
                    AccountListCreateGameButton,
                ))
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Create Game",
                        ui_theme.text_style_regular.clone(),
                    ));
                });
                p.spawn((button.clone(), AccountListCancelButton))
                    .with_children(|p| {
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
    mut net_client: ResMut<Client>,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<AccountLoginButton>)>,
    mut account_text_set: ParamSet<(
        Query<&Text, With<AccountLoginName>>,
        Query<&Text, With<AccountLoginPassword>>,
    )>,
) {
    for interaction in &interaction_q {
        //
        if *interaction != Interaction::Pressed {
            continue;
        }

        let name = account_text_set.p0().single().sections[0].value.clone();
        let password = account_text_set.p1().single().sections[0].value.clone();

        let login_msg = CSLoadAccount { name, password };

        net_client.send_message::<Channel1, _>(login_msg);
    }
}

pub fn cancel_create_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelCreateButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<AccountCreateRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub fn cancel_login_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CancelLoginButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<AccountLoginRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub fn cancel_account_list_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<AccountListCancelButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub fn account_list_create_game_button(
    mut net_client: ResMut<Client>,
    interaction_q: Query<
        (&AccountListSelectedCharacterUid, &Interaction),
        (Changed<Interaction>, With<AccountListCreateGameButton>),
    >,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
    account_q: Query<&RpgAccount>,
) {
    let interaction = interaction_q.get_single();
    if let Ok((selected_character_uid, Interaction::Pressed)) = interaction {
        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::None;

        let Some(selected_character_uid) = selected_character_uid.0 else {
            return;
        };

        let account = account_q.single();
        let character = account
            .0
            .info
            .character_info
            .iter()
            .find(|c| c.uid == selected_character_uid)
            .unwrap();

        info!("sending create game request");
        net_client.send_message::<Channel1, _>(CSCreateGame(character.game_mode));
        return;
    }
}

/*
if _ {
    ui_image.texture = textures.icons["checkmark"].clone_weak();
} else {
    ui_image.texture = textures.icons["transparent"].clone_weak();
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
