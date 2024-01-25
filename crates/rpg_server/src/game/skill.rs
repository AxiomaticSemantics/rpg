use super::plugin::{AabbResources, GameSessionCleanup};

use rpg_core::skill::{
    effect::*, skill_tables::SkillTableEntry, AreaInstance, DirectInstance, OrbitData, Origin,
    ProjectileInstance, ProjectileShape, Skill, SkillInfo, SkillInstance,
};
use rpg_util::{
    actions::AttackData,
    skill::{Invulnerability, SkillContactEvent, SkillTimer, SkillUse, SkillUseBundle, Tickable},
    unit::{Corpse, Unit},
};

use util::{
    cleanup::CleanupStrategy,
    math::{intersect_aabb, Aabb, AabbComponent},
    random::SharedRng,
};

use bevy::{
    ecs::{
        entity::Entity,
        event::EventWriter,
        query::{With, Without},
        system::{Commands, Query},
    },
    math::Vec3,
    time::{Time, Timer, TimerMode},
    transform::{components::Transform, TransformBundle},
};

use std::borrow::Cow;

pub(crate) fn prepare_skill(
    owner: Entity,
    attack_data: &AttackData,
    time: &Time,
    random: &mut SharedRng,
    aabbs: &mut AabbResources,
    skill_info: &SkillTableEntry,
    skill: &Skill,
    unit: &Unit,
    unit_transform: &Transform,
) -> (Aabb, Transform, SkillUse) {
    let mut origin = match &skill_info.origin {
        Origin::Direct(_) => {
            unit_transform.translation
                + unit_transform.forward() * (skill_info.use_range as f32 / 100. / 2.)
        }
        Origin::Remote(data) => data.offset + attack_data.target,
        Origin::Locked(_) => unit_transform.translation,
    };

    //println!("{:?}", &skill_info.origin);

    let effects = skill
        .effects
        .iter()
        .map(|e| {
            let data = match e {
                EffectInfo::Pierce(_) => EffectData::Pierce(PierceData::default()),
                EffectInfo::Chain(_) => EffectData::Chain(ChainData::default()),
                EffectInfo::Split(_) => EffectData::Split(SplitData::default()),
                EffectInfo::Dot(_) => EffectData::Dot(DotData::default()),
                EffectInfo::Knockback(_) => EffectData::Knockback(KnockbackData::default()),
            };

            EffectInstance::new(e.clone(), data)
        })
        .collect();

    let (aabb, skill_use, transform, tickable) = match &skill_info.info {
        SkillInfo::Direct(_) => {
            origin.y = 1.2;

            let aabb = aabbs.aabbs["direct_attack"];
            let SkillInfo::Direct(info) = &skill.info else {
                panic!("Expected direct attack")
            };

            let instance = SkillInstance::Direct(DirectInstance {
                info: info.clone(),
                frame: 0,
            });

            let skill_transform =
                Transform::from_translation(origin).looking_to(unit_transform.forward(), Vec3::Y);

            (aabb, instance, skill_transform, None)
        }
        SkillInfo::Projectile(info) => {
            // debug!("spawn {speed} {duration} {size}");

            let tickable = info.tick_rate.as_ref().map(|tr| Tickable {
                timer: Timer::from_seconds(*tr, TimerMode::Repeating),
                can_damage: true,
            });

            let aabb = if info.shape == ProjectileShape::Box {
                let aabb = if aabbs.aabbs.contains_key("bolt_01") {
                    aabbs.aabbs["bolt_01"]
                } else {
                    let aabb =
                        Aabb::from_min_max(Vec3::new(-0.1, -0.1, -0.25), Vec3::new(0.1, 0.1, 0.25));
                    aabbs.aabbs.insert("bolt_01".into(), aabb);
                    aabb
                };

                aabb
            } else {
                // FIXME convert to sphere collider
                let radius = info.size as f32 / 100. / 2.;
                let key = format!("orb_radius_{}", info.size);

                let aabb = if aabbs.aabbs.contains_key(key.as_str()) {
                    aabbs.aabbs[key.as_str()]
                } else {
                    let aabb = Aabb::from_min_max(Vec3::splat(-radius), Vec3::splat(radius));

                    aabbs.aabbs.insert(Cow::Owned(key.as_str().into()), aabb);

                    aabb
                };

                aabb
            };

            let time = time.elapsed_seconds();
            let forward = unit_transform.forward();

            let spawn_transform = if info.orbit.is_some() {
                let mut rot_transform = *unit_transform;
                rot_transform.translation += Vec3::new(0., 1.2, 0.);
                rot_transform.rotate_local_y(std::f32::consts::TAU * (0.5 - random.f32()));

                Transform::from_translation(
                    rot_transform.translation + rot_transform.forward() * 2.,
                )
            } else if info.aerial.is_some() {
                //println!("prepare aerial {attack_data:?}");
                Transform::from_translation(origin).looking_at(attack_data.target, Vec3::Y)
            } else {
                Transform::from_translation(
                    unit_transform.translation + Vec3::new(0., 1.2, 0.) + forward * 0.25,
                )
                .looking_to(forward, Vec3::Y)
            };

            let instance_info = SkillInstance::Projectile(ProjectileInstance {
                info: info.clone(),
                start_time: time,
                orbit: if info.orbit.is_some() {
                    Some(OrbitData {
                        origin: spawn_transform.translation,
                    })
                } else {
                    None
                },
            });

            (aabb, instance_info, spawn_transform, tickable)
        }
        SkillInfo::Area(info) => {
            let skill_instance = SkillInstance::Area(AreaInstance {
                info: info.clone(),
                start_time: time.elapsed_seconds(),
            });

            let tickable = info.tick_rate.as_ref().map(|tr| Tickable {
                timer: Timer::from_seconds(*tr, TimerMode::Repeating),
                can_damage: true,
            });

            let transform = Transform::from_translation(origin + Vec3::new(0.0, 0.01, 0.0))
                .looking_to(Vec3::NEG_Y, Vec3::Y);

            let radius = info.radius as f32 / 100.;
            let key = format!("area_radius_{}", info.radius);

            let aabb = if aabbs.aabbs.contains_key(key.as_str()) {
                aabbs.aabbs[key.as_str()]
            } else {
                // 2d shapes are on the XY plane
                let aabb = Aabb::from_min_max(
                    Vec3::new(-radius, -radius, 0.0),
                    Vec3::new(radius, radius, 0.5),
                );

                aabbs.aabbs.insert(Cow::Owned(key.as_str().into()), aabb);

                aabb
            };

            (aabb, skill_instance, transform, tickable)
        }
    };

    let instance = SkillUse::new(
        owner,
        unit.kind,
        skill.id,
        skill.damage.clone(),
        skill_use,
        effects,
        tickable,
    );

    (aabb, transform, instance)
}

