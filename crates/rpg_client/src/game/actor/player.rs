use crate::game::{
    actions::{Action, ActionData, Actions, AttackData},
    actor::{
        self,
        unit::{CorpseTimer, Hero, Unit, Villain},
    },
    assets::RenderResources,
    controls::{Controls, CursorPosition},
    metadata::MetadataResources,
    plugin::{GameCamera, GameState},
    skill::get_skill_origin,
    world::zone::Zone,
};

use rpg_core::unit::{HeroInfo, Unit as RpgUnit, UnitInfo, UnitKind};

use bevy::{
    ecs::{
        bundle::Bundle,
        component::Component,
        query::{With, Without},
        system::{Commands, Query, Res, ResMut},
    },
    gizmos::gizmos::Gizmos,
    math::Vec3,
    pbr::SpotLight,
    render::color::Color,
    time::Time,
    transform::components::Transform,
};

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
            get_skill_origin(&metadata, transform, cursor_position.ground, skill_id);

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

    if controls.mouse_secondary.pressed {
        actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
    } else if controls.mouse_secondary.just_released {
        actions.request(Action::new(ActionData::MoveEnd, None, true));
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
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
) {
    println!("spawn_player");

    let player_config = &game_state.player_config.as_ref().unwrap();

    let mut unit = RpgUnit::new(
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

    actor::spawn_actor(&mut commands, &metadata, &renderables, unit, None);
}
