use crate::{
    assets::TextureAssets,
    state::AppState,
    ui::menu::{
        account::{AccountCreateRoot, AccountLoginRoot},
        credits::CreditsRoot,
        settings::SettingsRoot,
    },
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
    sprite::{BorderRect, ImageScaleMode, SliceScaleMode, TextureSlicer},
    ui::{
        node_bundles::{AtlasImageBundle, ButtonBundle, ImageBundle, NodeBundle, TextBundle},
        AlignItems, AlignSelf, BackgroundColor, Display, Interaction, JustifyContent, Style,
        UiImage, UiRect, Val,
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
    let slicer = TextureSlicer {
        border: BorderRect::square(22.0),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    };

    let frame_image = UiImage {
        texture: textures.icons["border_patch1"].clone_weak(),
        ..default()
    };

    let slice_style = Style {
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::all(Val::Px(4.)),
        width: Val::Percent(100.),
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

            p.spawn(NodeBundle {
                style: Style {
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: Color::rgb(0.3, 0.25, 0.25).into(),
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
                    image: ButtonBundle {
                        image: frame_image.clone(),
                        background_color: Color::rgb(0.7, 0.0, 0.0).into(),
                        style: slice_style.clone(),
                        ..default()
                    },
                    ..default()
                },
                ImageScaleMode::Sliced(slicer.clone()),
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
                    image: ButtonBundle {
                        image: frame_image.clone(),
                        background_color: Color::rgb(0.7, 0.0, 0.0).into(),
                        style: slice_style.clone(),
                        ..default()
                    },
                    ..default()
                },
                ImageScaleMode::Sliced(slicer.clone()),
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
                    image: ButtonBundle {
                        image: frame_image.clone(),
                        background_color: Color::rgb(0.7, 0.0, 0.0).into(),
                        style: slice_style.clone(),
                        ..default()
                    },
                    ..default()
                },
                ImageScaleMode::Sliced(slicer.clone()),
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
                    image: ButtonBundle {
                        image: frame_image.clone(),
                        background_color: Color::rgb(0.7, 0.0, 0.0).into(),
                        style: slice_style.clone(),
                        ..default()
                    },
                    ..default()
                },
                ImageScaleMode::Sliced(slicer.clone()),
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
                    image: ButtonBundle {
                        image: frame_image.clone(),
                        background_color: Color::rgb(0.7, 0.0, 0.0).into(),
                        style: slice_style.clone(),
                        ..default()
                    },
                    ..default()
                },
                ImageScaleMode::Sliced(slicer.clone()),
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
