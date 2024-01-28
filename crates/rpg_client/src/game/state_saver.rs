use crate::state::AppState;

use bevy::ecs::{
    component::Component,
    event::{Event, EventReader},
    query::With,
    schedule::NextState,
    system::{Query, ResMut, Resource},
};

use util::fs::{open_read, open_write};

use std::{env, io, path::Path};

// This will be moved to the crate root and be used to persist settings
