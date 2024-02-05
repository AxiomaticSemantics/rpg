use super::{action, item, skill, unit, villain};

use crate::{
    assets::MetadataResources, net::server::NetworkParamsRW, server_state::ServerMetadataResource,
    state::AppState, world::WorldPlugin,
};

use bevy::{
    app::{App, FixedPreUpdate, FixedUpdate, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter},
        system::{Commands, Res, ResMut, Resource},
    },
    log::info,
    math::{bounding::Aabb3d, Vec3},
};

use lightyear::netcode::ClientId;
use lightyear::shared::NetworkTarget;

use rpg_account::{account::AccountId, character::CharacterSlot};
use rpg_core::{uid::Uid, unit::HeroGameMode, villain::VillainId};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    item::GroundItemDrops,
    skill::{clean_skills, update_skill, SkillContactEvent},
};

use util::random::{Rng, SharedRng};

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
    pub(crate) entity: Entity,
}

#[derive(Default, Debug, Resource)]
pub(crate) struct GameState {
    pub(crate) players: Vec<PlayerIdInfo>,
    pub(crate) options: GameOptions,
}

#[derive(Default, Resource)]
pub(crate) struct AabbResources {
    pub(crate) aabbs: HashMap<Cow<'static, str>, Aabb3d>,
}

impl GameState {
    pub(crate) fn client_ids(&self) -> Vec<ClientId> {
        self.players.iter().map(|p| p.client_id).collect()
    }

    pub(crate) fn get_id_info_from_uid(&self, uid: Uid) -> Option<&PlayerIdInfo> {
        self.players.iter().find(|p| p.character_id == uid)
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
            .init_resource::<action::MovingUnits>()
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
                    unit::collide_units,
                    villain::remote_spawn,
                    skill::update_invulnerability,
                    unit::remove_corpses,
                    clean_skills,
                    rpg_util::actions::action_tick,
                    unit::upkeep,
                    item::spawn_ground_items,
                )
                    .run_if(in_state(AppState::Simulation)),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_skill,
                    skill::collide_skills,
                    skill::handle_contacts,
                    villain::find_target,
                    villain::villain_think,
                    action::action,
                    action::try_move_units,
                    action::move_units,
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

    // FIXME more aabbs need to be inserted, impl FromWorld and move
    aabbs.aabbs.insert(
        Cow::Owned("hero".into()),
        Aabb3d {
            min: Vec3::new(-0.3, 0.0, -0.25),
            max: Vec3::new(0.3, 1.8, 0.25),
        },
    );

    aabbs.aabbs.insert(
        Cow::Owned("direct_attack".into()),
        Aabb3d {
            min: Vec3::new(-0.1, -0.1, -0.5),
            max: Vec3::new(0.1, 0.1, 0.5),
        },
    );

    aabbs.aabbs.insert(
        Cow::Owned("item_normal".into()),
        Aabb3d {
            min: Vec3::new(-0.2, -0.2, -0.2),
            max: Vec3::new(0.2, 0.2, 0.2),
        },
    );

    aabbs.aabbs.insert(
        Cow::Owned("bolt_01".into()),
        Aabb3d {
            min: Vec3::new(-0.1, -0.1, -0.25),
            max: Vec3::new(0.1, 0.1, 0.25),
        },
    );

    for _ in 0..32 {
        let position = Vec3::new(rng.f32() * 128.0 - 64.0, 0., rng.f32() * 128.0 - 64.0);

        let villain_id = VillainId::sample(&mut rng);
        villain::spawn(
            &mut commands,
            &mut server_metadata.0.next_uid,
            &position,
            &metadata.0,
            aabbs.aabbs["hero"],
            villain_id,
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

    net_params.server.send_message_to_target::<Channel1, _>(
        SCPlayerJoinSuccess,
        NetworkTarget::Only(game_state.client_ids()),
    );
}
