#![allow(clippy::too_many_arguments)]

use crate::game::{
    actor::{animation::AnimationState, player::Player},
    assets::RenderResources,
    health_bar::{HealthBar, HealthBarFrame, HealthBarRect},
    item::UnitStorage,
    metadata::MetadataResources,
    plugin::{GameCamera, GameState},
    skill,
};

use audio_manager::plugin::AudioActions;
use rpg_core::{
    skill::{SkillInfo, SkillUseResult},
    storage::Storage,
    unit::UnitKind,
};
use rpg_util::{
    actions::{ActionData, Actions, State},
    item::{GroundItem, ResourceItem, StorableItem},
    unit::{Corpse, Hero, Unit, UnitBundle, Villain},
};

use util::{
    math::{intersect_aabb, Aabb as UtilAabb, AabbComponent},
    random::SharedRng,
};

use bevy::{
    animation::RepeatAnimation,
    asset::Assets,
    ecs::{
        bundle::Bundle,
        component::Component,
        entity::Entity,
        query::{With, Without},
        system::{Commands, ParamSet, Query, Res, ResMut},
    },
    hierarchy::{Children, DespawnRecursiveExt},
    input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
    math::Vec3,
    prelude::{Deref, DerefMut},
    render::mesh::Mesh,
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};

pub fn update_health_bars(
    mut unit_q: Query<
        (&Unit, &Transform, &mut HealthBar),
        (
            Without<Corpse>,
            Without<GameCamera>,
            Without<HealthBarFrame>,
            Without<HealthBarRect>,
        ),
    >,
    camera_q: Query<&Transform, (With<GameCamera>, Without<Unit>)>,
    mut bar_set: ParamSet<(
        Query<
            (&mut Transform, &Children),
            (
                With<HealthBarFrame>,
                Without<Unit>,
                Without<GameCamera>,
                Without<HealthBarRect>,
            ),
        >,
        Query<
            &mut Transform,
            (
                With<HealthBarRect>,
                Without<Unit>,
                Without<GameCamera>,
                Without<HealthBarFrame>,
            ),
        >,
    )>,
) {
    let camera_forward = camera_q.single().forward();

    for (unit, unit_transform, mut health_bar) in &mut unit_q {
        let bar = {
            let mut frame_q = bar_set.p0();
            let (mut transform, children) = frame_q.get_mut(health_bar.bar_entity).unwrap();
            let target = unit_transform.translation + Vec3::Y * 2.4;
            if transform.translation != target {
                transform.translation = target;
            }

            transform.look_to(camera_forward, Vec3::Y);

            *children.first().unwrap()
        };

        let mut changed = false;
        if unit.stats.vitals.stats["Hp"].value != health_bar.curr {
            health_bar.curr = *unit.stats.vitals.stats["Hp"].value.u32();

            changed = true;
        };

        if unit.stats.vitals.stats["HpMax"].value != health_bar.max {
            health_bar.max = *unit.stats.vitals.stats["HpMax"].value.u32();

            changed = true;
        }

        if changed {
            let mut bar_q = bar_set.p1();
            let mut bar_transform = bar_q.get_mut(bar).unwrap();
            let scale_x = health_bar.curr as f32 / health_bar.max as f32;
            //println!("updating unit bar: scale_x {scale_x}");
            bar_transform.scale.x = scale_x;
            bar_transform.translation.x = -0.375 + scale_x * 0.375;
        }
    }
}

// TODO move this to somewhere else
pub fn pick_storable_items(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameState>,
    mut item_q: Query<(Entity, &Transform, &mut GroundItem), With<StorableItem>>,
    mut hero_q: Query<
        (&Transform, &mut UnitStorage, &mut AudioActions),
        (
            With<Hero>,
            With<Player>,
            Without<GroundItem>,
            Without<Corpse>,
        ),
    >,
) {
    let Ok((u_transform, mut u_storage, mut u_audio)) = hero_q.get_single_mut() else {
        return;
    };

    for (i_entity, i_transform, mut i_item) in &mut item_q {
        let mut i_ground = i_transform.translation;
        i_ground.y = 0.;

        let distance = u_transform.translation.distance(i_ground);
        if key_input.pressed(KeyCode::ControlLeft)
            && (key_input.just_pressed(KeyCode::Space)
                || mouse_input.just_pressed(MouseButton::Left))
            && distance < 0.5
        {
            let Some(slot) = u_storage.get_empty_slot_mut() else {
                return;
            };

            slot.item = i_item.0.take();

            u_audio.push("item_pickup".into());
            game_state.session_stats.items_looted += 1;

            commands.entity(i_entity).despawn_recursive();
            return;
        }
    }
}

