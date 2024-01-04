use crate::game::{
    actions::{Action, ActionData, Actions, AttackData},
    actor::{
        self,
        unit::{CorpseTimer, Hero, Unit},
        villain::Villain,
    },
    assets::RenderResources,
    controls::{Controls, CursorPosition},
    metadata::MetadataResources,
    plugin::{GameCamera, GameState},
    skill::get_skill_origin,
    state_saver::{LoadCharacter, SaveSlots},
    world::zone::Zone,
};

use rpg_core::{
    passive_tree::PassiveSkillGraph,
    storage::UnitStorage as RpgUnitStorage,
    unit::{HeroInfo, Unit as RpgUnit, UnitInfo, UnitKind},
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
    math::{Vec2, Vec3},
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
    if controls.is_inhibited() {
        return;
    }

    let (transform, mut actions, unit) = player_q.single_mut();

    if controls.mouse_primary.pressed || controls.gamepad_b.pressed {
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

    if controls.mouse_secondary.pressed || controls.gamepad_a.pressed {
        actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
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
        let look_point = cursor_position.ground;
        Some((look_point - transform.translation).normalize_or_zero())
    } else {
        None
    };

    if let Some(target_dir) = target_dir {
        let rot_diff = transform.forward().dot(target_dir);
        if (rot_diff - 1.).abs() > 0.001 {
            //println!("rot_diff {rot_diff}");
            actions.request(Action::new(ActionData::LookDir(target_dir), None, true));
        }
    }

    // println!("actions: {actions:?} controls: {controls:?}");
}

pub fn update_spotlight(
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
    save_slots: Res<SaveSlots>,
    mut load_event: EventReader<LoadCharacter>,
) {
    println!("spawn_player");

    let player_config = &game_state.player_config.as_ref().unwrap();

    let (unit, storage, passive_tree) = if !load_event.is_empty() {
        let slot_id = load_event.read().last().unwrap();

        let slot = &save_slots.slots[slot_id.0 as usize];
        let state = slot.state.as_ref().unwrap();
        (
            state.unit.clone(),
            state.storage.clone(),
            state.passive_tree.clone(),
        )
    } else {
        let mut unit = RpgUnit::new(
            game_state.next_uid.0,
            player_config.class,
            UnitKind::Hero,
            UnitInfo::Hero(HeroInfo::new(&metadata.rpg, player_config.game_mode)),
            1,
            player_config.name.clone(),
            None,
            &metadata.rpg,
        );

        // FIXME remove after testing
        unit.passive_skill_points = 10;

        game_state.next_uid.next();

        unit.add_default_skills(&metadata.rpg);

        let class = unit.class;
        (
            unit,
            RpgUnitStorage::default(),
            PassiveSkillGraph::new(class),
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
