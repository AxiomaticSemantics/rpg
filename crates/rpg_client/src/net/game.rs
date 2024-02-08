use crate::{
    game::{
        actor::{
            self,
            animation::{
                AnimationState, ANIM_ATTACK, ANIM_DEATH, ANIM_DEFEND, ANIM_IDLE, ANIM_WALK,
            },
            player::Player,
            spawn_actor,
        },
        assets::RenderResources,
        metadata::MetadataResources,
        plugin::GameState,
        skill,
    },
    net::account::RpgAccount,
    state::AppState,
};

use bevy::{
    asset::Assets,
    ecs::{
        entity::Entity,
        event::EventReader,
        query::{With, Without},
        schedule::NextState,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    log::{debug, info},
    math::Vec3,
    render::mesh::Mesh,
    transform::components::Transform,
};

use rpg_core::{
    combat::CombatResult,
    passive_tree::UnitPassiveSkills,
    skill::SkillInfo,
    stat::{Stat, StatId},
    storage::UnitStorage,
    unit::{HeroInfo, UnitInfo, UnitKind},
    value::Value,
};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    item::{GroundItem, GroundItemDrops},
    skill::{SkillSlots, SkillUse, Skills},
    unit::{Corpse, Hero, Unit, Villain},
};

use audio_manager::plugin::AudioActions;
use util::random::SharedRng;

use lightyear::client::events::MessageEvent;

pub(crate) fn receive_player_join_success(
    mut state: ResMut<NextState<AppState>>,
    mut join_events: EventReader<MessageEvent<SCPlayerJoinSuccess>>,
) {
    for event in join_events.read() {
        let join_msg = event.message();
        info!("player joined game {join_msg:?}");

        state.set(AppState::GameSpawn);

        join_events.clear();
        return;
    }
}

pub(crate) fn receive_player_join_error(
    mut state: ResMut<NextState<AppState>>,
    mut join_events: EventReader<MessageEvent<SCPlayerJoinError>>,
) {
    for _ in join_events.read() {
        info!("join error");
        // TODO Error screen

        state.set(AppState::GameCleanup);

        join_events.clear();
        return;
    }
}

pub(crate) fn receive_player_spawn(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    renderables: Res<RenderResources>,
    mut spawn_events: EventReader<MessageEvent<SCPlayerSpawn>>,
    account_q: Query<(Entity, &RpgAccount)>,
) {
    for event in spawn_events.read() {
        info!("spawning local player");

        let spawn_msg = event.message();

        let (entity, account) = account_q.single();

        let transform = Transform::from_translation(spawn_msg.position);

        let character_record = account
            .0
            .get_character_from_slot(account.0.info.selected_slot.unwrap())
            .unwrap();

        let (unit, skills, active, storage, passive_tree) = {
            (
                character_record.character.unit.clone(),
                character_record.character.skills.clone(),
                character_record.character.skill_slots.clone(),
                character_record.character.storage.clone(),
                character_record.character.passive_tree.clone(),
            )
        };

        actor::spawn_actor(
            entity,
            &mut commands,
            &renderables,
            transform,
            unit,
            Skills(skills),
            SkillSlots::new(active),
            Some(storage),
            Some(passive_tree),
        );

        state.set(AppState::Game);

        spawn_events.clear();
        return;
    }
}

pub(crate) fn receive_player_move(
    mut move_events: EventReader<MessageEvent<SCMovePlayer>>,
    mut player_q: Query<(&mut Transform, &mut AnimationState), With<Player>>,
) {
    for event in move_events.read() {
        let move_msg = event.message();

        // info!("move player {move_msg:?}");
        let (mut transform, mut anim) = player_q.single_mut();
        transform.translation = move_msg.0;

        if *anim != ANIM_WALK {
            *anim = ANIM_WALK;
        }
    }
}

pub(crate) fn receive_player_move_end(
    mut move_events: EventReader<MessageEvent<SCMovePlayerEnd>>,
    mut player_q: Query<(&mut Transform, &mut AnimationState), With<Player>>,
) {
    for event in move_events.read() {
        let move_msg = event.message();

        //info!("move player end {move_msg:?}");
        let (mut transform, mut anim) = player_q.single_mut();
        transform.translation = move_msg.0;

        *anim = ANIM_IDLE;
    }
}

pub(crate) fn receive_player_rotation(
    mut rotation_events: EventReader<MessageEvent<SCRotPlayer>>,
    mut player_q: Query<&mut Transform, With<Player>>,
) {
    for event in rotation_events.read() {
        let rot_msg = event.message();

        // info!("rot: {rot_msg:?}");
        let mut transform = player_q.single_mut();
        transform.look_to(rot_msg.0, Vec3::Y);
    }
}

