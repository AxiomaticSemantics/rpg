use crate::{account::AccountInstance, assets::MetadataResources, net::server::NetworkParamsRW};

use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::Actions,
    unit::{Corpse, Unit},
};
use util::math::{intersect_aabb, AabbComponent};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{QueryIter, With, Without},
        system::{Commands, Query, Res},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
    math::{Quat, Vec3},
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
    transform::components::Transform,
};

use lightyear::shared::NetworkTarget;

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
            // debug!("{updates:?}");

            net_params.server.send_message_to_target::<Channel1, _>(
                SCStatUpdates(updates),
                NetworkTarget::Only(vec![client.id]),
            );
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
            // TODO tell the client to despawn the entity
            net_params.server.send_message_to_target::<Channel1, _>(
                SCDespawnCorpse(unit.uid),
                NetworkTarget::All,
            );

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
    /*let mut combinations = unit_q.iter_combinations_mut();
    while let Some([(mut t1, a1), (t2, a2)]) = combinations.fetch_next() {
        while intersect_aabb(
            (t1.translation, t1.rotation, a1.0),
            (t2.translation, t2.rotation, a2.0),
        ) {
            let distance = t1.translation.distance(t2.translation);
            let offset = 0.01 * *t1.forward();

            if (t1.translation + offset).distance(t2.translation) > distance {
                t1.translation += offset;
            } else {
                t1.translation -= offset;
            }
        }
    }*/
}
