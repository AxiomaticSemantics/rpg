use bevy::{
    app::{App, Plugin, Startup, Update},
    ecs::{
        event::{Event, EventReader},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::info,
    math::Vec3,
    utils::default,
};

use rpg_world::zone::{Zone, ZoneId};

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
    pub(crate) zone: Zone,
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

fn spawn_world(mut rpg_world: ResMut<RpgWorld>, mut load_zone: EventReader<LoadZone>) {
    for zone in load_zone.read() {
        if rpg_world.zones.contains_key(&zone.0) {}
    }
}