// NOTE some desync is acceptible here
pub(crate) fn attract_resource_items(
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
    metadata: Res<MetadataResources>,
    mut item_q: Query<(Entity, &mut Transform, &mut GroundItem), With<ResourceItem>>,
    mut hero_q: Query<
        (&Transform, &mut Unit, &mut AudioActions),
        (With<Hero>, Without<GroundItem>, Without<Corpse>),
    >,
) {
    let Ok((u_transform, mut unit, mut u_audio)) = hero_q.get_single_mut() else {
        return;
    };

    for (i_entity, mut i_transform, mut i_item) in &mut item_q {
        let mut i_ground = i_transform.translation;
        i_ground.y = 0.;

        let pickup_radius = *unit.stats.vitals.stats["PickupRadius"].value.u32() as f32 / 100.;

        let distance = u_transform.translation.distance(i_ground);
        if distance > pickup_radius {
            continue;
        } else if distance < 0.25 {
            //let item = i_item.0.take().unwrap();
            //let _leveled_up = unit.apply_rewards(&metadata.rpg, &item);
            u_audio.push("item_pickup".into());
            game_state.session_stats.items_looted += 1;

            //commands.entity(i_entity).despawn_recursive();
        } else {
            let target_dir = (u_transform.translation - i_ground).normalize_or_zero();
            i_transform.translation += target_dir * time.delta_seconds() * 4.;
        }
    }
}

