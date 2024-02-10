use crate::state::AppState;

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        event::{Event, EventReader},
        schedule::{common_conditions::in_state, IntoSystemConfigs},
        system::{ResMut, Resource},
    },
    log::info,
    math::uvec2,
};

use rpg_world::{
    zone::{Kind, SizeInfo, Zone, ZoneId},
    zone_path::ZonePath,
};

use std::collections::HashMap;

#[derive(Event)]
pub(crate) struct LoadZone(pub(crate) ZoneId);

pub(crate) enum ZoneLoadStatus {
    Unloaded,
    Loading,
    Loaded,
    Unloading,
}

pub(crate) struct ZoneState {
    pub(crate) load_status: ZoneLoadStatus,
}

pub(crate) struct RpgZone {
    pub(crate) zone: Option<Zone>,
    pub(crate) state: ZoneState,
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

pub(crate) fn spawn_world(mut rpg_world: ResMut<RpgWorld>, mut load_zone: EventReader<LoadZone>) {
    for load_zone_request in load_zone.read() {
        if rpg_world.zones.contains_key(&load_zone_request.0) {
            // ..
            info!("zone is already loaded");
            continue;
        }

        info!("loading zone {:?}", load_zone_request.0);

        let zone = Zone::new(
            load_zone_request.0,
            1234,
            SizeInfo::new(uvec2(8, 8), uvec2(8, 8), uvec2(4, 4)),
            Kind::OverworldTown,
            ZonePath::generate(),
        );

        let zone = RpgZone {
            zone: Some(zone),
            state: ZoneState {
                load_status: ZoneLoadStatus::Loading,
            },
        };
        rpg_world.zones.insert(load_zone_request.0, zone);
    }
}
