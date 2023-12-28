#![allow(clippy::too_many_arguments)]

use crate::{
    assets::AudioAssets, loader::plugin::OutOfGameCamera, random::Random, state::AppState,
};

use super::{
    actions,
    actor::{
        self, player,
        unit::{self, Unit, VillainSpawner},
    },
    assets::RenderResources,
    controls::{self, Controls, CursorPosition},
    environment,
    item::{self, CursorItem, GroundItemDrops},
    metadata::MetadataResources,
    passive_tree,
    skill::{self, SkillEvent},
    state_saver, ui, zone,
};

use audio_manager::plugin::AudioActions;
use rpg_core::{class::Class, uid::NextUid};
use util::cleanup::CleanupStrategy;

use bevy::{
    app::{App, Plugin, PostUpdate, PreUpdate, Update},
    audio::{AudioBundle, AudioSink, PlaybackSettings},
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColor, core_3d::Camera3dBundle,
        tonemapping::Tonemapping,
    },
    ecs::prelude::*,
    ecs::schedule::IntoSystemConfigs,
    gizmos::{AabbGizmoConfig, GizmoConfig},
    hierarchy::prelude::*,
    math::Vec3,
    pbr::{AmbientLight, DirectionalLightShadowMap},
    render::{camera::Camera, color::Color},
    time::{Stopwatch, Time, Timer, TimerMode},
    utils::default,
};

use fastrand::Rng;

#[derive(Clone, Copy, Component)]
pub struct GameSessionCleanup;

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
    Play,
    Paused,
    GameOver(GameOverState),
}

impl PlayState {
    pub fn playing(&self) -> bool {
        matches!(self, Self::Play)
    }

    pub fn paused(&self) -> bool {
        matches!(self, Self::Paused)
    }

    pub fn game_over(&self) -> bool {
        matches!(self, Self::GameOver(_))
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
}

#[derive(Debug, Resource, Default)]
pub struct GameTime {
    pub watch: Stopwatch,
}

#[derive(Debug, Resource, Default)]
pub struct GameConfig {
    pub player_config: Option<PlayerOptions>,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        println!("Initializing game client plugin.");

