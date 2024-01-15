use crate::{
    assets::TextureAssets,
    net::lobby::Lobby,
    ui::menu::{
        account::{AccountListRoot, SelectedCharacterSlot},
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
        query::{Changed, With},
        system::{Commands, ParamSet, Query, Res, ResMut, Resource},
    },
    hierarchy::{BuildChildren, ChildBuilder, Children, DespawnRecursiveExt},
    log::info,
    prelude::{Deref, DerefMut},
    text::Text,
    ui::{
        node_bundles::{ButtonBundle, ImageBundle, NodeBundle, TextBundle},
        AlignItems, AlignSelf, Display, Interaction, JustifyContent, Style, UiImage, UiRect, Val,
    },
    utils::default,
};

#[derive(Component)]
pub struct LobbyRoot;

#[derive(Component)]
pub struct PlayersContainer;

#[derive(Component)]
pub struct GameCreateButton;

#[derive(Component)]
pub struct LeaveButton;

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
                            /*
                            .with_children(|p| {
                                p.spawn((TextBundle {
                                    text: Text::from_section(
                                        "player info",
                                        ui_theme.text_style_regular.clone(),
                                    ),
                                    style: Style {
                                        height: Val::Px(ui_theme.font_size_regular + 12.),
                                        width: Val::Px(128.0),
                                        ..default()
                                    },
                                    focus_policy: FocusPolicy::Pass,
                                    ..default()
                                },));
                            });*/
                        });

                        p.spawn(NodeBundle {
                            style: ui_theme.frame_row_style.clone(),
                            ..default()
                        })
                        .with_children(|p| {
                            p.spawn(TextBundle {
                                text: Text::from_section(
                                    "Send Message:",
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

                            p.spawn(NodeBundle {
                                style: button_style.clone(),
                                background_color: ui_theme.button_theme.normal_background_color,
                                border_color: ui_theme.border_color,
                                ..default()
                            })
                            .with_children(|p| {
                                p.spawn((
                                    Interaction::None,
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

            p.spawn(NodeBundle {
                style: ui_theme.row_style.clone(),
                ..default()
            })
            .with_children(|p| {
                p.spawn((button.clone(), LeaveButton)).with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Leave Lobby",
                        ui_theme.text_style_regular.clone(),
                    ));
                });
            });
        });
}

pub fn leave_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<LeaveButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<AccountListRoot>>,
        Query<&mut Style, With<LobbyRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::Flex;
        menu_set.p1().single_mut().display = Display::None;
    }
}

fn update_players_container(
    mut commands: Commands,
    mut lobby: ResMut<Lobby>,
    players_container_q: Query<Option<&Children>, With<PlayersContainer>>,
) {
    if !lobby.is_changed() {
        return;
    }

    info!("lobby changed, updating players container");

    let children = players_container_q.get_single();

    if let Ok(Some(children)) = children {
        for child in children.iter() {
            commands.entity(*child).despawn_recursive();
        }
    }

    /*
    for account in lobby.accounts {
       commands.spawn();
    }
    */
}