pub(crate) fn receive_unit_move(
    mut move_events: EventReader<MessageEvent<SCMoveUnit>>,
    mut unit_q: Query<(&mut Transform, &Unit, &mut AnimationState)>,
) {
    for event in move_events.read() {
        let move_msg = event.message();
        for (mut transform, unit, mut anim) in &mut unit_q {
            if unit.uid != move_msg.uid {
                continue;
            }

            //info!("move: {move_msg:?}");
            *anim = ANIM_WALK;

            transform.translation = move_msg.position;
        }
    }
}

pub(crate) fn receive_unit_move_end(
    mut move_events: EventReader<MessageEvent<SCMoveUnitEnd>>,
    mut unit_q: Query<(&mut Transform, &Unit, &mut AnimationState)>,
) {
    for event in move_events.read() {
        let move_msg = event.message();
        for (mut transform, unit, mut anim) in &mut unit_q {
            if unit.uid != move_msg.uid {
                continue;
            }

            //info!("move unit end: {move_msg:?}");
            *anim = ANIM_IDLE;

            transform.translation = move_msg.position;
        }
    }

    move_events.clear();
}

pub(crate) fn receive_unit_rotation(
    mut rotation_events: EventReader<MessageEvent<SCRotUnit>>,
    mut unit_q: Query<(&mut Transform, &Unit)>,
) {
    for event in rotation_events.read() {
        let rot_msg = event.message();
        for (mut transform, unit) in &mut unit_q {
            if unit.uid != rot_msg.uid {
                continue;
            }

            // info!("rot unit: {rot_msg:?}");
            transform.look_to(rot_msg.direction, Vec3::Y);
        }
    }
}

pub(crate) fn receive_stat_update(
    mut update_events: EventReader<MessageEvent<SCStatUpdate>>,
    mut player_q: Query<&mut Unit, With<Player>>,
) {
    for event in update_events.read() {
        let update_msg = event.message();

        // info!("player stat update: {:?}", update_msg.0);

        let mut player = player_q.single_mut();
        player
            .stats
            .vitals
            .set_from_id(update_msg.0.id, update_msg.0.total);
    }
}

pub(crate) fn receive_stat_updates(
    mut update_events: EventReader<MessageEvent<SCStatUpdates>>,
    mut player_q: Query<&mut Unit, With<Player>>,
) {
    for event in update_events.read() {
        let update_msg = event.message();

        let mut player = player_q.single_mut();
        for update in &update_msg.0 {
            player.stats.vitals.set_from_id(update.id, update.total);
            // info!("stat update: {update:?}");
        }
    }
}

pub(crate) fn receive_spawn_item(
    mut ground_items: ResMut<GroundItemDrops>,
    mut spawn_reader: EventReader<MessageEvent<SCSpawnItem>>,
) {
    for event in spawn_reader.read() {
        let spawn_msg = event.message();

        info!("spawning item: {:?}", spawn_msg);

        ground_items.0.push(spawn_msg.items.clone());
    }
}

pub(crate) fn receive_spawn_items(
    mut ground_items: ResMut<GroundItemDrops>,
    mut spawn_reader: EventReader<MessageEvent<SCSpawnItems>>,
) {
    for event in spawn_reader.read() {
        let spawn_msg = event.message();

        info!("spawning items: {:?}", spawn_msg);

        ground_items.0.push(spawn_msg.items.clone());
    }
}

pub(crate) fn receive_despawn_item(
    mut commands: Commands,
    mut despawn_reader: EventReader<MessageEvent<SCDespawnItem>>,
    item_q: Query<(Entity, &GroundItem)>,
) {
    for event in despawn_reader.read() {
        let despawn_msg = event.message();

        for (entity, item) in &item_q {
            if item.0.uid != despawn_msg.0 {
                continue;
            }

            commands.entity(entity).despawn_recursive();
            info!("ground item despawn: {despawn_msg:?}");
        }
    }
}

pub(crate) fn receive_spawn_skill(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut renderables: ResMut<RenderResources>,
    metadata: Res<MetadataResources>,
    mut spawn_reader: EventReader<MessageEvent<SCSpawnSkill>>,
) {
    for event in spawn_reader.read() {
        let spawn_msg = event.message();
        let skill_id = spawn_msg.id;

        info!("spawning skill: {spawn_msg:?}");

        let skill_meta = &metadata.rpg.skill.skills[&skill_id];

        let (aabb, transform, instance, mesh, material, timer) = skill::prepare_skill(
            spawn_msg.uid,
            &spawn_msg.target,
            &mut renderables,
            &mut meshes,
            skill_meta,
            spawn_msg.id,
        );

        skill::spawn_instance(
            &mut commands,
            aabb,
            transform,
            instance,
            mesh,
            material,
            timer,
        );
    }
}

