use crate::{account::AccountInstance, assets::MetadataResources, net::server::NetworkParamsRW};

use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, Query, Res},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
    prelude::{Deref, DerefMut},
    time::{Time, Timer},
};

use lightyear::shared::NetworkTarget;
use rpg_network_protocol::protocol::*;
use rpg_util::unit::{Corpse, Unit};

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
