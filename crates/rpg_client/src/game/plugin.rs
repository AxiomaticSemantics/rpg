#![allow(clippy::too_many_arguments)]

use crate::{assets::AudioAssets, loader::plugin::OutOfGameCamera, state::AppState};

use super::{
    actor::{self, player, unit},
    assets::RenderResources,
    controls::{self, Controls, CursorPosition},
    environment,
    item::{self, CursorItem},
    metadata::MetadataResources,
    passive_tree, skill, state_saver, ui, world,
};

use rpg_core::unit::HeroGameMode;
use rpg_network_protocol::protocol::*;
use rpg_util::{actions, item::GroundItemDrops, skill::update_skill};

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
        schedule::{
            common_conditions::*, Condition, IntoSystemConfigs, NextState, OnEnter, OnExit,
        },
        system::{Commands, Query, Res, ResMut, Resource},
    },
    gizmos::config::{GizmoConfig, GizmoConfigGroup},
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

#[derive(Debug, Default)]
pub struct SessionStats {
    pub kills: u32,
    pub times_hit: u32,
    pub attacks: u32,
    pub hits: u32,
    pub dodges: u32,
    pub times_dodged: u32,
    pub blocks: u32,
    pub times_blocked: u32,
    pub chains: u32,
    pub times_chained: u32,
    pub pierced: u32,
    pub times_pierced: u32,
    pub knockbacks: u32,
    pub knockback_distance: f32,
    pub times_knockbacked: u32,
    pub distance_knockbacked: f32,
    pub distance_travelled: f32,
    pub items_looted: u32,

    // Various stats
    pub hp_consumed: u32,
    pub hp_generated: u32,
    pub ep_consumed: u32,
    pub ep_generated: u32,
    pub mp_consumed: u32,
    pub mp_generated: u32,
    pub damage_dealt: u64,
    pub damage_received: u64,
    pub patk_damage_dealt: u64,
    pub matk_damage_dealt: u64,
    pub tatk_damage_dealt: u64,
    pub patk_damage_received: u64,
    pub matk_damage_received: u64,
    pub tatk_damage_received: u64,
}

pub(crate) fn build_stats_string(stats: &SessionStats) -> String {
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
    Death(GameOverState),
}

impl PlayState {
    pub fn loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    pub fn game(&self) -> bool {
        matches!(self, Self::Game)
    }

    pub fn death(&self) -> bool {
        matches!(self, Self::Death(_))
    }
}

#[derive(Debug, Default, PartialEq)]
pub enum GameOverState {
    #[default]
    Pending,
    Restart,
    Exit,
}

#[derive(Debug, Resource, Default)]
pub struct GameState {
    pub state: PlayState,
    pub mode: HeroGameMode,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        debug!("initializing");

        app.init_resource::<Controls>()
            .init_resource::<CursorPosition>()
            .init_resource::<CursorItem>()
            .init_resource::<GroundItemDrops>()
            .init_resource::<GameState>()
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
                    (setup, setup_audio, world::zone::setup, environment::setup).chain(),
                    send_client_ready,
                    (
                        ui::hud::setup,
                        ui::hero::setup,
                        ui::inventory::setup,
                        ui::menu::setup,
                        ui::game_over::setup,
                        passive_tree::setup,
                        passive_tree::setup_ui,
                    )
                        .after(environment::setup)
                        .before(send_client_ready),
                ),
            )
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
                            unit::attract_resource_items,
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
                Update,
                (
                    ui::menu::toggle_menu,
                    ui::menu::save_button,
                    ui::menu::cancel_button,
                )
                    .chain()
                    .run_if(in_state(AppState::Game).and_then(is_death)),
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
                )
                    .run_if(in_state(AppState::Game).and_then(is_game)),
            )
            // This system is special and transitions from Game to GameOver when the player dies
            .add_systems(
                PostUpdate,
                (
                    ui::hud::update,
                    ui::game_over::exit_button,
                    ui::game_over::restart_button,
                    ui::game_over::game_over_transition,
                )
                    .chain()
                    .run_if(in_state(AppState::Game).and_then(is_death)),
            )
            .add_systems(
                PreUpdate,
                (
                    environment::day_night_cycle,
                    // TODO decide if this will be needed againvillain::spawner,
                    unit::remove_healthbar,
                    skill::clean_skills,
                    actions::action_tick,
                )
                    .run_if(in_state(AppState::Game).and_then(is_game)),
            )
            .add_systems(OnEnter(AppState::Game), set_playing)
            // GameOver
            .add_systems(
                OnExit(AppState::GameOver),
                (
                    util::cleanup::cleanup::<GameSessionCleanup>,
                    environment::cleanup,
                    cleanup,
                ),
            )
            .add_systems(
                Update,
                (
                    actor::animation::animator,
                    ui::game_over::exit_button,
                    ui::game_over::restart_button,
                )
                    .run_if(in_state(AppState::GameOver)),
            )
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

fn send_client_ready(mut net_client: ResMut<Client>) {
    info!("sending client ready message");
    net_client.send_message::<Channel1, _>(CSClientReady);
}

fn is_loading(game_state: Res<GameState>) -> bool {
    game_state.state.loading()
}

fn is_game(game_state: Res<GameState>) -> bool {
    game_state.state.game()
}

fn is_death(game_state: Res<GameState>) -> bool {
    game_state.state.death()
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

fn setup(mut commands: Commands, mut camera_2d_q: Query<&mut Camera, With<OutOfGameCamera>>) {
    info!("spawning world");

    camera_2d_q.single_mut().is_active = false;

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

    info!("spawning complete");
}

fn cleanup(mut game_state: ResMut<GameState>, mut controls: ResMut<Controls>) {
    debug!("game::plugin cleanup");

    controls.reset();

    match game_state.state {
        PlayState::Death(GameOverState::Restart) => {
            game_state.state = PlayState::default();
        }
        PlayState::Death(GameOverState::Exit) => {
            *game_state = GameState::default();
        }
        _ => {
            panic!("Should not get here. {game_state:?}");
        }
    }
}
