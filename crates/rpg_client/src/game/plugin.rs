#![allow(clippy::too_many_arguments)]

use crate::{assets::AudioAssets, loader::plugin::OutOfGameCamera, state::AppState};

use super::{
    actor::{self, player, unit},
    assets::RenderResources,
    controls::{self, Controls, CursorPosition},
    environment,
    item::{self, CursorItem},
    metadata::MetadataResources,
    passive_tree, state_saver, ui, world,
    world::{LoadZone, RpgWorld},
};

use rpg_account::character_statistics::CharacterStatistics;
use rpg_core::game_mode::GameMode;
use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions,
    item::GroundItemDrops,
    skill::{clean_skills, update_skill},
};

use util::{
    cleanup::CleanupStrategy,
    random::{Rng, SharedRng},
};

use bevy::{
    app::{App, Plugin, PostUpdate, PreUpdate, Update},
    audio::{AudioSink, PlaybackSettings},
    core_pipeline::{bloom::BloomSettings, core_3d::Camera3dBundle, tonemapping::Tonemapping},
    ecs::{
        component::Component,
        entity::Entity,
        query::{With, Without},
        schedule::{common_conditions::*, Condition, IntoSystemConfigs, NextState, OnEnter},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    gizmos::config::{GizmoConfig, GizmoConfigGroup},
    hierarchy::DespawnRecursiveExt,
    log::{debug, info},
    math::Vec3,
    pbr::{AmbientLight, DirectionalLightShadowMap},
    reflect::Reflect,
    render::{
        camera::{Camera, ClearColorConfig},
        color::Color,
    },
    utils::default,
};

use bevy_renet::renet::RenetClient;

#[derive(Clone, Copy, Component)]
pub struct GameSessionCleanup;

#[derive(Default, Reflect, GizmoConfigGroup)]
struct GameGizmos {}

#[derive(Component)]
pub struct GameCamera {
    pub offset: Vec3,
    pub min_y: f32,
    pub max_y: f32,
}

pub(crate) fn build_character_stats_string(stats: &CharacterStatistics) -> String {
    format!(
        "Player Stats:\n\nKills: {} Attacks: {} Hits: {}\nBlocks: {} Dodges: {}\n\nTimes Hit: {} Times Blocked: {} Times Dodged: {}\n\nItems Looted: {}",
        stats.kills,
        stats.attacks,
        stats.hits,
        stats.blocks,
        stats.dodges,
        stats.times_hit,
        stats.times_blocked,
        stats.times_dodged,
        stats.items_looted,
    )
}

#[derive(Component)]
pub(crate) struct BackgroundMusic;

#[derive(Component)]
pub(crate) struct EnvironmentalSound;

#[derive(Component)]
struct ActorSound;

#[derive(Debug, Default, PartialEq)]
pub enum PlayState {
    #[default]
    Loading,
    Game,
}

impl PlayState {
    pub fn loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    pub fn game(&self) -> bool {
        matches!(self, Self::Game)
    }
}

#[derive(Debug, Resource, Default)]
pub struct GameState {
    pub state: PlayState,
    pub mode: GameMode,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        debug!("initializing");

        app.add_event::<LoadZone>()
            .init_resource::<Controls>()
            .init_resource::<CursorPosition>()
            .init_resource::<CursorItem>()
            .init_resource::<GroundItemDrops>()
            .init_resource::<GameState>()
            .init_resource::<world::RpgWorld>()
            .insert_resource(SharedRng(Rng::with_seed(1234)))
            .insert_resource(DirectionalLightShadowMap { size: 2048 })
            .insert_resource(AmbientLight {
                brightness: 0.,
                ..default()
            })
            // GameSpawn
            .add_systems(
                OnEnter(AppState::GameSpawn),
                (
                    (setup, setup_audio).chain(),
                    ui::hud::setup,
                    ui::hero::setup,
                    ui::inventory::setup,
                    ui::menu::setup,
                    passive_tree::setup,
                    passive_tree::setup_ui,
                ),
            )
            .add_systems(
                Update,
                (
                    world::zone::load_zone,
                    environment::prepare_environment,
                    transition_game_join,
                )
                    .chain()
                    .run_if(in_state(AppState::GameSpawn)),
            )
            .add_systems(OnEnter(AppState::GameJoin), send_client_ready)
            // Game
            /*.add_systems(
                First,
                actor::actor::replace_actor_materials.run_if(in_state(AppState::Game)),
            )*/
            .add_systems(
                Update,
                (
                    (
                        (
                            controls::update_controls,
                            player::input_actions,
                            unit::action,
                            update_skill,
                            unit::pick_storable_items,
                            player::update_camera,
                        )
                            .chain(),
                        (
                            (
                                passive_tree::toggle_passive_tree,
                                passive_tree::set_view,
                                passive_tree::update_legend,
                                passive_tree::display,
                            )
                                .chain(),
                            background_audio,
                            unit::unit_audio,
                            actor::animation::animator,
                            ui::menu::toggle_menu,
                            ui::menu::exit_button,
                            ui::menu::cancel_button,
                            ui::menu::respawn_button,
                        )
                            .after(player::update_camera),
                    ),
                    (
                        ui::inventory::inventory_update,
                        ui::inventory::hover_storage,
                        item::hover_ground_item,
                        ui::inventory::update_cursor_item,
                        ui::inventory::update,
                    )
                        .chain()
                        .after(unit::pick_storable_items),
                )
                    .run_if(in_state(AppState::Game).and_then(is_game)),
            )
            .add_systems(
                PostUpdate,
                (
                    player::update_debug_lines,
                    player::update_debug_gizmos,
                    player::update_spotlight,
                    unit::update_health_bars,
                    item::spawn_ground_items,
                    item::animate_ground_items,
                    ui::hud::update,
                    unit::toggle_healthbar,
                )
                    .run_if(in_state(AppState::Game).and_then(is_game)),
            )
            .add_systems(
                PreUpdate,
                (
                    environment::day_night_cycle,
                    clean_skills,
                    actions::action_tick,
                )
                    .run_if(in_state(AppState::Game).and_then(is_game)),
            )
            .add_systems(OnEnter(AppState::Game), set_playing)
            // GameCleanup
            .add_systems(
                OnEnter(AppState::GameCleanup),
                (
                    util::cleanup::cleanup::<GameSessionCleanup>,
                    environment::cleanup,
                    cleanup,
                ),
            )
            .add_systems(
                Update,
                (|mut state: ResMut<NextState<AppState>>| {
                    state.set(AppState::Menu);
                })
                .run_if(in_state(AppState::GameCleanup)),
            );
    }
}

