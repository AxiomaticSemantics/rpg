use super::{
    assets::RenderResources, controls::CursorPosition, metadata::MetadataResources,
    plugin::GameSessionCleanup, prop::PropHandle,
};

use audio_manager::plugin::AudioActions;
use rpg_core::{
    item::{Item, ItemInfo, ItemKind},
    metadata::Metadata,
    uid::Uid,
};
use rpg_util::{
    item::{GroundItem, GroundItemDrops},
    unit::Unit,
};
use util::{cleanup::CleanupStrategy, math::AabbComponent, random::SharedRng};

use bevy::{
    ecs::{
        component::Component,
        query::With,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
    log::debug,
    math::Vec3,
    prelude::{default, Deref, DerefMut},
    scene::SceneBundle,
    text::Text,
    time::Time,
    transform::components::Transform,
    ui::{Display, Style},
};

use fastrand::Rng;

use std::borrow::Cow;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct CursorItem(pub Option<Uid>);

#[derive(Component)]
pub struct GroundItemHover;

#[derive(Component)]
pub struct GroundItemStats;

pub(crate) fn hover_ground_item(
    input: Res<ButtonInput<KeyCode>>,
    cursor_position: Res<CursorPosition>,
    metadata: Res<MetadataResources>,
    ground_item_q: Query<(&Transform, &GroundItem)>,
    mut ground_hover_q: Query<&mut Style, With<GroundItemHover>>,
    mut ground_item_ui_q: Query<&mut Text, With<GroundItemStats>>,
) {
    if !(input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)) {
        return;
    }

    let mut style = ground_hover_q.single_mut();
    for (transform, item) in &ground_item_q {
        let item = &item.0;

        let mut item_ground_pos = transform.translation;
        item_ground_pos.y = 0.;
        let distance = item_ground_pos.distance(cursor_position.ground);
        if distance < 0.25 {
            let mut text = ground_item_ui_q.single_mut();
            text.sections[0].value = make_item_stat_string(item, &metadata.rpg);

            if style.display != Display::Flex {
                style.display = Display::Flex;
            }

            // Only show the first item in range
            return;
        }
    }

    // No item is hovered
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

// FIXME audio should be attached to the item itself
pub(crate) fn spawn_ground_items(
    mut commands: Commands,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
    mut rng: ResMut<SharedRng>,
    mut ground_drop_items: ResMut<GroundItemDrops>,
    mut unit_q: Query<(&mut AudioActions, &Transform, &Unit)>,
) {
    while let Some(items) = ground_drop_items.0.pop() {
        for (mut source_audio, source_transform, source_unit) in &mut unit_q {
            if source_unit.uid != items.source {
                continue;
            }

            for item in &items.items {
                let item_info = &metadata.rpg.item.items[&item.id];
                match item_info.kind {
                    ItemKind::Gem => source_audio.push("item_drop_gem".into()),
                    ItemKind::Potion => source_audio.push("item_drop_potion".into()),
                    ItemKind::Currency => source_audio.push("item_drop_potion".into()),
                }

                spawn_item(
                    &mut commands,
                    &mut rng.0,
                    &renderables,
                    &metadata.rpg,
                    source_transform.translation,
                    item.clone(),
                );
            }
        }
    }
}

// TODO this needs to be driven by prop metadata
pub(crate) fn get_prop_key(metadata: &Metadata, item_info: &ItemInfo) -> Cow<'static, str> {
    match &item_info {
        ItemInfo::Gem(_) => Cow::Borrowed("item_gem"),
        ItemInfo::Potion(info) => {
            let (id_str, descriptor) = &metadata
                .stat
                .stats
                .iter()
                .find(|d| d.1.id == info.id)
                .unwrap();

            // debug!("id {id_str} {:?}", descriptor);
            match descriptor.id {
                _ if id_str == &"Hp" => Cow::Borrowed("item_potion_hp"),
                _ if id_str == &"Ep" => Cow::Borrowed("item_potion_ep"),
                _ if id_str == &"Mp" => Cow::Borrowed("item_potion_mp"),
                _ => unreachable!("Should not get here. {id_str}"),
            }
        }
        ItemInfo::Currency(_) => Cow::Borrowed("item_gem"),
    }
}

pub(crate) fn make_item_stat_string(item: &Item, metadata: &Metadata) -> String {
    let mut value = String::new();

    if let ItemInfo::Gem(info) = &item.info {
        if info.identified {
            for modifier in &info.modifiers {
                let modifier_meta = &metadata.modifier.modifiers[&modifier.modifier.id];
                value = format!("{}{modifier} to {}\n", value, modifier_meta.name);
            }
        } else {
            value = "Unidentified Gem\n".into();
        }
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
    let key = get_prop_key(metadata, &item.info);
    // debug!("key: {key} position: {position:?}");

    let PropHandle::Scene(handle) = &renderables.props[&*key].handle else {
        panic!("bad handle");
    };

    let aabb = AabbComponent(renderables.aabbs["item"]);

    use std::f32::consts;

    let dir = consts::TAU * (0.5 - rng.f32());

    let mut transform = Transform::from_xyz(position.x, 0.8, position.z);
    transform.rotate_y(dir);

    commands.spawn((
        GameSessionCleanup,
        CleanupStrategy::DespawnRecursive,
        SceneBundle {
            scene: handle.clone_weak(),
            transform,
            ..default()
        },
        GroundItem(item),
        aabb,
    ));
}
