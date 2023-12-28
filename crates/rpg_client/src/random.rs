use bevy::{
    ecs::system::Resource,
    prelude::{Deref, DerefMut},
};

use fastrand::Rng;

#[derive(Resource, Deref, DerefMut)]
pub(crate) struct Random(pub(crate) Rng);
