use rpg_world::zone::{Zone, ZoneId};

use bevy::ecs::{
    component::Component,
    event::Event,
    system::{ResMut, Resource},
};

use fastrand::Rng;

use std::collections::HashMap;

#[derive(Event)]
pub(crate) struct LoadZone(pub(crate) ZoneId);

#[derive(Component)]
pub struct Ground;

#[derive(Debug, Default, Resource)]
pub struct RpgWorld {
    pub zones: HashMap<ZoneId, Zone>,
    pub options: RpgWorldDebugOptions,
    pub active_zone: Option<ZoneId>,
    pub env_loaded: bool,
    pub rng: Rng,
}

#[derive(Debug, Default)]
pub struct RpgWorldDebugOptions {
    pub room_debug: bool,
    pub tile_debug: bool,
    pub tile_edge_debug: bool,
}

pub fn cleanup(mut rpg_world: ResMut<RpgWorld>) {
    rpg_world.zones.clear();
}
