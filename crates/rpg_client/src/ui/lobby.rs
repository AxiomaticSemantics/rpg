use crate::{
    assets::TextureAssets,
    net::{account::RpgAccount, lobby::Lobby},
    ui::menu::account::{AccountListRoot, SelectedCharacter},
};

use ui_util::{
    style::{UiRoot, UiTheme},
    widgets::{EditText, FocusedElement, NewlineBehaviour},
};

use rpg_network_protocol::protocol::*;

use bevy::{
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        entity::Entity,
        query::{Changed, With},
        system::{Commands, ParamSet, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, ChildBuilder, Children, DespawnRecursiveExt},
    input::{keyboard::KeyCode, ButtonInput},
    log::info,
    render::color::Color,
    text::Text,
    ui::{
        node_bundles::{ButtonBundle, ImageBundle, NodeBundle, TextBundle},
        AlignItems, AlignSelf, Display, Interaction, JustifyContent, Style, UiImage, UiRect, Val,
    },
    utils::default,
};

#[derive(Component)]
pub(crate) struct LobbyRoot;

#[derive(Component)]
pub(crate) struct PlayersContainer;

#[derive(Component)]
pub(crate) struct LobbyMessageText;

#[derive(Component)]
pub(crate) struct LobbySendMessageText;

#[derive(Component)]
pub(crate) struct LobbyMessages;

#[derive(Component)]
pub(crate) struct LobbyMessageButton;

#[derive(Component)]
pub(crate) struct GameCreateButton;

#[derive(Component)]
pub(crate) struct GameJoinButton;

#[derive(Component)]
pub(crate) struct LeaveButton;

pub(crate) fn spawn(
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
            LobbyRoot,
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
                        TextBundle::from_section("Lobby", ui_theme.text_style_regular.clone())
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
                                "Players:",
                                ui_theme.text_style_regular.clone(),
                            ),));

                            //let mut edit_style = ui_theme.frame_row_style.clone();
                            //edit_style.border = UiRect::all(ui_theme.border);

                            p.spawn((
                                PlayersContainer,
                                NodeBundle {
                                    style: ui_theme.frame_row_style.clone(),
                                    border_color: ui_theme.frame_border_color,
                                    background_color: ui_theme.menu_background_color,
                                    ..default()
                                },
                            ));
                        });
                    });

                    p.spawn(NodeBundle {
                        style: ui_theme.row_style.clone(),
                        ..default()
                    })
                    .with_children(|p| {
                        p.spawn(TextBundle {
                            text: Text::from_section("Chat", ui_theme.text_style_regular.clone()),
                            ..default()
                        });

                        p.spawn((NodeBundle {
                            style: ui_theme.frame_col_style.clone(),
                            border_color: ui_theme.border_color,
                            background_color: ui_theme.background_color,
                            ..default()
                        },))
                            .with_children(|p| {
                                let mut chat_style = ui_theme.frame_col_style.clone();
                                chat_style.width = Val::Px(400.);
                                chat_style.height = Val::Px(24.);

                                // Chat History
                                p.spawn((
                                    LobbyMessages,
                                    NodeBundle {
                                        style: ui_theme.frame_col_style.clone(),
                                        background_color: Color::rgb(0.1, 0.1, 0.1).into(),
                                        ..default()
                                    },
                                ))
                                .with_children(|p| {
                                    for _ in 0..10 {
                                        p.spawn((
                                            LobbyMessageText,
                                            TextBundle::from_section(
                                                "",
                                                ui_theme.text_style_regular.clone(),
                                            )
                                            .with_style(chat_style.clone()),
                                        ));
                                    }
                                });

                                p.spawn(NodeBundle {
                                    style: ui_theme.frame_row_style.clone(),
                                    ..default()
                                })
                                .with_children(|p| {
                                    p.spawn(TextBundle {
                                        text: Text::from_section(
                                            "Message",
                                            ui_theme.text_style_regular.clone(),
                                        ),
                                        ..default()
                                    });

                                    p.spawn(NodeBundle {
                                        style: chat_style.clone(),
                                        background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                                        ..default()
                                    })
                                    .with_children(|p| {
                                        p.spawn((
                                            LobbySendMessageText,
                                            Interaction::None,
                                            EditText::new(NewlineBehaviour::Consume),
                                            TextBundle {
                                                text: Text::from_section(
                                                    "",
                                                    ui_theme.text_style_regular.clone(),
                                                ),
                                                style: chat_style.clone(),
                                                ..default()
                                            },
                                        ));
                                    });

                                    let button_style = Style {
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
                                        background_color: ui_theme
                                            .button_theme
                                            .normal_background_color,
                                        border_color: ui_theme.border_color,
                                        ..default()
                                    })
                                    .with_children(|p| {
                                        p.spawn((
                                            Interaction::None,
                                            LobbyMessageButton,
                                            ImageBundle {
                                                image: UiImage {
                                                    texture: textures.icons["checkmark"]
                                                        .clone_weak(),
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
                p.spawn((button.clone(), GameCreateButton))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Create Game",
                            ui_theme.text_style_regular.clone(),
                        ));
                    });

                p.spawn((button.clone(), GameJoinButton))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "Join Game",
                            ui_theme.text_style_regular.clone(),
                        ));
                    });

                p.spawn((button.clone(), LeaveButton)).with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Leave Lobby",
                        ui_theme.text_style_regular.clone(),
                    ));
                });
            });
        });
}

