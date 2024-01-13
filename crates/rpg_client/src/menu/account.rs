use crate::{
    assets::TextureAssets,
    game::plugin::{GameState, PlayerOptions},
    menu::{create::CreateRoot, main::MainRoot},
    net::account::RpgAccount,
    state::AppState,
};

use ui_util::{
    style::{UiRoot, UiTheme},
    widgets::EditText,
};

use rpg_account::{
    account::{Account, AccountInfo},
    character::CharacterSlot,
};
use rpg_core::{class::Class, uid::Uid, unit::HeroGameMode};
use rpg_network_protocol::protocol::*;

use lightyear::prelude::*;

use bevy::{
    ecs::{
        change_detection::{DetectChanges, Ref},
        component::Component,
        entity::Entity,
        query::{Changed, With},
        schedule::NextState,
        system::{Commands, ParamSet, Query, Res, ResMut, Resource},
    },
    hierarchy::{BuildChildren, ChildBuilder, Children, DespawnRecursiveExt},
    log::*,
    prelude::{Deref, DerefMut},
    text::Text,
    ui::{
        node_bundles::{ButtonBundle, ImageBundle, NodeBundle, TextBundle},
        AlignItems, AlignSelf, BackgroundColor, Display, FocusPolicy, Interaction, JustifyContent,
        Style, UiImage, UiRect, Val,
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
pub struct CreateName;

#[derive(Component)]
pub struct CreateEmail;

#[derive(Component)]
pub struct CreatePassword;

#[derive(Component)]
pub struct LoginName;

#[derive(Component)]
pub struct LoginPassword;

#[derive(Component)]
pub struct CreateButton;

#[derive(Component)]
pub struct LoginButton;

#[derive(Component)]
pub struct CancelCreateButton;

#[derive(Component)]
pub struct CancelLoginButton;

#[derive(Component)]
pub struct ListCancelButton;

#[derive(Component)]
pub struct ListCreateGameButton;

#[derive(Component)]
pub struct ListJoinGameButton;

#[derive(Component)]
pub struct ListCreateCharacterButton;

#[derive(Component)]
pub struct ListContainer;

#[derive(Resource, Default, Deref, DerefMut)]
pub struct SelectedCharacterSlot(pub Option<CharacterSlot>);

#[derive(Debug, Component, Clone, Deref, DerefMut)]
pub struct AccountCharacterSlot(pub CharacterSlot);

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
                                    CreateName,
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
                                    CreateEmail,
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
                                    CreatePassword,
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
                p.spawn((button.clone(), CreateButton)).with_children(|p| {
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
                                    LoginName,
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
                                    LoginPassword,
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
                p.spawn((button.clone(), LoginButton)).with_children(|p| {
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

    let row_bundle = NodeBundle {
        style: ui_theme.row_style.clone(),
        ..default()
    };
    let col_bundle = NodeBundle {
        style: ui_theme.col_style.clone(),
        ..default()
    };

    let mut slot_style = ui_theme.frame_col_style.clone();
    slot_style.width = Val::Px(256.);
    slot_style.height = Val::Px(48.);

    let account_slot_node_bundle = NodeBundle {
        style: slot_style.clone(),
        background_color: ui_theme.button_theme.normal_background_color,
        ..default()
    };

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
                        TextBundle::from_section("Characters", ui_theme.text_style_regular.clone())
                            .with_style(ui_theme.row_style.clone()),
                    );
                });

                // TODO create a container to place accounts in

                p.spawn((ListContainer, col_bundle.clone()))
                    .with_children(|p| {
                        for row in 0..6 {
                            p.spawn(row_bundle.clone()).with_children(|p| {
                                for col in 0..2 {
                                    let slot = row * 2 + col;

                                    p.spawn(col_bundle.clone()).with_children(|p| {
                                        p.spawn((
                                            AccountCharacterSlot(CharacterSlot(slot)),
                                            Interaction::None,
                                            TextBundle::from_section(
                                                "Empty Slot",
                                                ui_theme.text_style_regular.clone(),
                                            )
                                            .with_style(slot_style.clone()),
                                        ));
                                    });
                                }
                            });
                        }
                    });
            });

            p.spawn(NodeBundle {
                style: ui_theme.row_style.clone(),
                ..default()
            })
            .with_children(|p| {
                p.spawn((button.clone(), ListCreateCharacterButton))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Create Character",
                            ui_theme.text_style_regular.clone(),
                        ));
                    });

                p.spawn((button.clone(), ListCreateGameButton))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Create Game",
                            ui_theme.text_style_regular.clone(),
                        ));
                    });
                p.spawn((button.clone(), ListCancelButton))
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
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CreateButton>)>,
    mut account_text_set: ParamSet<(
        Query<&Text, With<CreateName>>,
        Query<&Text, With<CreateEmail>>,
        Query<&Text, With<CreatePassword>>,
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
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<LoginButton>)>,
    mut account_text_set: ParamSet<(
        Query<&Text, With<LoginName>>,
        Query<&Text, With<LoginPassword>>,
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
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ListCancelButton>)>,
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

