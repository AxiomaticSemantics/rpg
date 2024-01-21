#![allow(clippy::too_many_arguments)]

use crate::assets::TextureAssets;

use crate::game::{
    actor::player::Player,
    assets::RenderResources,
    controls::Controls,
    item::{
        self, CursorItem, GroundItem, GroundItemBundle, GroundItemHover, GroundItemStats,
        StorableItem, StorageSlot, UnitStorage,
    },
    metadata::MetadataResources,
    plugin::{GameSessionCleanup, GameState},
    prop::PropHandle,
};

use rpg_core::{
    item::Rarity,
    storage::{
        self,
        inventory::{HERO_INVENTORY_COLUMNS, HERO_INVENTORY_ROWS},
        SlotIndex, Storage, StorageIndex, StorageSlot as RpgStorageSlot,
    },
};
use rpg_util::unit::Unit;

use ui_util::style::UiTheme;
use util::cleanup::CleanupStrategy;

use bevy::{
    ecs::{
        change_detection::DetectChanges,
        component::Component,
        query::{Changed, With},
        system::{Commands, ParamSet, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, ChildBuilder, Children},
    input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
    math::Vec3,
    render::{color::Color, primitives::Aabb},
    scene::SceneBundle,
    text::Text,
    transform::components::Transform,
    ui::{
        node_bundles::{ImageBundle, NodeBundle, TextBundle},
        AlignItems, AlignSelf, BackgroundColor, BorderColor, Display, FlexDirection, Interaction,
        JustifyContent, Style, UiImage, UiRect, Val, ZIndex,
    },
    utils::default,
};

#[derive(Component)]
pub(crate) struct InventoryRoot;

#[derive(Component)]
pub(crate) struct CursorItemPopup;

#[derive(Component)]
pub(crate) struct CursorItemStats;

#[derive(Component)]
pub(crate) struct ItemPopup;

#[derive(Component)]
pub(crate) struct ItemStats;

#[derive(Component)]
pub(crate) struct Health;

#[derive(Component)]
pub(crate) struct HealthText;

#[derive(Component)]
struct PlayerName;

const RARITY_COLOR_NORMAL: Color = Color::ALICE_BLUE;
const RARITY_COLOR_MAGIC: Color = Color::BLUE;
const RARITY_COLOR_RARE: Color = Color::YELLOW;
const RARITY_COLOR_LEGENDARY: Color = Color::ORANGE;
const RARITY_COLOR_UNIQUE: Color = Color::GOLD;

pub(crate) fn hover_storage(
    metadata: Res<MetadataResources>,
    mut cursor_item: ResMut<CursorItem>,
    mut item_node_q: Query<
        (
            &mut StorageSlot,
            &Interaction,
            &mut BorderColor,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
    mut storage_q: Query<(&mut Unit, &mut UnitStorage), With<Player>>,
    mut item_popup_q: Query<&mut Style, With<ItemPopup>>,
    mut text_q: Query<&mut Text, With<ItemStats>>,
) {
    let (mut unit, mut storage) = storage_q.single_mut();

    let mut popup_style = item_popup_q.single_mut();

    for (mut node, interaction, mut border, mut background) in &mut item_node_q {
        let slot = storage
            .slot_from_index(node.storage_index, node.slot_index)
            .unwrap();

        match *interaction {
            Interaction::Hovered => {
                let mut text = text_q.single_mut();

                if border.0 != Color::INDIGO {
                    border.0 = Color::INDIGO;
                }

                if let Some(item) = &slot.item {
                    text.sections[0].value = item::make_item_stat_string(item, &metadata.rpg);

                    if popup_style.display != Display::Flex {
                        popup_style.display = Display::Flex;
                    }
                } else {
                    if !text.sections[0].value.is_empty() {
                        text.sections[0].value.clear();
                    }

                    if popup_style.display != Display::None {
                        popup_style.display = Display::None;
                    }
                }

                continue;
            }
            Interaction::Pressed => {
                if border.0 != Color::AZURE {
                    border.0 = Color::AZURE;
                }

                if let Some(item) = cursor_item.item {
                    if storage.slot_has_item(node.0) {
                        if item.0 == node.0 {
                            cursor_item.item = None;
                        } else {
                            let item = cursor_item.item;

                            cursor_item.item = Some(StorageSlot(node.0));
                            storage.0.swap_slot(item.unwrap().0, node.0);
                            node.0 = item.unwrap().0;
                            border.0 = Color::DARK_GRAY;
                            background.0 = Color::GRAY;

                            unit.apply_item_stats(&metadata.rpg, &storage.0);
                        }
                    } else {
                        storage.0.swap_slot(item.0, node.0);
                        cursor_item.item = None;

                        unit.apply_item_stats(&metadata.rpg, &storage.0);
                    }
                } else if slot.item.is_some() {
                    cursor_item.item = Some(StorageSlot(RpgStorageSlot {
                        storage_index: node.storage_index,
                        slot_index: node.slot_index,
                    }));

                    background.0 = Color::RED;
                }

                continue;
            }
            _ => {}
        }

        if slot.item.is_some() && border.0 != Color::RED {
            border.0 = Color::RED;
        } else if border.0 != Color::DARK_GRAY {
            border.0 = Color::DARK_GRAY;
        }
    }
}

pub(crate) fn update_cursor_item(
    mut commands: Commands,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
    mut cursor_item: ResMut<CursorItem>,
    input: Res<ButtonInput<MouseButton>>,
    player_q: Query<&Transform, With<Player>>,
    mut cursor_ui_q: Query<&mut Style, With<CursorItemPopup>>,
    mut cursor_item_stats_q: Query<&Children, With<CursorItemStats>>,
    mut storage_q: Query<&mut UnitStorage>,
    mut text_q: Query<&mut Text>,
) {
    let player_transform = player_q.single();
    let mut storage = storage_q.single_mut();

    if cursor_item.item.is_some() {
        let item_slot = &cursor_item.item.as_ref().unwrap();
        let mut style = cursor_ui_q.single_mut();
        if input.just_pressed(MouseButton::Right) {
            // Drop the cursor item on the ground
            let aabb = Aabb::from_min_max(Vec3::splat(-0.2), Vec3::splat(0.2));
            let item_slot = storage
                .slot_from_index_mut(item_slot.storage_index, item_slot.slot_index)
                .unwrap();
            //println!("item_slot {item_slot:?}");
            let item = item_slot.item.take().unwrap();
            let item_meta = &metadata.rpg.item.items[&item.id];

            let key = item::get_prop_key(&metadata.rpg, &item_meta.info);

            let prop_handle = &renderables.props[&*key].handle;
            match prop_handle {
                PropHandle::Scene(handle) => {
                    commands.spawn((
                        GameSessionCleanup,
                        CleanupStrategy::DespawnRecursive,
                        StorableItem,
                        GroundItemBundle {
                            prop: SceneBundle {
                                scene: handle.clone_weak(),
                                transform: *player_transform,
                                ..default()
                            },
                            item: GroundItem(Some(item)),
                        },
                        aabb,
                    ));
                }
                PropHandle::Mesh(_) => {}
            }

            style.display = Display::None;
            cursor_item.item = None;
            return;
        }
    }

    if !cursor_item.is_changed() {
        return;
    }

    let mut style = cursor_ui_q.single_mut();
    if let Some(item) = &cursor_item.item {
        let children = cursor_item_stats_q.single_mut();
        let mut text = text_q.get_mut(*children.first().unwrap()).unwrap();

        let slot = storage
            .slot_from_index(item.storage_index, item.slot_index)
            .unwrap();

        if let Some(item) = &slot.item {
            text.sections[0].value = item::make_item_stat_string(item, &metadata.rpg);
        }

        style.top = Val::Px(-200.);
        style.left = Val::Px(-400.);

        style.display = Display::Flex;
    } else {
        style.display = Display::None;
    }
}

pub(crate) fn inventory_update(
    mut item_node_q: Query<(&StorageSlot, &mut BackgroundColor)>,
    storage_q: Query<&UnitStorage, With<Player>>,
) {
    let storage = storage_q.single();

    for (node, mut background) in &mut item_node_q {
        let Some(slot) = storage.slot_from_index(node.storage_index, node.slot_index) else {
            continue;
        };

        let Some(item) = &slot.item else {
            if background.0 != Color::GRAY {
                background.0 = Color::GRAY;
            }
            continue;
        };

        let rarity_color = match item.rarity {
            Rarity::Normal => RARITY_COLOR_NORMAL,
            Rarity::Magic => RARITY_COLOR_MAGIC,
            Rarity::Rare => RARITY_COLOR_RARE,
            Rarity::Legendary => RARITY_COLOR_LEGENDARY,
            Rarity::Unique => RARITY_COLOR_UNIQUE,
        };

        if background.0 != rarity_color {
            background.0 = rarity_color;
        }
    }
}

pub(crate) fn update(
    _metadata: Res<MetadataResources>,
    input: Res<ButtonInput<KeyCode>>,
    mut controls: ResMut<Controls>,
    _player_q: Query<&Unit, With<Player>>,
    mut style_set: ParamSet<(
        Query<&mut Style, With<InventoryRoot>>,
        Query<&mut Style, With<Health>>,
        Query<&mut Style, With<ItemPopup>>,
    )>,
) {
    if input.just_pressed(KeyCode::KeyI) {
        if let Display::None = style_set.p0().single().display {
            controls.set_inhibited(true);
            style_set.p0().single_mut().display = Display::Flex;
        } else {
            controls.set_inhibited(false);
            style_set.p0().single_mut().display = Display::None;
            style_set.p2().single_mut().display = Display::None;
        }
    }

    /*
    let unit = player_q.single();
    let hero_info = unit.info.hero();
    let level_info = &metadata.rpg.level.levels[&unit.level];

    text_set.p0().single_mut().sections[0].value = format!(
        "HP {}/{}",
        unit.stats.vitals.hp.value.u32(),
        unit.stats.vitals.hp_max.value.u32()
    );
    */
}

pub(crate) fn setup(
    mut commands: Commands,
    game_state: Res<GameState>,
    ui_theme: Res<UiTheme>,
    textures: Res<TextureAssets>,
) {
    println!("game::ui::inventory::setup");

    let vertical_spacing = NodeBundle {
        style: ui_theme.vertical_spacer.clone(),
        ..default()
    };

    let horizontal_spacing = NodeBundle {
        style: ui_theme.horizontal_spacer.clone(),
        ..default()
    };

    let mut hidden_container_style = ui_theme.container_absolute_max.clone();
    hidden_container_style.display = Display::None;

    commands
        .spawn((
            CleanupStrategy::DespawnRecursive,
            GameSessionCleanup,
            GroundItemHover,
            NodeBundle {
                style: hidden_container_style.clone(),
                z_index: ZIndex::Global(11),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: ui_theme.frame_col_style.clone(),
                border_color: ui_theme.border_color,
                background_color: ui_theme.background_color,
                ..default()
            })
            .with_children(|p| {
                p.spawn(TextBundle::from_section(
                    "Ground Item",
                    ui_theme.text_style_regular.clone(),
                ));

                p.spawn((
                    GroundItemStats,
                    TextBundle::from_section("", ui_theme.text_style_regular.clone()),
                ));
            });
        });

    commands
        .spawn((
            CleanupStrategy::DespawnRecursive,
            GameSessionCleanup,
            ItemPopup,
            NodeBundle {
                style: hidden_container_style.clone(),
                z_index: ZIndex::Global(11),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: ui_theme.frame_col_style.clone(),
                border_color: ui_theme.border_color,
                background_color: ui_theme.background_color,
                ..default()
            })
            .with_children(|p| {
                p.spawn(TextBundle::from_section(
                    "Item",
                    ui_theme.text_style_regular.clone(),
                ));

                p.spawn((
                    ItemStats,
                    TextBundle::from_section("", ui_theme.text_style_regular.clone()),
                ));
            });
        });

    commands
        .spawn((
            CleanupStrategy::DespawnRecursive,
            GameSessionCleanup,
            CursorItemPopup,
            NodeBundle {
                style: hidden_container_style.clone(),
                z_index: ZIndex::Global(11),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: ui_theme.frame_col_style.clone(),
                border_color: ui_theme.border_color,
                background_color: ui_theme.background_color,
                ..default()
            })
            .with_children(|p| {
                p.spawn(TextBundle::from_section(
                    "Cursor Item:",
                    ui_theme.text_style_regular.clone(),
                ));

                p.spawn((CursorItemStats, NodeBundle::default()))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            "",
                            ui_theme.text_style_regular.clone(),
                        ));
                    });
            });
        });

    let row_style = Style {
        flex_direction: FlexDirection::Row,
        //align_items: AlignItems::Center,
        //align_content: AlignContent::SpaceEvenly,
        justify_content: JustifyContent::Center,
        ..default()
    };

    let col_style = Style {
        flex_direction: FlexDirection::Column,
        ..default()
    };

    let frame_col_style = Style {
        flex_direction: FlexDirection::Column,
        //margin: UiRect::all(ui_theme.margin),
        //padding: UiRect::all(ui_theme.padding),
        border: UiRect::all(ui_theme.border),
        ..default()
    };

    let item_node_size = (Val::Px(36.), Val::Px(36.));
    let item_node_style = Style {
        width: item_node_size.0,
        height: item_node_size.1,
        flex_direction: FlexDirection::Row,
        margin: UiRect::all(Val::Px(1.)),
        border: UiRect::all(Val::Px(3.)),
        ..default()
    };

    let node_background = BackgroundColor(Color::rgb(0.4, 0.4, 0.4));
    let node_border = BorderColor(Color::DARK_GRAY);

    let item_node = NodeBundle {
        style: item_node_style,
        background_color: node_background,
        border_color: node_border,
        ..default()
    };

    let storage_header = NodeBundle {
        style: frame_col_style.clone(),
        //border_color: Color::rgb(0.2, 0.2, 0.2).into(),
        ..default()
    };

    let frame_image = UiImage {
        texture: textures.icons["frame"].clone_weak(),
        ..default()
    };

    // Inventory View
    commands
        .spawn((
            InventoryRoot,
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            NodeBundle {
                style: hidden_container_style.clone(),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_self: AlignSelf::FlexStart,
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(ui_theme.border),
                    padding: UiRect::all(ui_theme.padding * 2.),
                    margin: UiRect::all(ui_theme.margin * 4.),
                    ..default()
                },
                border_color: ui_theme.border_color,
                background_color: ui_theme.frame_background_color,
                ..default()
            })
            .with_children(|p| {
                p.spawn(NodeBundle {
                    style: col_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(TextBundle::from_section(
                        "Stash",
                        ui_theme.text_style_regular.clone(),
                    ));

                    p.spawn(horizontal_spacing.clone());

                    spawn_storage(
                        p,
                        frame_image.clone(),
                        storage_header.clone(),
                        item_node.clone(),
                        row_style.clone(),
                        col_style.clone(),
                        HERO_INVENTORY_ROWS * 2,
                        HERO_INVENTORY_COLUMNS,
                        StorageIndex(storage::STORAGE_STASH),
                    );
                });

                p.spawn(horizontal_spacing.clone());

                p.spawn(NodeBundle {
                    style: col_style.clone(),
                    ..default()
                })
                .with_children(|p| {
                    p.spawn(NodeBundle {
                        style: row_style.clone(),
                        ..default()
                    })
                    .with_children(|p| {
                        p.spawn(NodeBundle {
                            style: col_style.clone(),
                            ..default()
                        })
                        .with_children(|p| {
                            // left-arm
                            spawn_storage(
                                p,
                                frame_image.clone(),
                                storage_header.clone(),
                                item_node.clone(),
                                row_style.clone(),
                                col_style.clone(),
                                3,
                                2,
                                StorageIndex(storage::STORAGE_LEFT_ARM),
                            );

                            p.spawn(vertical_spacing.clone());

                            // gloves
                            spawn_storage(
                                p,
                                frame_image.clone(),
                                storage_header.clone(),
                                item_node.clone(),
                                row_style.clone(),
                                col_style.clone(),
                                2,
                                2,
                                StorageIndex(storage::STORAGE_GLOVES),
                            );
                        });

                        p.spawn(horizontal_spacing.clone());

                        p.spawn(NodeBundle {
                            style: col_style.clone(),
                            ..default()
                        })
                        .with_children(|p| {
                            // helmet
                            spawn_storage(
                                p,
                                frame_image.clone(),
                                storage_header.clone(),
                                item_node.clone(),
                                row_style.clone(),
                                col_style.clone(),
                                2,
                                2,
                                StorageIndex(storage::STORAGE_HELMET),
                            );

                            p.spawn(vertical_spacing.clone());

                            // body
                            spawn_storage(
                                p,
                                frame_image.clone(),
                                storage_header.clone(),
                                item_node.clone(),
                                row_style.clone(),
                                col_style.clone(),
                                3,
                                2,
                                StorageIndex(storage::STORAGE_BODY),
                            );

                            p.spawn(vertical_spacing.clone());

                            // belt
                            spawn_storage(
                                p,
                                frame_image.clone(),
                                storage_header.clone(),
                                item_node.clone(),
                                row_style.clone(),
                                col_style.clone(),
                                1,
                                2,
                                StorageIndex(storage::STORAGE_BELT),
                            );
                        });

                        p.spawn(horizontal_spacing.clone());

                        p.spawn(NodeBundle {
                            style: col_style.clone(),
                            ..default()
                        })
                        .with_children(|p| {
                            // right arm
                            spawn_storage(
                                p,
                                frame_image.clone(),
                                storage_header.clone(),
                                item_node.clone(),
                                row_style.clone(),
                                col_style.clone(),
                                3,
                                2,
                                StorageIndex(storage::STORAGE_RIGHT_ARM),
                            );

                            p.spawn(vertical_spacing.clone());

                            // boots
                            spawn_storage(
                                p,
                                frame_image.clone(),
                                storage_header.clone(),
                                item_node.clone(),
                                row_style.clone(),
                                col_style.clone(),
                                2,
                                2,
                                StorageIndex(storage::STORAGE_BOOTS),
                            );
                        });
                    });

                    let mut wide = vertical_spacing.clone();
                    wide.style.width *= 2.0;
                    p.spawn(wide);

                    p.spawn(NodeBundle {
                        style: row_style.clone(),
                        ..default()
                    })
                    .with_children(|p| {
                        spawn_storage(
                            p,
                            frame_image.clone(),
                            storage_header.clone(),
                            item_node.clone(),
                            row_style.clone(),
                            col_style.clone(),
                            HERO_INVENTORY_ROWS,
                            HERO_INVENTORY_COLUMNS,
                            StorageIndex(storage::STORAGE_INVENTORY),
                        );
                    });
                });
            });
        });
}

fn spawn_storage(
    commands: &mut ChildBuilder,
    image: UiImage,
    header: NodeBundle,
    item_node: NodeBundle,
    row_style: Style,
    col_style: Style,
    rows: usize,
    columns: usize,
    storage_index: StorageIndex,
) {
    commands
        .spawn(ImageBundle {
            image: image.clone(),
            style: col_style.clone(),
            ..default()
        })
        .with_children(|p| {
            p.spawn(header).with_children(|p| {
                for row in 0..rows {
                    p.spawn(NodeBundle {
                        style: row_style.clone(),
                        ..default()
                    })
                    .with_children(|p| {
                        for column in 0..columns {
                            let index = (column + (columns * row)) as u32;

                            let storage_slot = StorageSlot(RpgStorageSlot {
                                storage_index,
                                slot_index: SlotIndex(index),
                            });

                            //println!("spawning {storage_slot:?}");

                            p.spawn(NodeBundle {
                                style: col_style.clone(),
                                ..default()
                            })
                            .with_children(|p| {
                                p.spawn((Interaction::None, storage_slot, item_node.clone()));
                            });
                        }
                    });
                }
            });
        });
}
