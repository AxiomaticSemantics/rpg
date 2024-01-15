use bevy::{
    app::{App, Plugin, Startup},
    audio::{AudioSink, AudioSource, GlobalVolume},
    ecs::{component::Component, system::ResMut},
    log::info,
    prelude::{Deref, DerefMut},
};

pub struct AudioManagerPlugin;

impl Plugin for AudioManagerPlugin {
    fn build(&self, _app: &mut App) {
        info!("Initializing audio plugin.");
    }
}

/// A marker component to signify a background track
#[derive(Component)]
pub struct BackgroundAudio;

/// A marker component to signify a foreground track
#[derive(Component)]
pub struct ForegroundAudio;

#[derive(Default, Debug, Component, Deref, DerefMut)]
pub struct AudioActions(pub Vec<String>);
