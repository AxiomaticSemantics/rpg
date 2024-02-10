use crate::zone::ZoneId;

use serde_derive::Deserialize as De;
use std::collections::HashMap;

#[derive(De, Default)]
pub struct TownDescriptor {
    pub name: String,
}

#[derive(De, Default)]
pub struct StaticZoneDescriptor {
    pub name: String,
}

#[derive(De)]
pub struct ZoneTable {
    pub towns: HashMap<ZoneId, TownDescriptor>,
    pub zones: HashMap<ZoneId, StaticZoneDescriptor>,
}
