use crate::Transports;

use bevy::{
    app::{
        App, FixedUpdate, Plugin, PluginGroup, PreUpdate, ScheduleRunnerPlugin, Startup, Update,
    },
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::IntoSystemConfigs,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    hierarchy::DespawnRecursiveExt,
    log::info,
    math::Vec3,
    transform::{components::Transform, TransformBundle},
    utils::default,
};

use lightyear::prelude::FixedUpdateSet;

use rpg_network_protocol::{protocol::*, *};
use rpg_world::zone::{Zone, ZoneId};

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

#[derive(Default, Resource)]
pub(crate) struct RpgWorld {
    pub(crate) zones: HashMap<ZoneId, Zone>,
}

pub(crate) struct ServerWorldPlugin;

impl Plugin for ServerWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RpgWorld>()
            .add_systems(Update, spawn_world.in_set(FixedUpdateSet::Main));
    }
}

fn spawn_world(world: ResMut<RpgWorld>) {}
