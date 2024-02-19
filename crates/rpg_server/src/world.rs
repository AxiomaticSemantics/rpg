use crate::{assets::MetadataResources, net::server::NetworkParamsRW, state::AppState};

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        event::{Event, EventReader},
        schedule::{common_conditions::in_state, IntoSystemConfigs},
        system::{Res, ResMut, Resource},
    },
    log::info,
};

use rpg_network_protocol::protocol::*;
use rpg_world::zone::{Kind, Zone, ZoneId};

use std::collections::HashMap;

#[derive(Event)]
pub(crate) struct LoadZone(pub(crate) ZoneId);

#[derive(Default)]
pub(crate) enum ZoneLoadStatus {
    #[default]
    Unloaded,
    Loading,
    Loaded,
    Unloading,
}

pub(crate) struct RpgZone {
    pub(crate) zone: Option<Zone>,
    pub(crate) status: ZoneLoadStatus,
}

#[derive(Default, Resource)]
pub(crate) struct RpgWorld {
    pub(crate) zones: HashMap<ZoneId, RpgZone>,
}

pub(crate) struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadZone>()
            .init_resource::<RpgWorld>()
            .add_systems(
                Update,
                spawn_world.run_if(in_state(AppState::SpawnSimulation)),
            );
    }
}

pub(crate) fn spawn_world(
    mut rpg_world: ResMut<RpgWorld>,
    metadata: Res<MetadataResources>,
    mut load_zone: EventReader<LoadZone>,
    mut net_params: NetworkParamsRW,
) {
    for load_zone_request in load_zone.read() {
        let zone_id = load_zone_request.0;
        let message = bincode::serialize(&ServerMessage::SCZoneLoad(SCZoneLoad(zone_id))).unwrap();
        if rpg_world.zones.contains_key(&zone_id) {
            // ..
            info!("zone is already loaded");
            net_params
                .server
                .broadcast_message(ServerChannel::Message, message);
            continue;
        }

        let zone_meta = &metadata.world.zone.towns[&zone_id];

        let zone_id = load_zone_request.0;
        info!("loading zone {zone_id:?}");
        let zone = match zone_meta.kind {
            Kind::OverworldTown | Kind::UnderworldTown => {
                Zone::create_town(zone_id, 1234, &metadata.world)
            }
            _ => panic!("not now"),
        };

        let zone = RpgZone {
            zone: Some(zone),
            status: ZoneLoadStatus::Loading,
        };

        net_params
            .server
            .broadcast_message(ServerChannel::Message, message);

        rpg_world.zones.insert(zone_id, zone);
    }
}

pub(crate) fn cleanup(mut rpg_world: ResMut<RpgWorld>) {
    rpg_world.zones.clear();
}
