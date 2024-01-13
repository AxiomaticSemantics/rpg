use crate::{
    assets::TextureAssets,
    menu::{
        account::{AccountCreateRoot, AccountLoginRoot},
        create::CreateRoot,
        credits::CreditsRoot,
        load::LoadRoot,
        settings::SettingsRoot,
    },
    state::AppState,
};

use ui_util::{
    style::UiTheme,
    widgets::{ImageButton, ImageButtonBundle},
};

use bevy::{
    ecs::{
        component::Component,
        query::{Changed, With},
        schedule::NextState,
        system::{ParamSet, Query, ResMut},
    },
    hierarchy::{BuildChildren, ChildBuilder},
    render::color::Color,
    ui::{
        node_bundles::{ButtonBundle, ImageBundle, NodeBundle, TextBundle},
        Display, Interaction, Style, UiImage, UiRect, Val,
    },
    utils::default,
};

#[derive(Component)]
pub struct MainRoot;

#[derive(Component)]
pub struct ExitButton;

#[derive(Component)]
pub struct AccountCreateButton;

#[derive(Component)]
pub struct AccountLoginButton;

#[derive(Component)]
pub struct SettingsButton;

#[derive(Component)]
pub struct CreditsButton;

pub fn spawn(
    textures: &TextureAssets,
    builder: &mut ChildBuilder,
    ui_theme: &UiTheme,
    button: &ButtonBundle,
    frame: &Style,
) {
    let frame_image = UiImage {
        texture: textures.icons["frame"].clone_weak(),
        ..default()
    };

    let vertical_spacer = NodeBundle {
        style: ui_theme.vertical_spacer.clone(),
        ..default()
    };

    builder
        .spawn((
            MainRoot,
            NodeBundle {
                style: frame.clone(),
                background_color: ui_theme.frame_background_color,
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: ui_theme.vertical_spacer.clone(),
                ..default()
            });

            p.spawn(ImageBundle {
                image: frame_image.clone(),
                background_color: Color::rgb(0.2, 0.05, 0.05).into(),
                ..default()
            })
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: Style {
                        margin: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(
                        TextBundle::from_section("Main Menu", ui_theme.text_style_regular.clone())
                            .with_style(ui_theme.row_style.clone()),
                    );
                });
            });

            p.spawn(NodeBundle {
                style: ui_theme.vertical_spacer.clone(),
                ..default()
            });

            p.spawn(NodeBundle {
                style: ui_theme.vertical_spacer.clone(),
                ..default()
            });

            p.spawn((
                AccountCreateButton,
                ImageButtonBundle {
                    marker: ImageButton,
                    image: ImageBundle {
                        image: frame_image.clone(),
                        background_color: Color::rgb(0.7, 0.0, 0.0).into(),
                        style: Style {
                            padding: UiRect::all(Val::Px(4.)),
                            ..default()
                        },
                        ..default()
                    },
                    interaction: Interaction::None,
                },
            ))
            .with_children(|p| {
                p.spawn(
                    TextBundle::from_section("Create Account", ui_theme.text_style_regular.clone())
                        .with_style(ui_theme.row_style.clone()),
                );
            });

            p.spawn(NodeBundle {
                style: ui_theme.vertical_spacer.clone(),
                ..default()
            });

            p.spawn((
                AccountLoginButton,
                ImageButtonBundle {
                    marker: ImageButton,
                    image: ImageBundle {
                        image: frame_image.clone(),
                        background_color: Color::rgb(0.7, 0.0, 0.0).into(),
                        style: Style {
                            padding: UiRect::all(Val::Px(4.)),
                            ..default()
                        },
                        ..default()
                    },
                    interaction: Interaction::None,
                },
            ))
            .with_children(|p| {
                p.spawn(
                    TextBundle::from_section("Account Login", ui_theme.text_style_regular.clone())
                        .with_style(ui_theme.row_style.clone()),
                );
            });

            p.spawn(NodeBundle {
                style: ui_theme.vertical_spacer.clone(),
                ..default()
            });

            p.spawn((
                SettingsButton,
                ImageButtonBundle {
                    marker: ImageButton,
                    image: ImageBundle {
                        image: frame_image.clone(),
                        background_color: Color::rgb(0.7, 0.0, 0.0).into(),
                        style: Style {
                            padding: UiRect::all(Val::Px(4.)),
                            ..default()
                        },
                        ..default()
                    },
                    interaction: Interaction::None,
                },
            ))
            .with_children(|p| {
                p.spawn(
                    TextBundle::from_section("Settings", ui_theme.text_style_regular.clone())
                        .with_style(ui_theme.row_style.clone()),
                );
            });

            p.spawn(NodeBundle {
                style: ui_theme.vertical_spacer.clone(),
                ..default()
            });

            p.spawn((
                CreditsButton,
                ImageButtonBundle {
                    marker: ImageButton,
                    image: ImageBundle {
                        image: frame_image.clone(),
                        background_color: Color::rgb(0.7, 0.0, 0.0).into(),
                        style: Style {
                            padding: UiRect::all(Val::Px(4.)),
                            ..default()
                        },
                        ..default()
                    },
                    interaction: Interaction::None,
                },
            ))
            .with_children(|p| {
                p.spawn(
                    TextBundle::from_section("Credits", ui_theme.text_style_regular.clone())
                        .with_style(ui_theme.row_style.clone()),
                );
            });

            p.spawn(NodeBundle {
                style: ui_theme.vertical_spacer.clone(),
                ..default()
            });

            p.spawn(NodeBundle {
                style: ui_theme.vertical_spacer.clone(),
                ..default()
            });

            p.spawn((
                ExitButton,
                ImageButtonBundle {
                    marker: ImageButton,
                    image: ImageBundle {
                        image: frame_image.clone(),
                        background_color: Color::rgb(0.7, 0.0, 0.0).into(),
                        style: Style {
                            padding: UiRect::all(Val::Px(4.)),
                            ..default()
                        },
                        ..default()
                    },
                    interaction: Interaction::None,
                },
            ))
            .with_children(|p| {
                p.spawn(
                    TextBundle::from_section("Exit", ui_theme.text_style_regular.clone())
                        .with_style(ui_theme.row_style.clone()),
                );
            });

            p.spawn(NodeBundle {
                style: ui_theme.vertical_spacer.clone(),
                ..default()
            });
        });
}

pub fn exit_button(
    mut state: ResMut<NextState<AppState>>,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<ExitButton>)>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        state.set(AppState::Shutdown);
    }
}

pub fn account_create_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<AccountCreateButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<AccountCreateRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::Flex;
    }
}

pub fn account_login_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<AccountLoginButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<AccountLoginRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::Flex;
    }
}

pub fn settings_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<SettingsButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<SettingsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::Flex;
    }
}

pub fn credits_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<CreditsButton>)>,
    mut menu_set: ParamSet<(
        Query<&mut Style, With<MainRoot>>,
        Query<&mut Style, With<CreditsRoot>>,
    )>,
) {
    let interaction = interaction_q.get_single();
    if let Ok(Interaction::Pressed) = interaction {
        menu_set.p0().single_mut().display = Display::None;
        menu_set.p1().single_mut().display = Display::Flex;
    }
}
