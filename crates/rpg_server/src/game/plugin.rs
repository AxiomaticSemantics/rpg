use super::{action, unit, villain};

use crate::{
    account::AccountInstance,
    assets::MetadataResources,
    net::server::NetworkParamsRW,
    server_state::ServerMetadataResource,
    state::AppState,
    world::{RpgWorld, WorldPlugin},
};

use bevy::{
    app::{App, FixedPreUpdate, FixedUpdate, Plugin, Update},
    ecs::{
        component::Component,
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    log::info,
    math::Vec3,
};

use lightyear::netcode::ClientId;
use lightyear::shared::replication::components::NetworkTarget;

use rpg_account::account::AccountId;
use rpg_core::{uid::Uid, unit::HeroGameMode};
use rpg_network_protocol::protocol::*;

use util::{
    math::Aabb,
    random::{Rng, SharedRng},
};

use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub(crate) struct GameOptions {
    pub(crate) mode: HeroGameMode,
    pub(crate) max_players: u8,
}

#[derive(Debug)]
pub(crate) struct PlayerIdInfo {
    pub(crate) client_id: ClientId,
    pub(crate) account_id: AccountId,
    pub(crate) character_id: Uid,
}

#[derive(Default, Debug, Resource)]
pub(crate) struct GameState {
    pub(crate) players: Vec<PlayerIdInfo>,
    pub(crate) options: GameOptions,
}

#[derive(Default, Resource)]
pub(crate) struct AabbResources {
    pub(crate) aabbs: HashMap<Cow<'static, str>, Aabb>,
}

impl GameState {
    pub(crate) fn client_ids(&self) -> Vec<ClientId> {
        self.players.iter().map(|p| p.client_id).collect()
    }

    pub(crate) fn get_id_info_from_account_id(
        &self,
        account_id: AccountId,
    ) -> Option<&PlayerIdInfo> {
        self.players.iter().find(|p| p.account_id == account_id)
    }
}

#[derive(Default, Component)]
pub(crate) struct GameSessionCleanup;

pub(crate) struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorldPlugin)
            .init_resource::<GameState>()
            .init_resource::<AabbResources>()
            .insert_resource(SharedRng(Rng::with_seed(1234)))
            .add_systems(OnEnter(AppState::SpawnSimulation), setup_simulation)
            .add_systems(OnEnter(AppState::Simulation), join_clients)
            .add_systems(
                Update,
                transition_to_game.run_if(in_state(AppState::SpawnSimulation)),
            )
            .add_systems(
                FixedPreUpdate,
                (villain::remote_spawn, unit::remove_corpses)
                    .run_if(in_state(AppState::Simulation)),
            )
            .add_systems(
                FixedUpdate,
                (unit::upkeep, villain::villain_think, action::action)
                    .chain()
                    .run_if(in_state(AppState::Simulation)),
            );
    }
}

pub(crate) fn setup_simulation(
    mut commands: Commands,
    game_state: Res<GameState>,
    mut rng: ResMut<SharedRng>,
    metadata: Res<MetadataResources>,
    mut server_metadata: ResMut<ServerMetadataResource>,
    account_q: Query<&AccountInstance>,
) {
    info!("spawning game");

    for account in &account_q {
        for id in game_state.players.iter() {
            if account.0.info.id == id.account_id {
                //
            }
        }
    }

    for _ in 0..50 {
        let position = Vec3::new(rng.f32() * 128.0 - 64.0, 0., rng.f32() * 128.0 - 64.0);

        villain::spawn(
            &mut commands,
            &mut server_metadata.0.next_uid,
            &position,
            &metadata.0,
            &mut rng,
        );
    }

    // TODO write server metadata
}

pub(crate) fn transition_to_game(mut state: ResMut<NextState<AppState>>) {
    info!("transitioning to game simulation");
    state.set(AppState::Simulation);
}

pub(crate) fn join_clients(mut game_state: ResMut<GameState>, mut net_params: NetworkParamsRW) {
    let client_ids = game_state.client_ids();

    // FIXME spawn positions need to account for player intersections
    // for now just spawn all clients at the origin
    net_params.server.send_message_to_target::<Channel1, _>(
        SCPlayerJoinSuccess,
        NetworkTarget::Only(client_ids.clone()),
    );
}
