use rpg_util::unit::Unit;

use bevy::ecs::{
    component::Component,
    system::{Query, Res},
};

// TODO reuse this, functionality was moved to a shared library, any client
// specific behavrious will can here.
