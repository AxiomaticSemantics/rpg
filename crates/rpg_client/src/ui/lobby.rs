use crate::{
    assets::TextureAssets,
    net::lobby::Lobby,
    ui::menu::{
        account::{AccountListRoot, SelectedCharacter},
        main::MainRoot,
    },
};

use ui_util::{
    style::{UiRoot, UiTheme},
    widgets::EditText,
};

use rpg_chat::chat::ChannelId;
use rpg_network_protocol::protocol::*;

use bevy::{
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        entity::Entity,
        query::{Changed, With},
        system::{Commands, ParamSet, Query, Res, ResMut, Resource},
    },
    hierarchy::{BuildChildren, ChildBuilder, Children, DespawnRecursiveExt},
    log::info,
    prelude::{Deref, DerefMut},
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
pub(crate) struct LobbyMessageButton;

#[derive(Component)]
pub(crate) struct GameCreateButton;

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

                        p.spawn(TextBundle {
                            text: Text::from_section("Chat", ui_theme.text_style_regular.clone()),
                            ..default()
                        });

                        p.spawn(NodeBundle {
                            style: ui_theme.frame_col_style.clone(),
                            border_color: ui_theme.border_color,
                            background_color: ui_theme.background_color,
                            ..default()
                        })
                        .with_children(|p| {
                            let mut chat_style = ui_theme.frame_row_style.clone();
                            chat_style.width = Val::Px(400.);
                            chat_style.height = Val::Px(28.);

                            for i in 0..10 {
                                p.spawn(NodeBundle {
                                    style: chat_style.clone(),
                                    background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                                    ..default()
                                })
                                .with_children(|p| {
                                    p.spawn(TextBundle::from_section(
                                        "",
                                        ui_theme.text_style_regular.clone(),
                                    ));
                                });
                            }

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
                                        LobbyMessageText,
                                        Interaction::None,
                                        EditText::default(),
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
                                        Interaction::None,
                                        LobbyMessageButton,
                                        ImageBundle {
                                            image: UiImage {
                                                texture: textures.icons["checkmark"].clone_weak(),
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

                p.spawn((button.clone(), LeaveButton)).with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Leave Lobby",
                        ui_theme.text_style_regular.clone(),
                    ));
                });
            });
        });
}

pub(crate) fn lobby_message_button(
    lobby: Res<Lobby>,
    mut net_client: ResMut<Client>,
    button_q: Query<&Interaction, (Changed<Interaction>, With<LobbyMessageButton>)>,
    text_q: Query<&Text, With<LobbyMessageText>>,
) {
    let interaction = button_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        let Some(lobby) = &lobby.0 else {
            info!("not in a lobby");
            return;
        };

        let text = text_q.single();
        if text.sections[0].value.is_empty() {
            info!("no message to send");
            return;
        }
        net_client.send_message::<Channel1, _>(CSLobbyMessage {
            id: lobby.id,
            message: text.sections[0].value.clone(),
        });
    }
}

pub(crate) fn game_create_button(
    selected_character: Res<SelectedCharacter>,
    lobby: Res<Lobby>,
    mut net_client: ResMut<Client>,
    button_q: Query<&Interaction, (Changed<Interaction>, With<GameCreateButton>)>,
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

pub(crate) fn update_players_container(
    mut commands: Commands,
    ui_theme: Res<UiTheme>,
    mut lobby: ResMut<Lobby>,
    players_container_q: Query<(Entity, Option<&Children>), With<PlayersContainer>>,
) {
    if !lobby.is_changed() {
        return;
    }

    info!("lobby changed, updating players container");

    let (entity, children) = players_container_q.single();

    // TODO optimize thi on a rainy day
    // clear all children in the containers hierarchy
    if let Some(children) = children {
        for child in children.iter() {
            commands.entity(*child).despawn_recursive();
        }
    }

    // rebuild a new node hierarchy
    if let Some(lobby) = &lobby.0 {
        for account in lobby.accounts.iter() {
            let child = commands
                .spawn(NodeBundle {
                    style: ui_theme.col_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        format!("{}", account.0),
                        ui_theme.text_style_regular.clone(),
                    ));
                })
                .id();

            commands.entity(entity).push_children(&[child]);
        }
    }
}
