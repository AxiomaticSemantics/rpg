use bevy::{
    app::{App, Plugin, Startup},
    asset::{AssetServer, Handle},
    audio::{AudioSink, AudioSource, GlobalVolume},
    ecs::{
        component::Component,
        system::{Commands, Res, ResMut, Resource},
    },
    prelude::{Deref, DerefMut},
};

pub struct AudioManagerPlugin;

impl Plugin for AudioManagerPlugin {
    fn build(&self, app: &mut App) {
        println!("Initializing audio plugin.");

        app.add_systems(Startup, setup);
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

fn setup(mut volume: ResMut<GlobalVolume>) {
    *volume.volume = 0.5;
}
