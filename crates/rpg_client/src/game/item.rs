use super::{
    actor::unit::Unit, assets::RenderResources, controls::CursorPosition,
    metadata::MetadataResources, plugin::GameSessionCleanup, prop::prop::PropHandle,
};
use crate::random::Random;

use audio_manager::plugin::AudioActions;
use rpg_core::{
    item::{Item, ItemInfo, ItemKind},
    metadata::Metadata,
    storage::{
        SlotIndex, StorageIndex, StorageSlot as RpgStorageSlot, UnitStorage as RpgUnitStorage,
    },
};
use util::cleanup::CleanupStrategy;

use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    hierarchy::Children,
    input::{mouse::MouseButton, ButtonInput},
    math::Vec3,
    prelude::{default, Deref, DerefMut},
    render::primitives::Aabb,
    scene::SceneBundle,
    text::Text,
    time::Time,
    transform::components::Transform,
    ui::{Display, Style},
};

use fastrand::Rng;

use std::borrow::Cow;

#[derive(Component, Copy, Clone, Deref, DerefMut, PartialEq, Eq, Debug)]
pub struct StorageSlot(pub RpgStorageSlot);

#[derive(Component, Deref, DerefMut)]
pub struct UnitStorage(pub RpgUnitStorage);

impl Default for UnitStorage {
    fn default() -> Self {
        Self(RpgUnitStorage::new())
    }
}

#[derive(Default, Resource)]
pub struct CursorItem {
    pub item: Option<StorageSlot>,
}

#[derive(Component, Deref, DerefMut)]
pub(crate) struct GroundItem(pub Option<Item>);

#[derive(Component)]
pub struct GroundItemHover;

#[derive(Component)]
pub struct GroundItemStats;

#[derive(Component)]
pub struct ResourceItem;

#[derive(Component)]
pub struct StorableItem;

pub(crate) struct GroundItemDrop {
    pub source: Entity,
    pub items: Vec<Item>,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct GroundItemDrops(pub Vec<GroundItemDrop>);

#[derive(Bundle)]
pub struct GroundItemBundle {
    pub prop: SceneBundle,
    pub item: GroundItem,
}

pub(crate) fn hover_ground_item(
    input: Res<ButtonInput<MouseButton>>,
    cursor_position: Res<CursorPosition>,
    metadata: Res<MetadataResources>,
    ground_item_q: Query<(&Transform, &GroundItem)>,
    mut ground_hover_q: Query<&mut Style, With<GroundItemHover>>,
    mut ground_item_ui_q: Query<&mut Text, With<GroundItemStats>>,
) {
    let mut style = ground_hover_q.single_mut();
    for (transform, item) in &ground_item_q {
        let item = item.as_ref().unwrap();

        let mut item_ground_pos = transform.translation;
        item_ground_pos.y = 0.;
        let distance = item_ground_pos.distance(cursor_position.ground);
        if distance < 0.25 {
            /* TODO decide if this is handled here or not
            input.just_pressed(MouseButton::Left) { // pick item }
            */

            let mut text = ground_item_ui_q.single_mut();
            text.sections[0].value = make_item_stat_string(item, &metadata.rpg);

            if style.display != Display::Flex {
                style.display = Display::Flex;
            }

            // Only show the first item in range
            return;
        }
    }

    if style.display != Display::None {
        style.display = Display::None;
    }
}

pub(crate) fn animate_ground_items(
    time: Res<Time>,
    mut ground_item_q: Query<&mut Transform, With<GroundItem>>,
) {
    let dt = time.delta_seconds();
    let d_y = dt.sin() * 0.2;

    for mut transform in &mut ground_item_q {
        transform.rotate_local_y(dt);
        transform.translation.y = 0.8 + d_y;
    }
}

pub(crate) fn spawn_ground_items(
    mut commands: Commands,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
    mut random: ResMut<Random>,
    mut ground_drop_items: ResMut<GroundItemDrops>,
    mut unit_q: Query<(&mut AudioActions, &Transform), With<Unit>>,
) {
    while let Some(item) = ground_drop_items.0.pop() {
        let (mut audio, transform) = unit_q.get_mut(item.source).unwrap();

        for item in item.items {
            let item_info = &metadata.rpg.item.items[&item.id];
            match item_info.kind {
                ItemKind::Gem => audio.push("item_drop_gem".into()),
                ItemKind::Resource => audio.push("item_drop_potion".into()),
            }

            spawn_item(
                &mut commands,
                &mut random.0,
                &renderables,
                &metadata.rpg,
                transform.translation,
                item,
            );
        }
    }
}

pub(crate) fn get_prop_key(metadata: &Metadata, item_info: &ItemInfo) -> Cow<'static, str> {
    match &item_info {
        ItemInfo::Gem(_) => Cow::Borrowed("fragment_xp"),
        ItemInfo::Resource(info) => {
            let (id_str, descriptor) = &metadata
                .stat
                .stats
                .iter()
                .find(|d| d.1.id == info.id)
                .unwrap();

            println!("id {id_str} {:?}", descriptor);
            match descriptor.id {
                _ if id_str == &"Hp" => Cow::Borrowed("potion_hp"),
                _ if id_str == &"Ep" => Cow::Borrowed("potion_ep"),
                _ if id_str == &"Mp" => Cow::Borrowed("potion_mp"),
                _ if id_str == &"Xp" => Cow::Borrowed("fragment_xp"), // FIXME need a mesh for this
                _ => unreachable!("Should not get here. {id_str}"),
            }
        }
    }
}

pub(crate) fn make_item_stat_string(item: &Item, metadata: &Metadata) -> String {
    let mut value = String::new();

    for modifier in &item.modifiers {
        let modifier_meta = &metadata.modifier.modifiers[&modifier.modifier.id];
        value = format!("{}{modifier} to {}\n", value, modifier_meta.name);
    }

    value
}

fn spawn_item(
    commands: &mut Commands,
    rng: &mut Rng,
    renderables: &RenderResources,
    metadata: &Metadata,
    position: Vec3,
    item: Item,
) {
    // println!("Spawning item at {position:?}");
    let item_info = &metadata.item.items[&item.id];

    let key = get_prop_key(metadata, &item_info.info);

    let PropHandle::Scene(handle) = &renderables.props[&*key].handle else {
        panic!("bad handle");
    };

    let aabb = Aabb::from_min_max(Vec3::splat(-0.2), Vec3::splat(0.2));

    use std::f32::consts;

    let dir = consts::TAU * (0.5 - rng.f32());

    let mut transform = Transform::from_xyz(position.x, 0.8, position.z);
    transform.rotate_y(dir);
    transform.translation += transform.forward() * 0.5;

    let id = commands
        .spawn((
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            GroundItemBundle {
                prop: SceneBundle {
                    scene: handle.clone_weak(),
                    transform,
                    ..default()
                },
                item: GroundItem(Some(item)),
            },
            aabb,
        ))
        .id();

    // Insert item kind marker
    match item_info.kind {
        ItemKind::Resource => {
            commands.entity(id).insert(ResourceItem);
        }
        _ => {
            commands.entity(id).insert(StorableItem);
        }
    }
}
