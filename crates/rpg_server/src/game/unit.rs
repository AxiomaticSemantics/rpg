use super::plugin::GameState;
use crate::{account::AccountInstance, assets::MetadataResources, net::server::NetworkParamsRW};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
    math::Vec3,
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
    transform::components::Transform,
};

use lightyear::shared::NetworkTarget;
use rpg_network_protocol::protocol::*;
use rpg_util::{
    item::{GroundItem, ResourceItem},
    unit::{Corpse, Hero, Unit},
};

#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct CorpseTimer(pub Timer);

pub(crate) fn upkeep(
    metadata: Res<MetadataResources>,
    mut net_params: NetworkParamsRW,
    time: Res<Time>,
    mut unit_q: Query<(&mut Unit, &AccountInstance), Without<Corpse>>,
) {
    for (mut unit, account) in &mut unit_q {
        let client = net_params
            .context
            .get_client_from_account_id(account.info.id)
            .unwrap();

        let updates = unit
            .0
            .stats
            .apply_regeneration(&metadata.0, time.delta_seconds());

        if !updates.is_empty() {
            info!("{updates:?}");

            net_params.server.send_message_to_target::<Channel1, _>(
                SCStatUpdates(updates),
                NetworkTarget::Only(vec![client.id]),
            );
        }
    }
}

pub(crate) fn attract_resource_items(
    mut commands: Commands,
    mut net_params: NetworkParamsRW,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    metadata: Res<MetadataResources>,
    mut item_q: Query<(Entity, &mut Transform, &mut GroundItem), With<ResourceItem>>,
    mut hero_q: Query<
        (Entity, &Transform, &mut Unit, &AccountInstance),
        (With<Hero>, Without<GroundItem>, Without<Corpse>),
    >,
) {
    // for each item, find the nearest hero in range and attract towards it
    for (i_entity, mut i_transform, mut i_item) in &mut item_q {
        let mut nearest = Entity::PLACEHOLDER;
        let max_distance = 8.;
        let mut nearest_distance = max_distance;

        for (u_entity, u_transform, unit, _) in &hero_q {
            let distance = u_transform.translation.distance(i_transform.translation);

            if distance < nearest_distance {
                nearest_distance = distance;
                nearest = u_entity;
            }
        }

        if nearest == Entity::PLACEHOLDER {
            info!("no hero nearby");
            continue;
        };

        let Ok((_, u_transform, mut unit, account)) = hero_q.get_mut(nearest) else {
            info!("hero query failed");
            continue;
        };

        let mut i_ground = i_transform.translation;
        i_ground.y = 0.;

        let pickup_radius = *unit.stats.vitals.stats["PickupRadius"].value.u32() as f32 / 100.;

        let distance = u_transform.translation.distance(i_ground);
        if distance > pickup_radius {
            continue;
        } else if distance < 0.25 {
            let item = i_item.0.take().unwrap();
            let _leveled_up = unit.apply_rewards(&metadata.0, &item);
            // TODO adjust unit statistics
            // TODO send rewards to client

            let client = net_params
                .context
                .get_client_from_account_id(account.0.info.id)
                .unwrap();

            net_params.server.send_message_to_target::<Channel1, _>(
                SCDespawnItem(item.uid.0),
                NetworkTarget::Only(vec![client.id]),
            );

            info!("hero attracted item");
            commands.entity(i_entity).despawn_recursive();
        } else {
            let target_dir = (u_transform.translation - i_ground).normalize_or_zero();
            i_transform.translation += target_dir * time.delta_seconds() * 4.;
        }
    }
}

pub(crate) fn remove_corpses(
    mut commands: Commands,
    time: Res<Time>,
    mut unit_q: Query<(Entity, &mut CorpseTimer), With<Unit>>,
) {
    for (entity, mut timer) in &mut unit_q {
        timer.tick(time.delta());
        if timer.just_finished() {
            // TODO tell the client to despawn the entity
            commands.entity(entity).despawn_recursive();
        }
    }
}
