use crate::{
    game::{
        actor,
        assets::RenderResources,
        controls::{Controls, CursorPosition},
        environment::PlayerSpotLight,
        metadata::MetadataResources,
        plugin::{GameCamera, GameState},
        world::zone::Zone,
    },
    net::account::RpgAccount,
};

use rpg_core::{
    passive_tree::PassiveSkillGraph,
    storage::UnitStorage as RpgUnitStorage,
    unit::{HeroInfo, Unit as RpgUnit, UnitInfo, UnitKind},
};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::{Action, ActionData, Actions, AttackData},
    skill::*,
    unit::{Hero, Unit, Villain},
};

use bevy::{
    ecs::{
        bundle::Bundle,
        change_detection::DetectChanges,
        component::Component,
        event::EventReader,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::gizmos::Gizmos,
    log::info,
    math::{Vec2, Vec3},
    pbr::SpotLight,
    render::color::Color,
    time::Time,
    transform::components::Transform,
};

/// Marker to denote the local player in the client
#[derive(Component)]
pub struct Player;

#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub hero: Hero,
}

#[derive(Component)]
pub struct Nearest;

pub fn update_debug_lines(
    mut gizmos: Gizmos,
    player_q: Query<&Transform, (With<Player>, Without<Villain>)>,
    villain_q: Query<&Transform, (With<Villain>, Without<Player>)>,
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

    // debug!("nearest {nearest_distance:?} {nearest:?}");
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
    mut net_client: ResMut<Client>,
    controls: Res<Controls>,
    cursor_position: Res<CursorPosition>,
    metadata: Res<MetadataResources>,
    mut player_q: Query<(&Transform, &mut Actions, &Unit), With<Player>>,
) {
    if controls.is_inhibited() {
        return;
    }

    let (transform, mut actions, unit) = player_q.single_mut();

    if controls.mouse_primary.pressed || controls.gamepad_b.pressed {
        let skill_id = unit.active_skills.primary.skill.unwrap();

        net_client.send_message::<Channel1, _>(CSSkillUseDirect(skill_id));

        let (origin, target) =
            get_skill_origin(&metadata.rpg, transform, cursor_position.ground, skill_id);

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

            // No other user actions can happen while attacking
            return;
        }
    }

    if controls.mouse_secondary.pressed || controls.gamepad_a.pressed {
        actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
        net_client.send_message::<Channel1, _>(CSMovePlayer);
    } else if controls.mouse_secondary.just_released || controls.gamepad_a.just_released {
        actions.request(Action::new(ActionData::MoveEnd, None, true));
    }

    /*if controls.gamepad_axis_left != Vec2::ZERO {
        let atan = controls
            .gamepad_axis_left
            .x
            .atan2(-controls.gamepad_axis_left.y);
        let sc = atan.sin_cos();
        //println!("atan {atan} f sc_f {sc:?} {}", transform.forward());

        Some(Vec3::new(sc.0, 0., sc.1))
    }*/
    let target_dir = if cursor_position.is_changed() {
        Some((cursor_position.ground - transform.translation).normalize_or_zero())
    } else {
        None
    };

    if let Some(target_dir) = target_dir {
        let rot_diff = transform.forward().dot(target_dir);
        if (rot_diff - 1.).abs() > 0.001 {
            //println!("rot_diff {rot_diff}");
            actions.request(Action::new(ActionData::LookDir(target_dir), None, true));
            net_client.send_message::<Channel1, _>(CSRotPlayer(target_dir));
        }
    }

    // debug!("actions: {actions:?} controls: {controls:?}");
}

pub fn update_spotlight(
    player_q: Query<&Transform, (With<Player>, Without<SpotLight>)>,
    mut spotlight_q: Query<
        &mut Transform,
        (With<PlayerSpotLight>, With<SpotLight>, Without<Player>),
    >,
) {
    let player_transform = player_q.single();

    let mut spotlight_transform = spotlight_q.single_mut();
    let target = player_transform.translation + Vec3::new(0., 6., 8.);
    if spotlight_transform.translation != target {
        spotlight_transform.translation = target;
        spotlight_transform.look_at(
            player_transform.translation + Vec3::new(0., 1.2, 0.),
            Vec3::Y,
        );
    }
}

pub fn update_camera(
    time: Res<Time>,
    controls: Res<Controls>,
    player_q: Query<&Transform, With<Player>>,
    mut camera_q: Query<(&mut Transform, &mut GameCamera), Without<Player>>,
) {
    if controls.is_inhibited() {
        return;
    }

    let player_transform = player_q.single();

    let (mut camera_transform, mut game_camera) = camera_q.single_mut();

    let delta = if controls.mouse_wheel_delta != 0. {
        controls.mouse_wheel_delta * time.delta_seconds()
    } else if controls.gamepad_lt_a.pressed {
        4. * time.delta_seconds()
    } else if controls.gamepad_lt_b.pressed {
        -4. * time.delta_seconds()
    } else {
        0.
    };

    if delta != 0. {
        game_camera.offset.y =
            (game_camera.offset.y - delta).clamp(game_camera.min_y, game_camera.max_y);
    }

    let wanted_z = game_camera.offset.y * 0.55;
    if (wanted_z - game_camera.offset.z).abs() > 0.001 {
        game_camera.offset.z = wanted_z;
    }

    let camera_position = player_transform.translation + game_camera.offset;
    if camera_transform.translation != camera_position {
        camera_transform.translation = camera_position;
        camera_transform.look_at(player_transform.translation, Vec3::Y);
    }
}

pub fn spawn_player(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
    account_q: Query<&RpgAccount>,
) {
    info!("spawning local player");

    let account = account_q.single();

    game_state.mode = account.0.characters[0].info.game_mode;

    let (unit, storage, passive_tree) = {
        (
            account.0.characters[0].character.unit.clone(),
            account.0.characters[0].character.storage.clone(),
            account.0.characters[0].character.passive_tree.clone(),
        )
    };

    actor::spawn_actor(
        &mut commands,
        &metadata,
        &renderables,
        unit,
        Some(storage),
        Some(passive_tree),
        None,
    );
}