pub fn action(
    mut commands: Commands,
    time: Res<Time>,
    metadata: Res<MetadataResources>,
    mut renderables: ResMut<RenderResources>,
    mut random: ResMut<SharedRng>,
    mut state: ResMut<GameState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut unit_q: Query<
        (
            Entity,
            &mut Unit,
            &mut Transform,
            &mut Actions,
            &mut AnimationState,
            &mut AudioActions,
        ),
        Without<Corpse>,
    >,
) {
    use std::f32::consts;

    let dt = time.delta_seconds();

    for (entity, mut unit, mut transform, mut actions, mut anim_state, mut audio_actions) in
        &mut unit_q
    {
        // debug!("action request {:?}", action.request);

        if let Some(action) = &mut actions.knockback {
            let ActionData::Knockback(knockback) = action.data else {
                panic!("expected knockback data");
            };

            if time.elapsed_seconds() < knockback.start + knockback.duration {
                let target =
                    -transform.forward() * time.delta_seconds() * (knockback.speed as f32 / 100.);
                transform.translation += target;
            } else {
                action.state = State::Completed;
            }

            continue;
        }

        if let Some(action) = &mut actions.attack {
            let ActionData::Attack(attack) = action.data else {
                panic!("expected attack data");
            };

            match &mut action.state {
                State::Pending => {
                    let distance = (attack.user.distance(attack.target) * 100.).round() as u32;
                    match unit.can_use_skill(&metadata.rpg, attack.skill_id, distance) {
                        SkillUseResult::Blocked
                        | SkillUseResult::OutOfRange
                        | SkillUseResult::InsufficientResources => {
                            action.state = State::Completed;
                            //println!("skill use blocked {:?}", unit.skills);
                            continue;
                        }
                        SkillUseResult::Ok => {}
                        SkillUseResult::Error => {
                            panic!("Skill use error");
                        }
                    }

                    let skill_id = unit.active_skills.primary.skill.unwrap();
                    let Some(skill_info) = metadata.rpg.skill.skills.get(&skill_id) else {
                        panic!("skill metadata not found");
                    };

                    let duration = skill_info.use_duration_secs
                        * unit.stats.vitals.stats["Cooldown"].value.f32();

                    *anim_state = AnimationState {
                        repeat: RepeatAnimation::Never,
                        paused: false,
                        index: 0,
                    };

                    let sound_key = match skill_info.info {
                        SkillInfo::Direct(_) => match random.usize(0..2) {
                            0 => "attack_proj1",
                            _ => "attack_proj2",
                        },
                        SkillInfo::Projectile(_) => match random.usize(0..2) {
                            0 => "attack_proj1",
                            _ => "attack_proj2",
                        },
                        SkillInfo::Area(_) => "attack_proj1",
                    };

                    audio_actions.push(sound_key.into());

                    action.timer = Some(Timer::from_seconds(duration, TimerMode::Once));
                    action.state = State::Timer;
                }
                State::Active => {
                    let distance = (attack.user.distance(attack.target) * 100.).round() as u32;
                    let skill_use_result = unit.use_skill(&metadata.rpg, attack.skill_id, distance);
                    match skill_use_result {
                        SkillUseResult::Ok => {}
                        _ => panic!("This should never happen. {skill_use_result:?}"),
                    }

                    let Some(skill) = unit.skills.iter().find(|s| s.id == attack.skill_id) else {
                        panic!("skill missing");
                    };
                    let Some(skill_info) = metadata.rpg.skill.skills.get(&attack.skill_id) else {
                        panic!("skill metadata not found");
                    };

                    if unit.kind == UnitKind::Hero {
                        state.session_stats.attacks += 1;
                    } else {
                        state.session_stats.villain_attacks += 1;
                    }

                    let (skill_aabb, skill_transform, skill_use, mesh_handle, material_handle) =
                        skill::prepare_skill(
                            entity,
                            &attack,
                            &time,
                            &mut random,
                            &mut renderables,
                            &mut meshes,
                            skill_info,
                            skill,
                            &unit,
                            &transform,
                        );

                    skill::spawn_instance(
                        &mut commands,
                        skill_aabb,
                        skill_transform,
                        skill_use,
                        mesh_handle,
                        material_handle,
                    );

                    action.state = State::Completed;
                    action.timer = None;
                }
                _ => {}
            }
        }

        if let Some(action) = &mut actions.look {
            let wanted = if let ActionData::LookPoint(target) = action.data {
                transform.looking_at(target, Vec3::Y)
            } else if let ActionData::LookDir(dir) = action.data {
                transform.looking_to(dir, Vec3::Y)
            } else {
                panic!("Invalid action data");
            };

            let diff = transform.rotation.angle_between(wanted.rotation);
            let speed = consts::TAU * 1.33;
            let ratio = (speed * dt) / diff;

            let lerped = transform
                .rotation
                .slerp(wanted.rotation, ratio.clamp(0., 1.));
            if transform.rotation != lerped {
                transform.rotation = lerped;
            }

            action.state = State::Completed;
        }

        if let Some(action) = &mut actions.movement {
            let movespeed = unit.get_effective_movement_speed() as f32 / 100. * dt;
            if unit.can_run() {
                unit.stats.consume_stamina(dt);
            }

            let translation = transform.forward() * movespeed;
            transform.translation += translation;

            *anim_state = AnimationState {
                repeat: RepeatAnimation::Forever,
                paused: false,
                index: 3,
            };

            action.state = State::Completed;
        }

        if let Some(action) = &mut actions.movement_end {
            *anim_state = AnimationState {
                repeat: RepeatAnimation::Forever,
                paused: true,
                index: 3,
            };

            action.state = State::Completed;
        }
    }
}

pub fn remove_healthbar(
    mut commands: Commands,
    time: Res<Time>,
    mut unit_q: Query<(Entity, &mut Corpse, &mut HealthBar), With<Unit>>,
) {
    for (entity, mut timer, mut health_bar) in &mut unit_q {
        if health_bar.bar_entity != Entity::PLACEHOLDER {
            commands.entity(health_bar.bar_entity).despawn_recursive();
            health_bar.bar_entity = Entity::PLACEHOLDER;
        }
    }
}
