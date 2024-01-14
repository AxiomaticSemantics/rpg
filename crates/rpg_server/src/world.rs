use crate::net::server::{ServerMode, ServerState};

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        event::{Event, EventReader},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::info,
    math::{uvec2, UVec2, Vec3},
    utils::default,
};

use rpg_world::zone::{Kind, SizeInfo, Zone, ZoneId};

use std::collections::{HashMap, VecDeque};

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

pub(crate) struct ServerWorldPlugin;

impl Plugin for ServerWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LoadZone>()
            .init_resource::<RpgWorld>()
            .add_systems(Update, spawn_world);
    }
}

fn spawn_world(
    mut rpg_world: ResMut<RpgWorld>,
    server_state: Res<ServerState>,
    mut load_zone: EventReader<LoadZone>,
) {
    if server_state.mode != ServerMode::Game {
        load_zone.clear();
        return;
    }

    for load_zone_request in load_zone.read() {
        if rpg_world.zones.contains_key(&load_zone_request.0) {
            // ..
            continue;
        }

        let zone = Zone::new(
            load_zone_request.0,
            1234,
            SizeInfo::new(uvec2(8, 8), uvec2(8, 8), uvec2(4, 4)),
            Kind::Overworld,
            VecDeque::new(),
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
