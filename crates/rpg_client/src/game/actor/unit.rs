#![allow(clippy::too_many_arguments)]

use crate::{
    assets::AudioAssets,
    game::{
        actor::{
            animation::{AnimationState, ANIM_ATTACK, ANIM_IDLE},
            player::Player,
        },
        health_bar::{HealthBar, HealthBarFrame, HealthBarRect},
        metadata::MetadataResources,
        plugin::{GameCamera, GameState},
    },
};

use audio_manager::plugin::AudioActions;
use rpg_core::{
    skill::{SkillInfo, SkillUseResult},
    storage::Storage,
};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::{ActionData, Actions, State},
    item::{GroundItem, StorableItem, UnitStorage},
    skill::{SkillSlots, Skills},
    unit::{Corpse, Hero, Unit},
};

use util::random::SharedRng;

use bevy::{
    audio::{AudioBundle, PlaybackSettings},
    ecs::{
        entity::Entity,
        query::{Changed, With, Without},
        system::{Commands, ParamSet, Query, Res, ResMut},
    },
    hierarchy::{BuildChildren, Children, DespawnRecursiveExt},
    input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
    math::Vec3,
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};

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

            transform.look_to(*camera_forward, Vec3::Y);

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

            commands.entity(i_entity).despawn_recursive();
            return;
        }
    }
}

pub fn action(
    mut net_client: ResMut<Client>,
    time: Res<Time>,
    metadata: Res<MetadataResources>,
    mut rng: ResMut<SharedRng>,
    mut unit_q: Query<
        (
            &Unit,
            &Skills,
            &SkillSlots,
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

    for (
        unit,
        skills,
        skill_slots,
        mut transform,
        mut actions,
        mut anim_state,
        mut audio_actions,
    ) in &mut unit_q
    {
        // debug!("action request {:?}", action.request);

        if let Some(action) = &mut actions.knockback {
            let ActionData::Knockback(knockback) = &action.data else {
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
            let ActionData::Attack(attack) = &action.data else {
                panic!("expected attack data");
            };

            match &mut action.state {
                State::Pending => {
                    let distance =
                        (attack.user.distance(attack.skill_target.target) * 100.).round() as u32;
                    match unit.can_use_skill(&skills, &metadata.rpg, attack.skill_id, distance) {
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

                    let skill_id = skill_slots.slots[0].skill_id.unwrap();
                    let Some(skill_info) = metadata.rpg.skill.skills.get(&skill_id) else {
                        panic!("skill metadata not found");
                    };

                    let duration = skill_info.use_duration_secs
                        * unit.stats.vitals.stats["Cooldown"].value.f32();

                    *anim_state = ANIM_ATTACK;

                    let sound_key = match skill_info.info {
                        SkillInfo::Direct(_) => match rng.usize(0..2) {
                            0 => "attack_proj1",
                            _ => "attack_proj2",
                        },
                        SkillInfo::Projectile(_) => match rng.usize(0..2) {
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
                    action.state = State::Finalize;
                    action.timer = None;
                }
                State::Finalize => {
                    *anim_state = ANIM_IDLE;
                    action.state = State::Completed;
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

            net_client.send_message::<Channel1, _>(CSRotPlayer(*wanted.forward()));

            action.state = State::Completed;
        }

        /*
        if let Some(action) = &mut actions.movement {
            let movespeed = unit.get_effective_movement_speed() as f32 / 100. * dt;
            let translation = transform.forward() * movespeed;
            transform.translation += translation;

            *anim_state = AnimationState {
                repeat: RepeatAnimation::Forever,
                paused: false,
                index: 3,
            };

            action.state = State::Completed;
        }
        */
    }
}

pub fn remove_healthbar(
    mut commands: Commands,
    unit_q: Query<(Entity, &Unit, &HealthBar), (With<AudioActions>, With<Corpse>)>,
) {
    for (entity, _unit, health_bar) in &unit_q {
        commands.entity(entity).remove::<AudioActions>();
        commands.entity(health_bar.bar_entity).despawn_recursive();
    }
}
