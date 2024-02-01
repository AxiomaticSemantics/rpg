use super::plugin::{AabbResources, GameSessionCleanup};
use crate::assets::MetadataResources;

use rpg_core::{
    item::{Item, ItemKind},
    metadata::Metadata,
};
use rpg_util::{
    item::{GroundItem, GroundItemBundle, GroundItemDrops, ResourceItem, StorableItem},
    unit::Unit,
};

use util::{cleanup::CleanupStrategy, math::AabbComponent};

use bevy::{
    ecs::system::{Commands, Query, Res, ResMut},
    math::Vec3,
    transform::components::Transform,
};

pub(crate) fn spawn_ground_items(
    mut commands: Commands,
    metadata: Res<MetadataResources>,
    aabbs: Res<AabbResources>,
    mut ground_drop_items: ResMut<GroundItemDrops>,
    mut unit_q: Query<(&Transform, &Unit)>,
) {
    while let Some(items) = ground_drop_items.0.pop() {
        for (source_transform, source_unit) in &mut unit_q {
            if source_unit.uid != items.source {
                continue;
            }

            for item in &items.items {
                spawn_item(
                    &mut commands,
                    &metadata.0,
                    &aabbs,
                    source_transform.translation,
                    item.clone(),
                );
            }
        }
    }
}

fn spawn_item(
    commands: &mut Commands,
    metadata: &Metadata,
    aabbs: &AabbResources,
    position: Vec3,
    item: Item,
) {
    // info!("spawning ground item at {position:?}");
    let item_info = &metadata.item.items[&item.id];

    let aabb = AabbComponent(aabbs.aabbs["item_normal"]);

    let transform = Transform::from_xyz(position.x, 0.8, position.z);

    let id = commands
        .spawn((
            GameSessionCleanup,
            CleanupStrategy::DespawnRecursive,
            transform,
            GroundItemBundle {
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
