#![allow(clippy::too_many_arguments)]

use crate::assets::TextureAssets;

use crate::game::{
    actor::player::Player,
    metadata::MetadataResources,
    plugin::{GameSessionCleanup, GameState},
};
use rpg_util::unit::Unit;
use ui_util::style::UiTheme;
use util::cleanup::CleanupStrategy;

use bevy::{
    ecs::prelude::*, hierarchy::BuildChildren, render::prelude::*, text::prelude::*,
    ui::prelude::*, utils::prelude::*,
};

#[derive(Component)]
pub(crate) struct HeroRoot;

#[derive(Component)]
pub(crate) struct Health;

#[derive(Component)]
pub(crate) struct Mana;

#[derive(Component)]
pub(crate) struct Stamina;

#[derive(Component)]
pub(crate) struct Xp;

#[derive(Component)]
pub(crate) struct HealthText;

#[derive(Component)]
pub(crate) struct ManaText;

#[derive(Component)]
pub(crate) struct StaminaText;

#[derive(Component)]
pub(crate) struct XpText;

#[derive(Component)]
struct PlayerName;

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
    let unit = player_q.single();

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

pub(crate) fn setup(
    mut commands: Commands,
    game_state: Res<GameState>,
    ui_theme: Res<UiTheme>,
    _textures: Res<TextureAssets>,
) {
    println!("game::ui::hero::setup");

    let mut container_hidden_style = ui_theme.container_absolute_max.clone();
    container_hidden_style.display = Display::None;

    /*
    let vertical_spacing = NodeBundle {
        style: ui_theme.vertical_spacer.clone(),
        ..default()
    };

    let large_text_style = TextStyle {
        font: ui_theme.font.clone(),
        font_size: ui_theme.font_size_large,
        color: ui_theme.text_color_dark,
    };

    let normal_text_style = TextStyle {
        font: ui_theme.font.clone(),
        font_size: ui_theme.font_size_regular,
        color: ui_theme.text_color_dark,
    };

    let row_node = NodeBundle {
        style: ui_theme.row_style.clone(),
        ..default()
    };

    let col_node = NodeBundle {
        style: ui_theme.col_style.clone(),
        ..default()
    };

    let frame_col_node = NodeBundle {
        style: ui_theme.frame_col_style.clone(),
        border_color: ui_theme.border_color,
        background_color: ui_theme.frame_background_color.0.with_a(0.98).into(),
        ..default()
    };

    let frame_row_node = NodeBundle {
        style: ui_theme.frame_row_style.clone(),
        background_color: ui_theme.menu_background_color,
        ..default()
    };*/

    // Hero View
    commands
        .spawn((
            HeroRoot,
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            NodeBundle {
                style: container_hidden_style.clone(),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    //align_items: AlignItems::Center,
                    //justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|p| {
                let frame_color = ui_theme.frame_background_color.0.with_a(0.9);
                p.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_self: AlignSelf::Center,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Percent(100.),
                        border: UiRect::all(ui_theme.border),
                        margin: UiRect::all(ui_theme.margin),
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
                                Health,
                                HealthText,
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
                                Stamina,
                                StaminaText,
                                bar_size.0,
                                bar_size.1,
                            );
                        });
                    });
                });
            });
        });
}
