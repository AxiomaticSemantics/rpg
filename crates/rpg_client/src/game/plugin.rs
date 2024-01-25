#![allow(clippy::too_many_arguments)]

use crate::{assets::AudioAssets, loader::plugin::OutOfGameCamera, state::AppState};

use super::{
    actor::{self, player, unit, villain},
    assets::RenderResources,
    controls::{self, Controls, CursorPosition},
    environment,
    item::{self, CursorItem, GroundItemDrops},
    metadata::MetadataResources,
    passive_tree, skill, state_saver, ui, world,
};

use rpg_core::{class::Class, uid::NextUid, unit::HeroGameMode};
use rpg_network_protocol::protocol::*;
use rpg_util::{actions, skill::SkillContactEvent, unit::Unit};

use audio_manager::plugin::AudioActions;
use util::{
    cleanup::CleanupStrategy,
    random::{Rng, SharedRng},
};

use bevy::{
    app::{App, Plugin, PostUpdate, PreUpdate, Update},
    audio::{AudioBundle, AudioSink, PlaybackSettings},
    core_pipeline::{bloom::BloomSettings, core_3d::Camera3dBundle, tonemapping::Tonemapping},
    ecs::{
        component::Component,
        entity::Entity,
        query::{Changed, With, Without},
        schedule::{
            common_conditions::*, Condition, IntoSystemConfigs, NextState, OnEnter, OnExit,
        },
        system::{Commands, Query, Res, ResMut, Resource},
    },
    gizmos::config::{GizmoConfig, GizmoConfigGroup},
    hierarchy::{BuildChildren, ChildBuilder, DespawnRecursiveExt},
    log::{debug, info},
    math::Vec3,
    pbr::{AmbientLight, DirectionalLightShadowMap},
    reflect::Reflect,
    render::{
        camera::{Camera, ClearColorConfig},
        color::Color,
    },
    time::{Stopwatch, Time, Timer, TimerMode},
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

#[derive(Debug)]
pub struct PlayerOptions {
    pub class: Class,
    pub name: String,
    pub game_mode: HeroGameMode,
}

#[derive(Debug, Default)]
pub struct SessionStats {
    pub spawned: u32,
    pub kills: u32,
    pub villain_attacks: u32,
    pub villain_hits: u32,
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
    pub items_spawned: u32,
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
    pub session_stats: SessionStats,
    pub next_uid: NextUid,
    pub state: PlayState,
    pub mode: HeroGameMode,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        debug!("initializing");

        app.add_event::<SkillContactEvent>()
            .init_resource::<Controls>()
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
                    (
                        setup,
                        setup_audio,
                        world::zone::setup,
                        environment::setup,
                        actor::player::spawn_player,
                    )
                        .chain(),
                    transition_to_game,
                    (
                        ui::hud::setup,
                        ui::hero::setup,
                        ui::inventory::setup,
                        ui::pause::setup,
                        ui::game_over::setup,
                        passive_tree::setup,
                    )
                        .after(actor::player::spawn_player)
                        .before(transition_to_game),
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
                            skill::update_skill,
                            // reuse for local unit::collide_units,
                            skill::collide_skills,
                            skill::handle_contact,
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
                            unit_audio,
                            actor::animation::animator,
                            ui::pause::user_pause,
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
                (ui::pause::user_pause, ui::pause::game_exit_button_pressed)
                    .chain()
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
                )
                    .run_if(in_state(AppState::Game).and_then(is_game)),
            )
            // This system is special and transitions from Game to GameOver when the player dies
            .add_systems(
                PostUpdate,
                (ui::hud::update, ui::game_over::game_over_transition)
                    .chain()
                    .run_if(in_state(AppState::Game).and_then(is_death)),
            )
            .add_systems(
                PreUpdate,
                (
                    environment::day_night_cycle,
                    // TODO decide if this will be needed again villain::spawner,
                    unit::remove_healthbar,
                    skill::clean_skills,
                    skill::update_invulnerability,
                    actions::action_tick,
                )
                    .run_if(in_state(AppState::Game).and_then(is_game)),
            )
            .add_systems(OnEnter(AppState::Game), set_playing)
            .add_systems(OnExit(AppState::GameSpawn), send_player_ready)
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
                (actor::animation::animator, ui::game_over::game_over)
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

