use super::plugin::{AabbResources, GameSessionCleanup};

use rpg_core::{item::Item, uid::Uid};
use rpg_util::{item::GroundItemDrops, unit::Unit};

use util::{cleanup::CleanupStrategy, math::AabbComponent};

use bevy::{
    ecs::{
        component::Component,
        system::{Commands, Query, Res, ResMut},
    },
    math::Vec3,
    transform::components::Transform,
};

#[derive(Component)]
pub(crate) struct GroundItem(pub(crate) Uid);

pub(crate) fn spawn_ground_items(
    mut commands: Commands,
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
                    &aabbs,
                    source_transform.translation,
                    item.clone(),
                );
            }
        }
    }
}

fn spawn_item(commands: &mut Commands, aabbs: &AabbResources, position: Vec3, item: Item) {
    // info!("spawning ground item at {position:?}");
    let aabb = AabbComponent(aabbs.aabbs["item_normal"]);

    let transform = Transform::from_xyz(position.x, 0.8, position.z);

    commands.spawn((
        GameSessionCleanup,
        CleanupStrategy::DespawnRecursive,
        transform,
        GroundItem(item.uid),
        aabb,
    ));
}
