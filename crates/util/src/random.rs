use bevy::{
    ecs::system::Resource,
    prelude::{Deref, DerefMut},
};

pub use fastrand::Rng;

#[derive(Resource, Deref, DerefMut)]
pub struct SharedRng(pub Rng);
