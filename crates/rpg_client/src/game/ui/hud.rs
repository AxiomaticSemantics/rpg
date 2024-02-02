#![allow(clippy::too_many_arguments)]

use crate::assets::TextureAssets;

use crate::game::{actor::player::Player, metadata::MetadataResources, plugin::GameSessionCleanup};
use rpg_util::unit::Unit;
use ui_util::style::UiTheme;
use util::cleanup::CleanupStrategy;

use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Commands, ParamSet, Query, Res},
    },
    hierarchy::BuildChildren,
    render::color::Color,
    text::prelude::*,
    ui::prelude::*,
    utils::prelude::default,
};

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ReturnToMenu;

#[derive(Component)]
pub struct Health;

#[derive(Component)]
pub struct Mana;

#[derive(Component)]
pub struct Stamina;

#[derive(Component)]
pub struct Xp;

#[derive(Component)]
pub(crate) struct HealthText;

#[derive(Component)]
pub(crate) struct ManaText;

#[derive(Component)]
pub(crate) struct StaminaText;

#[derive(Component)]
pub(crate) struct XpText;

#[derive(Component)]
pub(crate) struct PlayerName;

pub(crate) fn update(
    metadata: Res<MetadataResources>,
    player_q: Query<&Unit, With<Player>>,
    mut bar_set: ParamSet<(
        Query<&mut Style, With<Health>>,
        Query<&mut Style, With<Stamina>>,
        Query<&mut Style, With<Mana>>,
        Query<&mut Style, With<Xp>>,
    )>,
    mut text_set: ParamSet<(
        Query<&mut Text, With<HealthText>>,
        Query<&mut Text, With<StaminaText>>,
        Query<&mut Text, With<ManaText>>,
        Query<&mut Text, With<XpText>>,
    )>,
) {
    let Ok(unit) = player_q.get_single() else {
        return;
    };

    // TODO optimize to avoid change detection
    bar_set.p0().single_mut().width = Val::Percent(
        (100.0
            * (*unit.stats.vitals.stats["Hp"].value.u32() as f32
                / *unit.stats.vitals.stats["HpMax"].value.u32() as f32))
            .round(),
    );

    bar_set.p1().single_mut().width = Val::Percent(
        (100.0
            * (*unit.stats.vitals.stats["Ep"].value.u32() as f32
                / *unit.stats.vitals.stats["EpMax"].value.u32() as f32))
            .round(),
    );

    bar_set.p2().single_mut().width = Val::Percent(
        (100.0
            * (*unit.stats.vitals.stats["Mp"].value.u32() as f32
                / *unit.stats.vitals.stats["MpMax"].value.u32() as f32))
            .round(),
    );

    let hero_info = unit.info.hero();
    let level_info = &metadata.rpg.level.levels[&unit.level];

    let xp_total = level_info.xp_end - level_info.xp_begin;
    let xp_remaining = level_info.xp_end - *hero_info.xp_curr.value.u64();

    bar_set.p3().single_mut().width =
        Val::Percent((100. * (1.0 - xp_remaining as f64 / xp_total as f64) as f32).round());

    text_set.p0().single_mut().sections[0].value = format!(
        "HP {}/{}",
        unit.stats.vitals.stats["Hp"].value.u32(),
        unit.stats.vitals.stats["HpMax"].value.u32()
    );

    text_set.p1().single_mut().sections[0].value = format!(
        "EP {}/{}",
        unit.stats.vitals.stats["Ep"].value.u32(),
        unit.stats.vitals.stats["EpMax"].value.u32()
    );

    text_set.p2().single_mut().sections[0].value = format!(
        "MP {}/{}",
        unit.stats.vitals.stats["Mp"].value.u32(),
        unit.stats.vitals.stats["MpMax"].value.u32()
    );

    text_set.p3().single_mut().sections[0].value = format!(
        "Level {} ({}/{})",
        unit.level,
        hero_info.xp_curr.value.u64(),
        level_info.xp_end,
    );
}