pub(crate) fn spawn_instance(
    commands: &mut Commands,
    aabb: Aabb,
    transform: Transform,
    skill_use_instance: SkillUse,
) {
    let skill_use = SkillUseBundle::new(skill_use_instance, SkillTimer(None));

    commands.spawn((
        GameSessionCleanup,
        CleanupStrategy::DespawnRecursive,
        AabbComponent(aabb),
        Invulnerability::default(),
        skill_use,
        TransformBundle::from(transform),
    ));
}

pub(crate) fn collide_skills(
    mut skill_events: EventWriter<SkillContactEvent>,
    mut skill_q: Query<(
        Entity,
        &Transform,
        &AabbComponent,
        &Invulnerability,
        &SkillUse,
    )>,
    unit_q: Query<(Entity, &Transform, &AabbComponent, &Unit), Without<Corpse>>,
) {
    for (s_entity, s_transform, s_aabb, invulnerability, instance) in &mut skill_q {
        if let Some(tickable) = &instance.tickable {
            if !tickable.can_damage {
                continue;
            }
        }

        for (u_entity, u_transform, u_aabb, unit) in &unit_q {
            if !unit.is_alive()
                || unit.kind == instance.owner_kind
                || u_entity == instance.owner
                || invulnerability.iter().any(|i| i.entity == u_entity)
            {
                continue;
            }

            /*println!(
                "unit {} skill {}",
                u_transform.translation, s_transform.translation
            );*/

            let unit_offset = Vec3::new(0.0, 1.2, 0.0);

            let collision = match &instance.instance {
                SkillInstance::Direct(_) | SkillInstance::Projectile(_) => intersect_aabb(
                    (
                        &(s_transform.translation),
                        &Aabb {
                            center: s_aabb.center,
                            half_extents: s_aabb.half_extents,
                        },
                    ),
                    (
                        &(u_transform.translation + unit_offset),
                        &Aabb {
                            center: u_aabb.center,
                            half_extents: u_aabb.half_extents,
                        },
                    ),
                ),
                SkillInstance::Area(info) => {
                    s_transform.translation.distance(u_transform.translation)
                        <= info.info.radius as f32 / 100.
                }
            };

            if collision {
                skill_events.send(SkillContactEvent {
                    entity: s_entity,
                    owner_entity: instance.owner,
                    defender_entity: u_entity,
                });
            }
        }
    }
}