        app.add_event::<state_saver::SaveGame>()
            .add_event::<SkillEvent>()
            .init_resource::<Controls>()
            .init_resource::<CursorPosition>()
            .init_resource::<CursorItem>()
            .init_resource::<GameConfig>()
            .init_resource::<GroundItemDrops>()
            .insert_resource(ClearColor(Color::BLACK))
            .insert_resource(DirectionalLightShadowMap { size: 2048 })
            .insert_resource(AmbientLight {
                brightness: 0.,
                ..default()
            })
            .insert_resource(Random(Rng::with_seed(1234)))
            .init_resource::<GameState>()
            .insert_resource(VillainSpawner {
                units: 1,
                frequency: 10.,
                timer: Timer::from_seconds(10., TimerMode::Repeating),
            })
            .init_resource::<GameTime>()
            // Load Enter
            .add_systems(
                OnEnter(AppState::GameSpawn),
                (
                    (
                        setup,
                        setup_audio,
                        zone::zone::setup,
                        environment::setup,
                        actor::player::spawn_player,
                    )
                        .chain(),
                    (
                        ui::hud::setup,
                        ui::hero::setup,
                        ui::inventory::setup,
                        ui::pause::setup,
                        ui::game_over::setup,
                        passive_tree::passive_tree::setup,
                    )
                        .after(actor::player::spawn_player),
                ),
            )
            /*.add_systems(
                First,
                actor::actor::replace_actor_materials.run_if(in_state(AppState::Game)),
            )*/
            .add_systems(
                Update,
                spawn_to_game_transition.run_if(in_state(AppState::GameSpawn)),
            )
            .add_systems(
                Update,
                (
                    (
                        (
                            controls::update_controls,
                            player::input_actions,
                            unit::villain_think,
                            unit::action,
                            skill::update_skill,
                            unit::collide_units,
                            skill::collide_skills,
                            skill::handle_contact,
                            unit::attract_resource_items,
                            unit::pick_storable_items,
                            player::update_camera,
                        )
                            .chain(),
                        (
                            (
                                passive_tree::passive_tree::toggle_passive_tree,
                                passive_tree::passive_tree::set_view,
                                passive_tree::passive_tree::update_legend,
                                passive_tree::passive_tree::display,
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
                        ui::inventory::hover_storage,
                        ui::inventory::inventory_update,
                        item::hover_ground_item,
                        ui::inventory::update_cursor_item,
                        ui::inventory::update,
                    )
                        .chain()
                        .after(unit::pick_storable_items),
                )
                    .run_if(in_state(AppState::Game).and_then(is_playing)),
            )
            .add_systems(
                Update,
                (
                    ui::pause::user_pause,
                    ui::pause::save_button_pressed,
                    state_saver::save_character,
                )
                    .chain()
                    .run_if(in_state(AppState::Game).and_then(is_paused)),
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
                    ui::game_over::game_over_transition,
                )
                    .run_if(in_state(AppState::Game).and_then(is_playing)),
            )
            .add_systems(
                PreUpdate,
                (
                    environment::day_night_cycle,
                    stopwatch,
                    unit::upkeep,
                    unit::spawner,
                    unit::corpse_removal,
                    skill::clean_skills,
                    skill::update_invulnerability,
                    actions::action_tick,
                )
                    .run_if(in_state(AppState::Game).and_then(is_playing)),
            )
            .add_systems(OnEnter(AppState::Game), stopwatch_restart)
            // Exit
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

pub(crate) fn background_audio(
    mut commands: Commands,
    audio_assets: Res<AudioAssets>,
    background_music_q: Query<Entity, (Without<AudioSink>, With<BackgroundMusic>)>,
    environmental_sound_q: Query<Entity, (Without<AudioSink>, With<EnvironmentalSound>)>,
    mut random: ResMut<Random>,
) {
    for entity in &background_music_q {
        let track_roll = random.0.usize(1..=7);
        let key = format!("bg_loop{track_roll}");
        let handle = audio_assets.background_tracks[key.as_str()].clone_weak();

        commands
            .entity(entity)
            .insert((handle, PlaybackSettings::REMOVE));

        println!("switching to bg track {key}");
    }

    for entity in &environmental_sound_q {
        let handle = audio_assets.background_tracks["env_swamp"].clone_weak();

        commands
            .entity(entity)
            .insert((handle, PlaybackSettings::LOOP));
    }
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

fn is_playing(game_state: Res<GameState>) -> bool {
    game_state.state.playing()
}

fn is_paused(game_state: Res<GameState>) -> bool {
    game_state.state.paused()
}

/*fn is_passive_tree_displayed(game_state: Res<GameState>) -> bool {
    game_state.state.passive_tree()
}*/

/*
pub(crate) fn game_over(game_state: Res<GameState>, mut state: ResMut<NextState<AppState>>) {
    if let PlayState::GameOver(_) = game_state.state {
        println!("game_over transition ");
    }
}*/

/*
fn _calculate_normals(indices: &Vec<u32>, vertices: &[[f32; 3]], normals: &mut [[f32; 3]]) {
    let vertex_count = indices.len();

    for i in (0..vertex_count).step_by(3) {
        let v1 = Vec3::from_array(vertices[indices[i + 1] as usize])
            - Vec3::from_array(vertices[indices[i] as usize]);
        let v2 = Vec3::from_array(vertices[indices[i + 2] as usize])
            - Vec3::from_array(vertices[indices[i] as usize]);
        let face_normal = v1.cross(v2).normalize();

        // Add the face normal to the 3 vertices normal touching this face
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
*/

pub(crate) fn load_assets(mut commands: Commands) {
    println!("loading game assets");

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

pub(crate) fn setup(
    mut commands: Commands,
    mut camera_2d_q: Query<&mut Camera, With<OutOfGameCamera>>,
) {
    println!("begin game spawn");

    camera_2d_q.single_mut().is_active = false;

    commands.insert_resource(GizmoConfig {
        enabled: true,
        aabb: AabbGizmoConfig {
            draw_all: false,
            ..default()
        },
        ..default()
    });

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

    println!("game spawn complete");
}

fn cleanup(
    mut game_config: ResMut<GameConfig>,
    mut game_state: ResMut<GameState>,
    mut controls: ResMut<Controls>,
) {
    println!("cleanup");

    println!("resetting controls");
    controls.reset();

    match game_state.state {
        PlayState::GameOver(GameOverState::Restart) => {
            game_state.state = PlayState::default();
        }
        PlayState::GameOver(GameOverState::Exit) => {
            *game_state = GameState::default();
            game_config.player_config = None;
        }
        _ => {
            panic!("Should not get here. {game_state:?} {game_config:?}");
        }
    }
}

fn stopwatch_restart(mut game_time: ResMut<GameTime>) {
    game_time.watch.reset();
    game_time.watch.unpause();
}

fn stopwatch(time: Res<Time>, mut game_time: ResMut<GameTime>) {
    game_time.watch.tick(time.delta());
}

fn spawn_to_game_transition(mut state: ResMut<NextState<AppState>>) {
    println!("game spawn to play transition");
    state.set(AppState::Game);
}
