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
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
    transform::components::Transform,
};

use lightyear::shared::replication::components::NetworkTarget;
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
                SCStatUpdates { updates },
                NetworkTarget::Only(vec![client.id]),
            );
        }
    }
}

pub(crate) fn collide_units() {
    //
}

// TODO factor out unit targetting code to a component
pub(crate) fn attract_resource_items(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    metadata: Res<MetadataResources>,
    mut item_q: Query<(Entity, &mut Transform, &mut GroundItem), With<ResourceItem>>,
    mut hero_q: Query<(&Transform, &mut Unit), (With<Hero>, Without<GroundItem>, Without<Corpse>)>,
) {
    let Ok((u_transform, mut unit)) = hero_q.get_single_mut() else {
        return;
    };

    for (i_entity, mut i_transform, mut i_item) in &mut item_q {
        let mut i_ground = i_transform.translation;
        i_ground.y = 0.;

        let pickup_radius = *unit.stats.vitals.stats["PickupRadius"].value.u32() as f32 / 100.;

        let distance = u_transform.translation.distance(i_ground);
        if distance > pickup_radius {
            continue;
        } else if distance < 0.25 {
            let item = i_item.0.take().unwrap();
            let _leveled_up = unit.apply_rewards(&metadata.0, &item);
            //u_audio.push("item_pickup".into());
            //game_state.stats.items_looted += 1;

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
