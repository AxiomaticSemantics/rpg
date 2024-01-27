use crate::assets::MetadataResources;

use rpg_core::{
    item::{Item, ItemKind},
    metadata::Metadata,
};
use rpg_util::{
    item::{GroundItem, GroundItemBundle, GroundItemDrops, ResourceItem, StorableItem},
    unit::Unit,
};

use util::{
    math::{Aabb, AabbComponent},
    random::{Rng, SharedRng},
};

use bevy::{
    ecs::{
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    math::Vec3,
    transform::components::Transform,
};

// FIXME audio should be attached to the item itself
pub(crate) fn spawn_ground_items(
    mut commands: Commands,
    metadata: Res<MetadataResources>,
    mut rng: ResMut<SharedRng>,
    mut ground_drop_items: ResMut<GroundItemDrops>,
    mut unit_q: Query<(Entity, &Transform, &Unit)>,
) {
    while let Some(items) = ground_drop_items.0.pop() {
        for (source, source_transform, source_unit) in &mut unit_q {
            if source_unit.uid != items.source {
                continue;
            }

            for item in &items.items {
                let item_info = &metadata.0.item.items[&item.id];

                spawn_item(
                    &mut commands,
                    &mut rng.0,
                    &metadata.0,
                    source_transform.translation,
                    item.clone(),
                );
            }
        }
    }
}

fn spawn_item(
    commands: &mut Commands,
    rng: &mut Rng,
    metadata: &Metadata,
    position: Vec3,
    item: Item,
) {
    // info!("spawning ground item at {position:?}");
    let item_info = &metadata.item.items[&item.id];

    let aabb = AabbComponent(Aabb::from_min_max(Vec3::splat(-0.2), Vec3::splat(0.2)));

    use std::f32::consts;

    let dir = consts::TAU * (0.5 - rng.f32());

    let mut transform = Transform::from_xyz(position.x, 0.8, position.z);
    transform.rotate_y(dir);

    let id = commands
        .spawn((
            // FIXME GameSessionCleanup,
            //CleanupStrategy::DespawnRecursive,
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
