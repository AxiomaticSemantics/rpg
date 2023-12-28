use crate::game::{
    actions::{Action, ActionData, Actions, AttackData},
    actor::{
        actor::{
            get_hero_actor_key, ActorBasicBundle, ActorHandle, ActorKey, ActorMeshBundle,
            ActorSceneBundle,
        },
        unit::{CorpseTimer, Hero, Unit, UnitBundle, Villain},
    },
    assets::RenderResources,
    controls::{Controls, CursorPosition},
    health_bar::HealthBar,
    item::UnitStorage,
    metadata::MetadataResources,
    plugin::{GameCamera, GameConfig, GameSessionCleanup, GameState},
    skill::get_skill_origin,
    zone::zone::Zone,
};

use rpg_core::unit::{HeroInfo, UnitInfo, UnitKind};
use util::cleanup::CleanupStrategy;

use bevy::{
    ecs::prelude::*,
    gizmos::{gizmos::Gizmos, AabbGizmo},
    math::Vec3,
    pbr::SpotLight,
    render::{prelude::*, primitives::Aabb},
    scene::SceneBundle,
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
    utils::default,
};

#[derive(Component)]
pub(crate) struct Player;

#[derive(Bundle)]
pub(crate) struct PlayerBundle {
    pub player: Player,
    pub hero: Hero,
}

#[derive(Component)]
pub(crate) struct Nearest;

pub(crate) fn update_debug_lines(
    mut gizmos: Gizmos,
    player_q: Query<&Transform, (With<Player>, Without<Villain>)>,
    villain_q: Query<&Transform, (With<Villain>, Without<CorpseTimer>, Without<Player>)>,
) {
    let mut nearest = None::<&Transform>;
    let mut nearest_distance = 8.;

    let player_transform = player_q.single();

    for villain_transform in &villain_q {
        let distance = villain_transform
            .translation
            .distance(player_transform.translation);

        if distance < nearest_distance {
            nearest_distance = distance;
            nearest = Some(villain_transform);
        }
    }

    let Some(nearest) = nearest else {
        return;
    };

    gizmos.line(
        player_transform.translation,
        nearest.translation,
        Color::RED,
    );

    // println!("nearest {nearest_distance:?} {nearest:?}");
}

pub fn update_debug_gizmos(zone: Res<Zone>, mut gizmos: Gizmos) {
    gizmos.linestrip(
        zone.zone
            .curves
            .front()
            .unwrap()
            .iter()
            .map(|v| Vec3::new(-64. + v.x * 4. + 2., 0., -64. + v.z * 4. + 2.)),
        Color::RED,
    );
}

pub fn input_actions(
    controls: Res<Controls>,
    cursor_position: Res<CursorPosition>,
    metadata: Res<MetadataResources>,
    mut player_q: Query<(&Transform, &mut Actions, &Unit), With<Player>>,
) {
    let (transform, mut actions, unit) = player_q.single_mut();

    if controls.mouse_primary.pressed {
        let skill_id = unit.active_skills.primary.skill.unwrap();

        let (origin, target) =
            get_skill_origin(&metadata, &transform, cursor_position.ground, skill_id);

        if actions.attack.is_none() && actions.knockback.is_none() {
            actions.request(Action::new(
                ActionData::Attack(AttackData {
                    skill_id,
                    user: transform.translation,
                    origin,
                    target,
                }),
                None,
                true,
            ));

            // No other actions can happen while attacking
            return;
        }
    }

    if controls.mouse_secondary.pressed {
        actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
    } else if controls.mouse_secondary.just_released {
        actions.request(Action::new(ActionData::MoveEnd, None, true))
    }

    let look_point = cursor_position.ground;
    let target_dir = (look_point - transform.translation).normalize_or_zero();
    let rot_diff = transform.forward().dot(target_dir);
    if (rot_diff - 1.).abs() > 0.001 {
        //println!("rot_diff {rot_diff}");
        actions.request(Action::new(ActionData::Look(look_point), None, true));
    }

    // println!("actions: {actions:?} controls: {controls:?}");
}

pub(crate) fn update_spotlight(
    player_q: Query<&Transform, (With<Player>, Without<SpotLight>)>,
    mut spotlight_q: Query<&mut Transform, (With<SpotLight>, Without<Player>)>,
) {
    let player_transform = player_q.single();

    let mut spotlight = spotlight_q.single_mut();
    let target = player_transform.translation + Vec3::new(0., 6., 8.);
    if spotlight.translation != target {
        spotlight.translation = target;
        spotlight.look_at(
            player_transform.translation + Vec3::new(0., 0., 0.),
            Vec3::Y,
        );
    }
}

pub(crate) fn update_camera(
    time: Res<Time>,
    controls: Res<Controls>,
    player_q: Query<&Transform, With<Player>>,
    mut camera_q: Query<(&mut Transform, &mut GameCamera), Without<Player>>,
) {
    let player_transform = player_q.single();

    let (mut camera_transform, mut game_camera) = camera_q.single_mut();
    // println!("wheel {} {camera_transform:?} {player_transform:?}", controls.mouse_wheel);

    if controls.mouse_wheel_delta != 0. {
        game_camera.offset.y -= controls.mouse_wheel_delta * time.delta_seconds();
        game_camera.offset.y = game_camera
            .offset
            .y
            .clamp(game_camera.min_y, game_camera.max_y);
        game_camera.offset.z = game_camera.offset.y * 0.55;
    }

    camera_transform.translation = player_transform.translation + game_camera.offset;
    camera_transform.look_at(player_transform.translation, Vec3::Y);
}

pub(crate) fn spawn_player(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    game_config: Res<GameConfig>,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
) {
    println!("spawn_player");

    let player_config = &game_config.player_config.as_ref().unwrap();

    let mut unit = rpg_core::unit::Unit::new(
        game_state.next_uid.0,
        player_config.class,
        UnitKind::Hero,
        UnitInfo::Hero(HeroInfo::new(&metadata.rpg)),
        1,
        player_config.name.clone(),
        None,
        &metadata.rpg,
    );

    // FIXME remove after testing
    unit.passive_skill_points = 10;

    game_state.next_uid.next();

    unit.add_default_skills(&metadata.rpg);

    let body_aabb = Aabb::from_min_max(Vec3::new(-0.3, 0., -0.25), Vec3::new(0.3, 1.8, 0.25));

    let mut inv_timer = Timer::from_seconds(0.25, TimerMode::Once);
    inv_timer.pause();

    let actor_key = get_hero_actor_key(unit.class);
    let (actor, actor_key) = (
        renderables.actors[actor_key].actor.clone(),
        ActorKey(actor_key),
    );

    let bar = HealthBar::spawn_bars(&mut commands, &renderables, Transform::default());

    let actor_bundle = match actor {
        ActorHandle::Mesh(handle) => {
            todo!()
        }
        ActorHandle::Scene(handle) => ActorSceneBundle {
            basic: ActorBasicBundle {
                health_bar: HealthBar::new(bar, 0.8),
                actor_key,
                aabb: body_aabb,
                ..default()
            },
            scene: SceneBundle {
                scene: handle,
                ..default()
            },
        },
    };

    commands.spawn((
        GameSessionCleanup,
        CleanupStrategy::DespawnRecursive,
        PlayerBundle {
            player: Player,
            hero: Hero,
        },
        UnitBundle::new(Unit(unit)),
        UnitStorage::default(),
        actor_bundle,
        //AabbGizmo::default(),
    ));
}