// TODO need to correlate skill instances between client and server
pub(crate) fn receive_despawn_skill(
    mut commands: Commands,
    mut despawn_reader: EventReader<MessageEvent<SCDespawnSkill>>,
    skill_q: Query<Entity, With<SkillUse>>,
) {
    for event in despawn_reader.read() {
        let despawn_msg = event.message();
        for entity in &skill_q {
            // TODO
            // commands.entity(despawn_msg.0);
        }
    }
}

pub(crate) fn receive_spawn_hero(
    mut commands: Commands,
    mut spawn_reader: EventReader<MessageEvent<SCSpawnHero>>,
    metadata: Res<MetadataResources>,
    game_state: Res<GameState>,
    renderables: Res<RenderResources>,
) {
    for event in spawn_reader.read() {
        let spawn_msg = event.message();

        info!("spawning hero {spawn_msg:?}");

        let unit = rpg_core::unit::Unit::new(
            spawn_msg.uid,
            spawn_msg.class,
            UnitKind::Hero,
            UnitInfo::Hero(HeroInfo {
                game_mode: game_state.mode,
                xp_curr: Stat {
                    id: StatId(23),
                    value: Value::U64(0),
                },
            }),
            spawn_msg.level,
            spawn_msg.name.clone(),
            &metadata.rpg,
        );

        let transform = Transform::from_translation(spawn_msg.position);
        let skills = Skills(spawn_msg.skills.clone());
        let skill_slots = SkillSlots::new(spawn_msg.skill_slots.clone());

        // FIXME storage and skill graph are dummy data here
        spawn_actor(
            Entity::PLACEHOLDER,
            &mut commands,
            &renderables,
            transform,
            unit,
            skills,
            skill_slots,
            Some(UnitStorage::default()),
            Some(UnitPassiveSkills::new(spawn_msg.class)),
        );
    }
}

pub(crate) fn receive_spawn_villain(
    mut commands: Commands,
    mut spawn_reader: EventReader<MessageEvent<SCSpawnVillain>>,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
) {
    for event in spawn_reader.read() {
        let spawn_msg = event.message();

        info!("spawning villain {spawn_msg:?}");

        let villain_meta = &metadata.rpg.unit.villains[&spawn_msg.info.id];

        // TODO
        // - ensure a server-side villain is the same as one generated by the based on
        //   the message data
        let unit = rpg_core::unit::Unit::new(
            spawn_msg.uid,
            villain_meta.class,
            UnitKind::Villain,
            UnitInfo::Villain(spawn_msg.info.clone()),
            spawn_msg.level,
            villain_meta.name.clone(),
            &metadata.rpg,
        );

        let skills = Skills(spawn_msg.skills.clone());
        let skill_slots = SkillSlots::new(spawn_msg.skill_slots.clone());

        let transform = Transform::from_translation(spawn_msg.position)
            .looking_to(spawn_msg.direction, Vec3::Y);
        spawn_actor(
            Entity::PLACEHOLDER,
            &mut commands,
            &renderables,
            transform,
            unit,
            skills,
            skill_slots,
            None,
            None,
        );
    }
}

pub(crate) fn receive_combat_result(
    metadata: Res<MetadataResources>,
    mut combat_reader: EventReader<MessageEvent<SCCombatResult>>,
    mut player_q: Query<(&mut Unit, &mut AudioActions, &mut AnimationState), With<Player>>,
) {
    for event in combat_reader.read() {
        let combat_msg = event.message();

        let (mut player, mut audio, mut anim) = player_q.single_mut();
        match &combat_msg.0 {
            CombatResult::Damage(damage) => {
                player.stats.vitals.set("Hp", Value::U32(damage.total));
                *anim = ANIM_DEFEND;
                audio.push("hit_soft".into());
            }
            CombatResult::HeroDeath(_) => {
                player.stats.vitals.set("Hp", Value::U32(0));
                audio.push("hit_death".into());
                *anim = ANIM_DEATH;
            }
            CombatResult::VillainDeath(death) => {
                if let Some(reward) = &death.reward {
                    player.info.hero_mut().xp_curr.value = reward.xp_total;
                    if let Some(level) = &reward.level {
                        player.level = level.level;
                        player.passive_skill_points = level.passive_points;
                    }
                }
            }
            CombatResult::Blocked | CombatResult::Dodged => {
                audio.push("hit_blocked".into());
                *anim = ANIM_DEFEND;
            }
            CombatResult::Error => debug!("combat error received!?"),
        }

        info!("combat result {combat_msg:?}");
    }
}

