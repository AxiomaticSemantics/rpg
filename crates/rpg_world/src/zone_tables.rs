use crate::zone::{Kind, ZoneId};

use bevy_math::UVec2;

use serde_derive::Deserialize as De;
use std::collections::HashMap;

#[derive(De)]
pub struct ZonePropDescriptor {
    pub key: String,
    pub position: UVec2,
}

#[derive(De)]
pub struct TownDescriptor {
    pub name: String,
    pub spawn_position: UVec2,
    pub size: UVec2,
    pub kind: Kind,
    pub props: Vec<ZonePropDescriptor>,
}

#[derive(De)]
pub struct StaticZoneDescriptor {
    pub name: String,
}

#[derive(De)]
pub struct BasicSizeInfo {
    pub room: UVec2,
    pub tile: UVec2,
}

#[derive(De)]
pub struct ZoneTable {
    pub size_info: BasicSizeInfo,
    pub towns: HashMap<ZoneId, TownDescriptor>,
    pub zones: HashMap<ZoneId, StaticZoneDescriptor>,
}