pub fn list_create_character_button(
    mut net_client: ResMut<Client>,
    selected_character_slot: Res<SelectedCharacterSlot>,
    mut style_set: ParamSet<(
        Query<&mut Style, With<CreateRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
        Query<(&mut Style, &Interaction), (Changed<Interaction>, With<ListCreateCharacterButton>)>,
    )>,
) {
    let mut interaction = style_set.p2();
    if let Ok((style, Interaction::Pressed)) = interaction.get_single_mut() {
        let Some(selected_character_slot) = selected_character_slot.0 else {
            info!("no character slot selected");
            return;
        };

        style_set.p0().single_mut().display = Display::Flex;
        style_set.p1().single_mut().display = Display::None;
    }
}

pub fn list_join_game_button(
    mut net_client: ResMut<Client>,
    selected_character_slot: Res<SelectedCharacterSlot>,
    mut style_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
        Query<(&Style, &Interaction), (Changed<Interaction>, With<ListJoinGameButton>)>,
    )>,
    account_q: Query<&RpgAccount>,
) {
    let mut interaction = style_set.p2();
    if let Ok((style, Interaction::Pressed)) = interaction.get_single_mut() {
        let Some(selected_character_slot) = selected_character_slot.0 else {
            info!("no character slot selected");
            return;
        };

        let account = account_q.single();
        let character_record = account
            .0
            .characters
            .iter()
            .find(|c| c.info.slot == selected_character_slot)
            .unwrap();

        net_client
            .send_message::<Channel1, _>(CSJoinGame {
                game_mode: character_record.info.game_mode,
                slot: selected_character_slot,
            })
            .unwrap();

        style_set.p0().single_mut().display = Display::None;
        style_set.p1().single_mut().display = Display::None;
    }
}

pub fn list_create_game_button(
    mut net_client: ResMut<Client>,
    selected_character_slot: Res<SelectedCharacterSlot>,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ListCreateGameButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<AccountListRoot>>,
    )>,
    account_q: Query<&RpgAccount>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        let Some(selected_character_slot) = selected_character_slot.0 else {
            info!("no character slot selected");
            return;
        };

        //menu_set.p0().single_mut().display = Display::None;
        //menu_set.p1().single_mut().display = Display::None;

        let account = account_q.single();
        let character = account
            .0
            .characters
            .iter()
            .find(|c| c.info.slot == selected_character_slot)
            .unwrap();

        info!("sending create game request");
        net_client
            .send_message::<Channel1, _>(CSCreateGame {
                game_mode: character.info.game_mode,
                slot: selected_character_slot,
            })
            .unwrap();
    }
}

pub fn list_cancel_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ListCancelButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<AccountListRoot>>,
        Query<&mut Style, With<AccountLoginRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::Flex;
    }
}

pub fn list_select_slot(
    ui_theme: Res<UiTheme>,
    mut selected_character_slot: ResMut<SelectedCharacterSlot>,
    mut slot_q: Query<(
        &mut Style,
        &mut BackgroundColor,
        Ref<Interaction>,
        &AccountCharacterSlot,
    )>,
) {
    for (mut style, mut bg_color, interaction, slot) in &mut slot_q {
        match *interaction {
            Interaction::Pressed => {
                if interaction.is_changed() && selected_character_slot.0 != Some(slot.0) {
                    info!("setting selected character slot to {slot:?}");
                    selected_character_slot.0 = Some(slot.0);
                }
            }
            Interaction::Hovered => {
                if bg_color.0 != ui_theme.button_theme.hovered_background_color.0 {
                    *bg_color = ui_theme.button_theme.hovered_background_color;
                }
            }
            Interaction::None => {
                if bg_color.0 != ui_theme.button_theme.normal_background_color.0 {
                    *bg_color = ui_theme.button_theme.normal_background_color;
                }
            }
        }

        if selected_character_slot.0 == Some(slot.0) {
            if bg_color.0 != ui_theme.button_theme.pressed_background_color.0 {
                *bg_color = ui_theme.button_theme.pressed_background_color;
            }
        }
    }
}

pub fn update_character_list(
    account_q: Query<&RpgAccount, Changed<RpgAccount>>,
    mut slot_q: Query<(&mut Text, &AccountCharacterSlot)>,
) {
    if let Ok(account) = account_q.get_single() {
        info!("account changed, updating character slots");

        for character in account.0.characters.iter() {
            info!(
                "character uid: {:?} slot: {:?}",
                character.info.uid, character.info.slot
            );

            for (mut text, slot) in &mut slot_q {
                if slot.0 == character.info.slot {
                    info!("slot match, updating");

                    let slot_string = format!(
                        "{} {:?} {}",
                        character.info.name,
                        character.info.game_mode,
                        character.character.unit.level
                    );

                    if text.sections[0].value != slot_string {
                        text.sections[0].value = slot_string;
                    }
                }
            }
        }
    }
}