fn transition_to_game(mut state: ResMut<NextState<AppState>>) {
    debug!("transition to game");
    state.set(AppState::Game);
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

fn send_player_ready(mut net_client: ResMut<Client>) {
    net_client.send_message::<Channel1, _>(CSPlayerReady);
}

pub(crate) fn unit_audio(
    mut commands: Commands,
    tracks: Res<AudioAssets>,
    mut unit_q: Query<(Entity, &mut AudioActions), (With<Unit>, Changed<AudioActions>)>,
) {
    for (entity, mut audio_actions) in &mut unit_q {
        for action in audio_actions.iter() {
            commands
                .spawn(AudioBundle {
                    source: tracks.foreground_tracks[action.as_str()].clone_weak(),
                    settings: PlaybackSettings::REMOVE,
                })
                .set_parent(entity);
        }
        audio_actions.clear();
    }
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

fn _calculate_normals(indices: &Vec<u32>, vertices: &[[f32; 3]], normals: &mut [[f32; 3]]) {
    let vertex_count = indices.len();

    for i in (0..vertex_count).step_by(3) {
        let v1 = Vec3::from_array(vertices[indices[i + 1] as usize])
            - Vec3::from_array(vertices[indices[i] as usize]);
        let v2 = Vec3::from_array(vertices[indices[i + 2] as usize])
            - Vec3::from_array(vertices[indices[i] as usize]);
        let face_normal = v1.cross(v2).normalize();

        // Add the face normal to the 3 vertex normals that are touching this face
        normals[indices[i] as usize] =
            (Vec3::from_array(normals[indices[i] as usize]) + face_normal).to_array();
        normals[indices[i + 1] as usize] =
            (Vec3::from_array(normals[indices[i + 1] as usize]) + face_normal).to_array();
        normals[indices[i + 2] as usize] =
            (Vec3::from_array(normals[indices[i + 2] as usize]) + face_normal).to_array();
    }

    // Now loop through each vertex vector, and avarage out all the normals stored.
    for normal in &mut normals.iter_mut() {
        *normal = Vec3::from_array(*normal).normalize().to_array();
    }
}

fn _make_indices(indices: &mut Vec<u32>, size: [u32; 2]) {
    for y in 0..size[1] - 1 {
        for x in 0..size[0] - 1 {
            let index = y * size[0] + x;
            indices.push(index + size[0] + 1);
            indices.push(index + 1);
            indices.push(index + size[0]);
            indices.push(index);
            indices.push(index + size[0]);
            indices.push(index + 1);
        }
    }
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

    // let mut image = images.get_mut(&textures.heightmap).unwrap();
    // image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
    // address_mode_u: AddressMode::MirrorRepeat,
    // address_mode_v: AddressMode::MirrorRepeat,
    // address_mode_w: AddressMode::Repeat,
    // ..default()
    // });
    //
    // let image_size = image.size();
    // let size = image_size.x as u32;
    // let size_y = size - 1;
    // let size_x = size - 1;
    // let num_vertices = (size_y * size_x) as usize;
    // let num_indices = ((size_y - 1) * (size_x - 1) * 6) as usize;
    //
    // let mut positions: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    // let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    // let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
    // let mut indices: Vec<u32> = Vec::with_capacity(num_indices);
    //
    // let mut uv: [f32; 2] = [0., 1.];
    // for y in 0..size_y {
    // if y % 8 == 0 {
    // uv[1] = 1.;
    // }
    //
    // for x in 0..size_x {
    // if x % 8 == 0 {
    // uv[0] = 0.;
    // }
    //
    // let index = y * size_x + x;
    //
    // let h = *image.data.get(index as usize * 4).unwrap();
    // let h_s = (size_x - 1) as f32 / 2.;
    //
    // let pos = Vec3::new(-h_s + x as f32, -16. + (h as f32) / 8., -h_s + y as f32);
    // positions.push(pos.to_array());
    // normals.push([0., 0., 0.]);
    // uvs.push(uv);
    //
    // println!("UV: {uv:?}");
    //
    // uv[0] += 0.142857142857;
    // }
    // uv[1] -= 0.142857142857;
    // }
    //
    // make_indices(&mut indices, [size_x, size_y]);
    // calculate_normals(&indices, &positions, &mut normals);
    //
    // let mut terrain_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    // terrain_mesh.set_indices(Some(Indices::U32(indices)));
    // terrain_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    // terrain_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    // terrain_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    //
    // commands.spawn(PbrBundle {
    // mesh: meshes.add(terrain_mesh),
    // material: materials.add(StandardMaterial {
    // base_color_texture: Some(textures.seamless_grass.clone()),
    // perceptual_roughness: 0.95,
    // ..default()
    // }),
    // ..default()
    // });
    // transform: Transform::from_xyz(32., 32., 0.),
    // ..default()
    // });

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
