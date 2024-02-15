use crate::{
    account::AccountInstance, assets::MetadataResources, net::server::NetworkParamsRW,
    world::RpgWorld,
};

use rpg_network_protocol::protocol::*;
use rpg_util::unit::{Corpse, Unit};
use rpg_world::zone::{ZoneId, ZoneInfo};
use util::math::{intersect_aabb, AabbComponent};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
    math::{Quat, UVec2, Vec3},
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
    transform::components::Transform,
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
            .apply_regeneration(&metadata.rpg, time.delta_seconds());

        if !updates.is_empty() {
            // debug!("{updates:?}");

            let message =
                bincode::serialize(&ServerMessage::SCStatUpdates(SCStatUpdates(updates))).unwrap();
            net_params
                .server
                .send_message(client.client_id, ServerChannel::Message, message);
        }
    }
}

pub(crate) fn remove_corpses(
    mut commands: Commands,
    mut net_params: NetworkParamsRW,
    time: Res<Time>,
    mut unit_q: Query<(Entity, &Unit, &mut CorpseTimer), With<Corpse>>,
) {
    for (entity, unit, mut timer) in &mut unit_q {
        timer.tick(time.delta());
        if timer.just_finished() {
            let message =
                bincode::serialize(&ServerMessage::SCDespawnCorpse(SCDespawnCorpse(unit.uid)))
                    .unwrap();
            // TODO tell the client to despawn the entity
            net_params
                .server
                .broadcast_message(ServerChannel::Message, message);

            commands.entity(entity).despawn_recursive();
        }
    }
}

/// This is used to see if a unit can move without colliding
pub(crate) fn can_move(
    lhs: (&Vec3, &Quat, &AabbComponent),
    rhs: (&Vec3, &Quat, &AabbComponent),
) -> bool {
    !intersect_aabb((*lhs.0, *lhs.1, lhs.2 .0), (*rhs.0, *rhs.1, rhs.2 .0))
}

// TODO FIXME this is just a buggy hack
pub fn collide_units(
    mut unit_q: Query<(&mut Transform, &AabbComponent), (With<Unit>, Without<Corpse>)>,
) {
    let mut combinations = unit_q.iter_combinations_mut();
    while let Some([(mut t1, a1), (t2, a2)]) = combinations.fetch_next() {
        while intersect_aabb(
            (t1.translation, t1.rotation, a1.0),
            (t2.translation, t2.rotation, a2.0),
        ) {
            info!("units intersections, attempting to resolve");
            let distance = t1.translation.distance(t2.translation);
            let offset = 0.01 * *t1.forward();

            if (t1.translation + offset).distance(t2.translation) > distance {
                t1.translation += offset;
            } else {
                t1.translation -= offset;
            }
        }
    }
}
