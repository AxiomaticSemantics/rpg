use super::{action, item, skill, unit, villain};

use crate::{
    assets::MetadataResources, net::server::NetworkParamsRW, server_state::ServerMetadataResource,
    state::AppState, world::WorldPlugin,
};

use bevy::{
    app::{App, FixedPreUpdate, FixedUpdate, Plugin, Update},
    ecs::{
        component::Component,
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter},
        system::{Commands, Res, ResMut, Resource},
    },
    log::info,
    math::Vec3,
};

use lightyear::netcode::ClientId;
use lightyear::shared::NetworkTarget;

use rpg_account::{account::AccountId, character::CharacterSlot};
use rpg_core::{uid::Uid, unit::HeroGameMode};
use rpg_network_protocol::protocol::*;
use rpg_util::{item::GroundItemDrops, skill::SkillContactEvent};

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
    pub(crate) slot: CharacterSlot,
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
            .add_event::<SkillContactEvent>()
            .init_resource::<GameState>()
            .init_resource::<AabbResources>()
            .init_resource::<GroundItemDrops>()
            .insert_resource(SharedRng(Rng::with_seed(1234)))
            .add_systems(OnEnter(AppState::SpawnSimulation), setup_simulation)
            .add_systems(OnEnter(AppState::Simulation), join_clients)
            .add_systems(
                Update,
                transition_to_game.run_if(in_state(AppState::SpawnSimulation)),
            )
            .add_systems(
                FixedPreUpdate,
                (
                    villain::remote_spawn,
                    skill::update_invulnerability,
                    unit::attract_resource_items,
                    unit::remove_corpses,
                    skill::clean_skills,
                    rpg_util::actions::action_tick,
                )
                    .run_if(in_state(AppState::Simulation)),
            )
            .add_systems(
                FixedUpdate,
                (
                    unit::upkeep,
                    action::action,
                    skill::update_skill,
                    skill::collide_skills,
                    skill::handle_contacts,
                    item::spawn_ground_items,
                    villain::find_target,
                    villain::villain_think,
                )
                    .chain()
                    .run_if(in_state(AppState::Simulation)),
            );
    }
}

pub(crate) fn setup_simulation(
    mut commands: Commands,
    mut rng: ResMut<SharedRng>,
    metadata: Res<MetadataResources>,
    mut aabbs: ResMut<AabbResources>,
    mut server_metadata: ResMut<ServerMetadataResource>,
) {
    info!("spawning game");

    aabbs.aabbs.insert(
        Cow::Owned("direct_attack".into()),
        Aabb::from_min_max(Vec3::new(-0.1, -0.1, -0.5), Vec3::new(0.1, 0.1, 0.5)),
    );
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

pub(crate) fn join_clients(game_state: ResMut<GameState>, mut net_params: NetworkParamsRW) {
    info!("joining clients to game");

    let client_ids = game_state.client_ids();

    // FIXME spawn positions need to account for player intersections
    // for now just spawn all clients at the origin
    net_params.server.send_message_to_target::<Channel1, _>(
        SCPlayerJoinSuccess,
        NetworkTarget::Only(client_ids.clone()),
    );
}