fn set_playing(mut game_state: ResMut<GameState>) {
    game_state.state = PlayState::Game;
}

pub(crate) fn background_audio(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    background_music_q: Query<Entity, (Without<AudioSink>, With<BackgroundMusic>)>,
    environmental_sound_q: Query<Entity, (Without<AudioSink>, With<EnvironmentalSound>)>,
    mut rng: ResMut<SharedRng>,
) {
    for entity in &background_music_q {
        let track_roll = rng.0.usize(1..=7);
        let key = format!("bg_loop{track_roll}");
        let handle = audio_assets.background_tracks[key.as_str()].clone_weak();

        commands
            .entity(entity)
            .insert((handle, PlaybackSettings::REMOVE));

        debug!("switching to bg track {key}");
    }

    for entity in &environmental_sound_q {
        let handle = audio_assets.background_tracks["env_swamp"].clone_weak();

        commands
            .entity(entity)
            .insert((handle, PlaybackSettings::LOOP));
    }
}

fn transition_game_join(mut state: ResMut<NextState<AppState>>, rpg_world: Res<RpgWorld>) {
    if rpg_world.active_zone.is_some() && rpg_world.env_loaded {
        state.set(AppState::GameJoin);
    }
}

fn send_client_ready(mut net_client: ResMut<RenetClient>) {
    info!("sending client ready message");
    let message = bincode::serialize(&ClientMessage::CSClientReady(CSClientReady)).unwrap();
    net_client.send_message(ClientChannel::Message, message);
}

fn is_loading(game_state: Res<GameState>) -> bool {
    game_state.state.loading()
}

fn is_game(game_state: Res<GameState>) -> bool {
    game_state.state.game()
}

pub(crate) fn load_assets(mut commands: Commands) {
    info!("loading game assets");

    commands.init_resource::<RenderResources>();
    commands.init_resource::<MetadataResources>();
}

fn setup_audio(mut commands: Commands) {
    commands.spawn((
        GameSessionCleanup,
        CleanupStrategy::Despawn,
        BackgroundMusic,
    ));
    commands.spawn((
        GameSessionCleanup,
        CleanupStrategy::Despawn,
        EnvironmentalSound,
    ));
}

fn setup(mut commands: Commands, camera_q: Query<Entity, With<OutOfGameCamera>>) {
    info!("spawning world");

    commands.entity(camera_q.single()).despawn_recursive();

    /* FIXME update this
    commands.insert_resource(GizmoConfig {
        enabled: true,
        aabb: AabbGizmoConfig {
            draw_all: false,
            ..default()
        },
        ..default()
    });*/

    let default_y = 10.;

    // camera
    commands.spawn((
        GameSessionCleanup,
        CleanupStrategy::DespawnRecursive,
        GameCamera {
            offset: Vec3::new(0., default_y, default_y * 0.6),
            min_y: 8.,
            max_y: 128.,
        },
        Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings {
            intensity: 0.25,
            ..default()
        },
        /*FogSettings {
            color: Color::rgba(0.2, 0.2, 0.2, 1.0),
            directional_light_color: Color::rgba(0.98, 0.98, 0.95, 0.5),
            directional_light_exponent: 40.0,
            falloff: FogFalloff::from_visibility_colors(
                100.,
                Color::rgb(0.15, 0.15, 0.15), // atmospheric extinction color
                Color::rgb(0.25, 0.25, 0.25), // atmospheric inscattering color
            ),
        },*/
    ));
}

fn cleanup(mut game_state: ResMut<GameState>, mut controls: ResMut<Controls>) {
    debug!("cleanup");

    controls.reset();

    *game_state = GameState::default();
}