pub(crate) fn receive_damage(
    mut combat_reader: EventReader<MessageEvent<SCDamage>>,
    mut unit_q: Query<(&mut AnimationState, &mut AudioActions, &mut Unit)>,
) {
    for event in combat_reader.read() {
        let combat_msg = event.message();

        for (mut anim, mut audio, mut unit) in &mut unit_q {
            if unit.uid != combat_msg.uid {
                continue;
            }

            *anim = ANIM_DEFEND;
            audio.push("hit_soft".into());

            let hp = &mut unit.stats.vitals.stats.get_mut("Hp").unwrap();
            let hp_ref = hp.value.u32_mut();
            *hp_ref = combat_msg.damage.total;
        }
        info!("combat result {combat_msg:?}");
    }
}

pub(crate) fn receive_unit_attack(
    mut rng: ResMut<SharedRng>,
    metadata: Res<MetadataResources>,
    mut attack_reader: EventReader<MessageEvent<SCUnitAttack>>,
    mut unit_q: Query<(&mut AnimationState, &mut AudioActions, &Unit)>,
) {
    for event in attack_reader.read() {
        let attack_msg = event.message();

        for (mut anim, mut audio, unit) in &mut unit_q {
            if unit.uid != attack_msg.uid {
                continue;
            }

            let skill_info = &metadata.rpg.skill.skills[&attack_msg.skill_id];
            let audio_key = match skill_info.info {
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
            audio.push(audio_key.into());
            *anim = ANIM_ATTACK;
        }
    }
}

pub(crate) fn receive_unit_anim(
    mut anim_reader: EventReader<MessageEvent<SCUnitAnim>>,
    mut unit_q: Query<(&mut AnimationState, &mut AudioActions, &Unit)>,
) {
    for event in anim_reader.read() {
        let anim_msg = event.message();

        for (mut anim, mut audio, unit) in &mut unit_q {
            if unit.uid != anim_msg.uid {
                continue;
            }

            match anim_msg.anim {
                0 => {
                    *anim = ANIM_DEFEND;
                    audio.push("hit_blocked".into());
                }
                1 => {
                    *anim = ANIM_DEFEND;
                    audio.push("hit_blocked".into());
                }
                2 => {
                    *anim = ANIM_IDLE;
                }
                id => {
                    info!("unhandled anim {id}");
                }
            }
        }
    }
}

pub(crate) fn receive_villain_death(
    mut commands: Commands,
    mut death_reader: EventReader<MessageEvent<SCVillainDeath>>,
    mut villain_q: Query<(Entity, &Unit, &mut AudioActions, &mut AnimationState), With<Villain>>,
) {
    for event in death_reader.read() {
        let death_msg = event.message();

        info!("villain death {death_msg:?}");

        for (entity, villain, mut villain_audio, mut villain_anim) in &mut villain_q {
            if villain.uid != death_msg.0 {
                continue;
            }

            commands.entity(entity).insert(Corpse);

            *villain_anim = ANIM_DEATH;
            villain_audio.push("hit_death".into());
        }
    }
}

pub(crate) fn receive_hero_death(
    mut commands: Commands,
    mut death_reader: EventReader<MessageEvent<SCHeroDeath>>,
    mut hero_q: Query<(Entity, &Unit, &mut AudioActions, &mut AnimationState), With<Hero>>,
) {
    for event in death_reader.read() {
        let death_msg = event.message();

        info!("hero death {death_msg:?}");
        let (entity, unit, mut audio, mut anim) = hero_q.single_mut();
        audio.push("hit_death".into());
        *anim = ANIM_DEATH;
        if unit.uid == death_msg.0 {
            //
        }

        commands.entity(entity).insert(Corpse);
        death_reader.clear();
        return;
    }
}

pub(crate) fn receive_despawn_corpse(
    mut commands: Commands,
    mut despawn_reader: EventReader<MessageEvent<SCDespawnCorpse>>,
    unit_q: Query<(Entity, &Unit), With<Corpse>>,
) {
    for event in despawn_reader.read() {
        let despawn_msg = event.message();

        info!("despawning corpse {despawn_msg:?}");
        for (entity, unit) in &unit_q {
            if unit.uid != despawn_msg.0 {
                continue;
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}

pub(crate) fn recieve_item_pickup(
    mut pickup_reader: EventReader<MessageEvent<SCDespawnCorpse>>,
    unit_q: Query<(Entity, &Unit), Without<Corpse>>,
) {
}