pub(crate) fn setup(mut commands: Commands, ui_theme: Res<UiTheme>, _textures: Res<TextureAssets>) {
    let mut container_hidden_style = ui_theme.container_absolute_max.clone();
    container_hidden_style.display = Display::None;

    /*
    let mut slider_style = Style { ..default() };
    slider_style.height = Val::Px(24.);
    slider_style.width = Val::Px(96.);

    let mut slider_inner_style = slider_style.clone();
    slider_inner_style.height = Val::Px(18.);
    slider_inner_style.width = Val::Px(12.);
    slider_inner_style.left = Val::Px(0.);
    slider_inner_style.right = Val::Px(12.);
    */

    // HUD
    commands
        .spawn((
            HudRoot,
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            NodeBundle {
                style: ui_theme.container_absolute_max.clone(),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            })
            .with_children(|p| {
                // Blank fill
                p.spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    ..default()
                });

                let frame_color = ui_theme.frame_background_color.0.with_a(0.98);
                p.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_self: AlignSelf::Center,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Percent(100.),
                        border: UiRect::all(ui_theme.border),
                        padding: UiRect::all(ui_theme.padding),
                        margin: UiRect::all(ui_theme.margin),

                        min_width: Val::Px(720.),
                        ..default()
                    },
                    border_color: ui_theme.border_color,
                    background_color: frame_color.into(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            align_content: AlignContent::SpaceEvenly,
                            justify_content: JustifyContent::Center,
                            //width: Val::Percent(100.),
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|p| {
                        let bar_size = (Val::Px(128.), Val::Px(32.));

                        p.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(33.333333),
                                flex_direction: FlexDirection::Column,
                                margin: UiRect::all(ui_theme.margin),
                                padding: UiRect::all(ui_theme.padding),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|p| {
                            //ui_util::widgets::build_bar_label(p, &ui_theme, HealthText);
                            ui_util::widgets::build_horizontal_bar(
                                p,
                                &ui_theme,
                                Color::rgb_u8(0xfa, 0x50, 0x50),
                                HealthText,
                                Health,
                                bar_size.0,
                                bar_size.1,
                            );
                        });

                        p.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(33.333333),
                                flex_direction: FlexDirection::Column,
                                margin: UiRect::all(ui_theme.margin),
                                padding: UiRect::all(ui_theme.padding),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|p| {
                            //ui_util::widgets::build_bar_label(p, &ui_theme, StaminaText);
                            ui_util::widgets::build_horizontal_bar(
                                p,
                                &ui_theme,
                                Color::rgb_u8(0x50, 0xfa, 0x50),
                                StaminaText,
                                Stamina,
                                bar_size.0,
                                bar_size.1,
                            );
                        });

                        p.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(33.333333),
                                flex_direction: FlexDirection::Column,
                                margin: UiRect::all(ui_theme.margin),
                                padding: UiRect::all(ui_theme.padding),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|p| {
                            //ui_util::widgets::build_bar_label(p, &ui_theme, ManaText);
                            ui_util::widgets::build_horizontal_bar(
                                p,
                                &ui_theme,
                                Color::rgb_u8(0x50, 0x50, 0xfa),
                                ManaText,
                                Mana,
                                bar_size.0,
                                bar_size.1,
                            );
                        });
                    });

                    p.spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        // background_color: Color::GREEN.into(),
                        ..default()
                    })
                    .with_children(|p| {
                        p.spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.),
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|p| {
                            //ui_util::widgets::build_bar_label(p, &ui_theme, XpText);
                            ui_util::widgets::build_horizontal_bar(
                                p,
                                &ui_theme,
                                Color::rgb_u8(0xfa, 0x50, 0xfa),
                                XpText,
                                Xp,
                                Val::Px(512.),
                                Val::Px(32.),
                            );
                        });
                    });
                });
            });
        });
}