pub(crate) fn lobby_send_message(
    lobby: Res<Lobby>,
    focused: Res<FocusedElement>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut net_client: ResMut<Client>,
    button_q: Query<&Interaction, (Changed<Interaction>, With<LobbyMessageButton>)>,
    mut text_q: Query<(Entity, &mut Text), With<LobbySendMessageText>>,
) {
    let interaction = button_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        let Some(lobby) = &lobby.0 else {
            info!("not in a lobby");
            return;
        };

        let (_, mut text) = text_q.single_mut();
        if text.sections[0].value.is_empty() {
            info!("no message to send");
            return;
        }
        net_client.send_message::<Channel1, _>(CSLobbyMessage {
            id: lobby.id,
            message: text.sections[0].value.clone(),
        });

        text.sections[0].value.clear();
    } else if key_input.just_pressed(KeyCode::Enter) {
        let Some(lobby) = &lobby.0 else {
            info!("not in a lobby");
            return;
        };

        let Some(focused) = focused.0 else {
            return;
        };

        let (entity, mut text) = text_q.single_mut();
        if focused != entity {
            return;
        }

        if text.sections[0].value.is_empty() {
            info!("no message to send");
            return;
        }
        net_client.send_message::<Channel1, _>(CSLobbyMessage {
            id: lobby.id,
            message: text.sections[0].value.clone(),
        });

        text.sections[0].value.clear();
    }
}

pub(crate) fn game_create_button(
    selected_character: Res<SelectedCharacter>,
    lobby: Res<Lobby>,
    mut net_client: ResMut<Client>,
    mut account_q: Query<&mut RpgAccount>,
    button_q: Query<&Interaction, (Changed<Interaction>, With<GameCreateButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<UiRoot>>,
        Query<&mut Style, With<LobbyRoot>>,
    )>,
) {
    let interaction = button_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        let Some(lobby) = &lobby.0 else {
            info!("not in a lobby");
            return;
        };

        let Some(slot_character) = &selected_character.0 else {
            info!("no character selected");
            return;
        };

        net_client.send_message::<Channel1, _>(CSCreateGame {
            game_mode: lobby.game_mode,
            slot: slot_character.slot,
        });

        // FIXME temp hack
        account_q.single_mut().0.info.selected_slot = Some(slot_character.slot);

        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub(crate) fn game_join_button(
    selected_character: Res<SelectedCharacter>,
    lobby: Res<Lobby>,
    mut net_client: ResMut<Client>,
    mut account_q: Query<&mut RpgAccount>,
    button_q: Query<&Interaction, (Changed<Interaction>, With<GameJoinButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<UiRoot>>,
        Query<&mut Style, With<LobbyRoot>>,
    )>,
) {
    let interaction = button_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        let Some(lobby) = &lobby.0 else {
            info!("not in a lobby");
            return;
        };

        let Some(slot_character) = &selected_character.0 else {
            info!("no character selected");
            return;
        };

        net_client.send_message::<Channel1, _>(CSJoinGame {
            game_mode: lobby.game_mode,
            slot: slot_character.slot,
        });

        // FIXME temp hack
        account_q.single_mut().0.info.selected_slot = Some(slot_character.slot);

        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub(crate) fn leave_button(
    mut net_client: ResMut<Client>,
    button_q: Query<&Interaction, (Changed<Interaction>, With<LeaveButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<AccountListRoot>>,
        Query<&mut Style, With<LobbyRoot>>,
    )>,
) {
    let interaction = button_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        net_client.send_message::<Channel1, _>(CSLobbyLeave);

        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

pub(crate) fn update_lobby_messages(
    mut lobby: ResMut<Lobby>,
    lobby_messages_q: Query<&Children, With<LobbyMessages>>,
    mut message_item_q: Query<&mut Text, With<LobbyMessageText>>,
) {
    if !lobby.is_changed() {
        return;
    }

    let Some(lobby) = &mut lobby.0 else {
        info!("lobby is not populated");
        return;
    };

    let len = lobby.messages.len();
    info!("updating lobby messages {}", len);

    let mut count = len.saturating_sub(10);
    let children = lobby_messages_q.single();

    for child in children.iter() {
        let message_text = &mut message_item_q.get_mut(*child).unwrap();
        let Some(msg) = lobby.messages.get(count) else {
            continue;
        };

        let message = format!("{}: {}", msg.sender, msg.message);
        if message_text.sections[0].value != message {
            message_text.sections[0].value = message;
        }

        count += 1;
    }
}

pub(crate) fn update_players_container(
    mut commands: Commands,
    ui_theme: Res<UiTheme>,
    lobby: Res<Lobby>,
    players_container_q: Query<(Entity, Option<&Children>), With<PlayersContainer>>,
) {
    if !lobby.is_changed() {
        return;
    }

    info!("lobby changed, updating players container");

    let (entity, children) = players_container_q.single();

    // TODO optimize this on a rainy day

    // clear all children in the containers hierarchy
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(*child).despawn_recursive();
        }
    }

    // rebuild a new node hierarchy
    if let Some(lobby) = &lobby.0 {
        for player in lobby.players.iter() {
            let child = commands
                .spawn(NodeBundle {
                    style: ui_theme.col_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        player.account_name.clone(),
                        ui_theme.text_style_regular.clone(),
                    ));
                })
                .id();

            commands.entity(entity).push_children(&[child]);
        }
    }
}
