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
        controls::Controls,
        health_bar::{HealthBar, HealthBarFrame},
        metadata::MetadataResources,
        plugin::GameState,
        skill,
        world::LoadZone,
    },
    net::account::RpgAccount,
    state::AppState,
};

use bevy::{
    asset::Assets,
    ecs::{
        entity::Entity,
        event::{EventReader, EventWriter},
        query::{With, Without},
        schedule::NextState,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    log::{debug, info},
    math::Vec3,
    render::{mesh::Mesh, view::Visibility},
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

pub(crate) fn receive_player_join_success(mut join_events: EventReader<ServerMessage>) {
    for event in join_events.read() {
        let ServerMessage::SCPlayerJoinSuccess(msg) = event else {
            continue;
        };

        info!("player joined game {msg:?}");

        return;
    }
}

pub(crate) fn receive_player_join_error(
    mut state: ResMut<NextState<AppState>>,
    mut join_events: EventReader<ServerMessage>,
) {
    for event in join_events.read() {
        let ServerMessage::SCPlayerJoinError(_) = event else {
            continue;
        };

        info!("join error");
        // TODO Error screen

        state.set(AppState::GameCleanup);

        return;
    }
}

pub(crate) fn receive_player_spawn(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    renderables: Res<RenderResources>,
    mut spawn_events: EventReader<ServerMessage>,
    account_q: Query<&RpgAccount>,
) {
    for event in spawn_events.read() {
        let ServerMessage::SCPlayerSpawn(msg) = event else {
            continue;
        };

        info!("spawning local player");

        let account = account_q.single();

        let transform = Transform::from_translation(msg.position);

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
            Entity::PLACEHOLDER,
            true,
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

pub(crate) fn receive_player_revive(
    mut commands: Commands,
    mut controls: ResMut<Controls>,
    mut revive_reader: EventReader<ServerMessage>,
    mut player_q: Query<
        (
            Entity,
            &mut Transform,
            &mut Unit,
            &mut AnimationState,
            &HealthBar,
        ),
        With<Player>,
    >,
    mut bar_q: Query<&mut Visibility, With<HealthBarFrame>>,
) {
    for event in revive_reader.read() {
        let ServerMessage::SCPlayerRevive(msg) = event else {
            continue;
        };

        info!("revive player {msg:?}");
        let (entity, mut transform, mut unit, mut anim, health_bar) = player_q.single_mut();
        transform.translation = msg.position;
        let hero_info = unit.info.hero_mut();
        hero_info.deaths = Some(msg.deaths);

        // Reset the units vitals stats on revive
        unit.stats.vitals.get_mut_stat("Hp").unwrap().value =
            unit.stats.vitals.get_stat("HpMax").unwrap().value;
        unit.stats.vitals.get_mut_stat("Ep").unwrap().value =
            unit.stats.vitals.get_stat("EpMax").unwrap().value;
        unit.stats.vitals.get_mut_stat("Mp").unwrap().value =
            unit.stats.vitals.get_stat("MpMax").unwrap().value;

        controls.set_inhibited(false);

        let mut bar = bar_q.get_mut(health_bar.bar_entity).unwrap();
        *bar = Visibility::Inherited;

        commands.entity(entity).remove::<Corpse>();

        *anim = ANIM_WALK;
    }
}

pub(crate) fn receive_hero_revive(
    mut commands: Commands,
    mut revive_reader: EventReader<ServerMessage>,
    mut player_q: Query<(Entity, &mut Transform, &mut AnimationState, &HealthBar), With<Hero>>,
    mut bar_q: Query<&mut Visibility, With<HealthBarFrame>>,
) {
    for event in revive_reader.read() {
        let ServerMessage::SCHeroRevive(msg) = event else {
            continue;
        };

        info!("revive hero {msg:?}");
        let (entity, mut transform, mut anim, health_bar) = player_q.single_mut();
        transform.translation = msg.0;

        let mut bar = bar_q.get_mut(health_bar.bar_entity).unwrap();
        *bar = Visibility::Inherited;

        *anim = ANIM_IDLE;

        commands.entity(entity).remove::<Corpse>();
    }
}

pub(crate) fn receive_player_move(
    mut move_events: EventReader<ServerMessage>,
    mut player_q: Query<(&mut Transform, &mut AnimationState), With<Player>>,
) {
    for event in move_events.read() {
        let ServerMessage::SCMovePlayer(msg) = event else {
            continue;
        };

        // info!("move player {msg:?}");
        let (mut transform, mut anim) = player_q.single_mut();
        transform.translation = msg.0;

        if *anim != ANIM_WALK {
            *anim = ANIM_WALK;
        }
    }
}

pub(crate) fn receive_player_move_end(
    mut move_events: EventReader<ServerMessage>,
    mut player_q: Query<(&mut Transform, &mut AnimationState), With<Player>>,
) {
    for event in move_events.read() {
        let ServerMessage::SCMovePlayerEnd(msg) = event else {
            continue;
        };

        //info!("move player end {msg:?}");
        let (mut transform, mut anim) = player_q.single_mut();
        transform.translation = msg.0;

        *anim = ANIM_IDLE;
    }
}

pub(crate) fn receive_player_rotation(
    mut rotation_events: EventReader<ServerMessage>,
    mut player_q: Query<&mut Transform, With<Player>>,
) {
    for event in rotation_events.read() {
        let ServerMessage::SCRotPlayer(msg) = event else {
            continue;
        };

        // info!("rot: {msg:?}");
        let mut transform = player_q.single_mut();
        transform.look_to(msg.0, Vec3::Y);
    }
}

pub(crate) fn receive_unit_move(
    mut move_events: EventReader<ServerMessage>,
    mut unit_q: Query<(&mut Transform, &Unit, &mut AnimationState)>,
) {
    for event in move_events.read() {
        let ServerMessage::SCMoveUnit(msg) = event else {
            continue;
        };

        for (mut transform, unit, mut anim) in &mut unit_q {
            if unit.uid != msg.uid {
                continue;
            }

            //info!("move: {msg:?}");
            *anim = ANIM_WALK;

            transform.translation = msg.position;
        }
    }
}

pub(crate) fn receive_unit_move_end(
    mut move_events: EventReader<ServerMessage>,
    mut unit_q: Query<(&mut Transform, &Unit, &mut AnimationState)>,
) {
    for event in move_events.read() {
        let ServerMessage::SCMoveUnitEnd(msg) = event else {
            continue;
        };

        for (mut transform, unit, mut anim) in &mut unit_q {
            if unit.uid != msg.uid {
                continue;
            }

            //info!("move unit end: {msg:?}");
            *anim = ANIM_IDLE;

            transform.translation = msg.position;
        }
    }
}

pub(crate) fn receive_unit_rotation(
    mut rotation_events: EventReader<ServerMessage>,
    mut unit_q: Query<(&mut Transform, &Unit)>,
) {
    for event in rotation_events.read() {
        let ServerMessage::SCRotUnit(msg) = event else {
            continue;
        };

        for (mut transform, unit) in &mut unit_q {
            if unit.uid != msg.uid {
                continue;
            }

            // info!("rot unit: {msg:?}");
            transform.look_to(msg.direction, Vec3::Y);
        }
    }
}

pub(crate) fn receive_stat_update(
    mut update_events: EventReader<ServerMessage>,
    mut player_q: Query<&mut Unit, With<Player>>,
) {
    for event in update_events.read() {
        let ServerMessage::SCStatUpdate(msg) = event else {
            continue;
        };

        // info!("player stat update: {:?}", msg.0);

        let mut player = player_q.single_mut();
        player.stats.vitals.set_from_id(msg.0.id, msg.0.total);
    }
}

pub(crate) fn receive_stat_updates(
    mut update_events: EventReader<ServerMessage>,
    mut player_q: Query<&mut Unit, With<Player>>,
) {
    for event in update_events.read() {
        let ServerMessage::SCStatUpdates(msg) = event else {
            continue;
        };

        let mut player = player_q.single_mut();
        for update in &msg.0 {
            player.stats.vitals.set_from_id(update.id, update.total);
            // info!("stat update: {update:?}");
        }
    }
}

pub(crate) fn receive_spawn_item(
    mut ground_items: ResMut<GroundItemDrops>,
    mut spawn_reader: EventReader<ServerMessage>,
) {
    for event in spawn_reader.read() {
        let ServerMessage::SCSpawnItem(msg) = event else {
            continue;
        };

        info!("spawning item: {msg:?}");

        ground_items.0.push(msg.items.clone());
    }
}

pub(crate) fn receive_spawn_items(
    mut ground_items: ResMut<GroundItemDrops>,
    mut spawn_reader: EventReader<ServerMessage>,
) {
    for event in spawn_reader.read() {
        let ServerMessage::SCSpawnItems(msg) = event else {
            continue;
        };

        info!("spawning items: {msg:?}");

        ground_items.0.push(msg.items.clone());
    }
}

pub(crate) fn receive_despawn_item(
    mut commands: Commands,
    mut despawn_reader: EventReader<ServerMessage>,
    item_q: Query<(Entity, &GroundItem)>,
) {
    for event in despawn_reader.read() {
        let ServerMessage::SCDespawnItem(msg) = event else {
            continue;
        };

        for (entity, item) in &item_q {
            if item.0.uid != msg.0 {
                continue;
            }

            commands.entity(entity).despawn_recursive();
            info!("ground item despawn: {msg:?}");
        }
    }
}

pub(crate) fn receive_spawn_skill(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut renderables: ResMut<RenderResources>,
    metadata: Res<MetadataResources>,
    mut spawn_reader: EventReader<ServerMessage>,
) {
    for event in spawn_reader.read() {
        let ServerMessage::SCSpawnSkill(msg) = event else {
            continue;
        };

        let skill_id = msg.id;
        let skill_meta = &metadata.rpg.skill.skills[&skill_id];

        let (aabb, transform, instance, mesh, material, timer) = skill::prepare_skill(
            msg.instance_uid,
            msg.owner_uid,
            &msg.target,
            &mut renderables,
            &mut meshes,
            skill_meta,
            skill_id,
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

        info!("spawning skill: {msg:?}");
    }
}

// TODO need to correlate skill instances between client and server
pub(crate) fn receive_despawn_skill(
    mut commands: Commands,
    mut despawn_reader: EventReader<ServerMessage>,
    skill_q: Query<(Entity, &SkillUse)>,
) {
    for event in despawn_reader.read() {
        let ServerMessage::SCDespawnSkill(msg) = event else {
            continue;
        };

        for (entity, skill_use) in &skill_q {
            if skill_use.instance_uid != msg.0 {
                continue;
            }

            // TODO
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub(crate) fn receive_spawn_hero(
    mut commands: Commands,
    mut spawn_reader: EventReader<ServerMessage>,
    metadata: Res<MetadataResources>,
    game_state: Res<GameState>,
    renderables: Res<RenderResources>,
) {
    for event in spawn_reader.read() {
        let ServerMessage::SCSpawnHero(msg) = event else {
            continue;
        };

        info!("spawning hero {msg:?}");

        let unit = rpg_core::unit::Unit::new(
            msg.uid,
            msg.class,
            UnitKind::Hero,
            UnitInfo::Hero(HeroInfo {
                game_mode: game_state.mode,
                xp_curr: Stat {
                    id: StatId(23),
                    value: Value::U64(0),
                },
                deaths: msg.deaths,
            }),
            msg.level,
            msg.name.clone(),
            &metadata.rpg,
        );

        let transform = Transform::from_translation(msg.position);
        let skills = Skills(msg.skills.clone());
        let skill_slots = SkillSlots::new(msg.skill_slots.clone());

        // FIXME storage and skill graph are dummy data here
        spawn_actor(
            Entity::PLACEHOLDER,
            false,
            &mut commands,
            &renderables,
            transform,
            unit,
            skills,
            skill_slots,
            Some(UnitStorage::default()),
            Some(UnitPassiveSkills::new(msg.class)),
        );
    }
}

pub(crate) fn receive_spawn_villain(
    mut commands: Commands,
    mut spawn_reader: EventReader<ServerMessage>,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
) {
    for event in spawn_reader.read() {
        let ServerMessage::SCSpawnVillain(msg) = event else {
            continue;
        };

        info!("spawning villain {msg:?}");

        let villain_meta = &metadata.rpg.unit.villains[&msg.info.id];

        // TODO
        // - ensure a server-side villain is the same as one generated by the based on
        //   the message data
        let unit = rpg_core::unit::Unit::new(
            msg.uid,
            villain_meta.class,
            UnitKind::Villain,
            UnitInfo::Villain(msg.info.clone()),
            msg.level,
            villain_meta.name.clone(),
            &metadata.rpg,
        );

        let skills = Skills(msg.skills.clone());
        let skill_slots = SkillSlots::new(msg.skill_slots.clone());

        let transform =
            Transform::from_translation(msg.position).looking_to(msg.direction, Vec3::Y);
        spawn_actor(
            Entity::PLACEHOLDER,
            false,
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
    mut combat_reader: EventReader<ServerMessage>,
    mut player_q: Query<(&mut Unit, &mut AudioActions, &mut AnimationState), With<Player>>,
) {
    for event in combat_reader.read() {
        let ServerMessage::SCCombatResult(msg) = event else {
            continue;
        };

        let (mut player, mut audio, mut anim) = player_q.single_mut();
        match &msg.0 {
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

        info!("combat result {msg:?}");
    }
}

pub(crate) fn receive_damage(
    mut combat_reader: EventReader<ServerMessage>,
    mut unit_q: Query<(&mut AnimationState, &mut AudioActions, &mut Unit)>,
) {
    for event in combat_reader.read() {
        let ServerMessage::SCDamage(msg) = event else {
            continue;
        };

        for (mut anim, mut audio, mut unit) in &mut unit_q {
            if unit.uid != msg.uid {
                continue;
            }

            *anim = ANIM_DEFEND;
            audio.push("hit_soft".into());

            let hp = &mut unit.stats.vitals.stats.get_mut("Hp").unwrap();
            let hp_ref = hp.value.u32_mut();
            *hp_ref = msg.damage.total;
        }
        info!("combat result {msg:?}");
    }
}

pub(crate) fn receive_unit_attack(
    mut rng: ResMut<SharedRng>,
    metadata: Res<MetadataResources>,
    mut attack_reader: EventReader<ServerMessage>,
    mut unit_q: Query<(&mut AnimationState, &mut AudioActions, &Unit)>,
) {
    for event in attack_reader.read() {
        let ServerMessage::SCUnitAttack(msg) = event else {
            continue;
        };

        for (mut anim, mut audio, unit) in &mut unit_q {
            if unit.uid != msg.uid {
                continue;
            }

            let skill_info = &metadata.rpg.skill.skills[&msg.skill_id];
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
    mut anim_reader: EventReader<ServerMessage>,
    mut unit_q: Query<(&mut AnimationState, &mut AudioActions, &Unit)>,
) {
    for event in anim_reader.read() {
        let ServerMessage::SCUnitAnim(msg) = event else {
            continue;
        };

        for (mut anim, mut audio, unit) in &mut unit_q {
            if unit.uid != msg.uid {
                continue;
            }

            match msg.anim {
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
    mut death_reader: EventReader<ServerMessage>,
    mut villain_q: Query<(Entity, &Unit, &mut AudioActions, &mut AnimationState), With<Villain>>,
) {
    for event in death_reader.read() {
        let ServerMessage::SCVillainDeath(msg) = event else {
            continue;
        };

        info!("villain death {msg:?}");

        for (entity, villain, mut villain_audio, mut villain_anim) in &mut villain_q {
            if villain.uid != msg.0 {
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
    mut death_reader: EventReader<ServerMessage>,
    mut hero_q: Query<(Entity, &Unit, &mut AudioActions, &mut AnimationState), With<Hero>>,
) {
    for event in death_reader.read() {
        let ServerMessage::SCHeroDeath(msg) = event else {
            continue;
        };

        info!("hero death {msg:?}");
        let (entity, unit, mut audio, mut anim) = hero_q.single_mut();
        audio.push("hit_death".into());
        *anim = ANIM_DEATH;

        commands.entity(entity).insert(Corpse);
        death_reader.clear();
        return;
    }
}

pub(crate) fn receive_despawn_corpse(
    mut commands: Commands,
    mut despawn_reader: EventReader<ServerMessage>,
    unit_q: Query<(Entity, &Unit), With<Corpse>>,
) {
    for event in despawn_reader.read() {
        let ServerMessage::SCDespawnCorpse(msg) = event else {
            continue;
        };

        info!("despawning corpse {msg:?}");
        for (entity, unit) in &unit_q {
            if unit.uid != msg.0 {
                continue;
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}

pub(crate) fn receive_item_pickup(
    mut pickup_reader: EventReader<ServerMessage>,
    unit_q: Query<(Entity, &Unit), Without<Corpse>>,
) {
    // SCItemPickup
}

pub(crate) fn receive_item_drop(
    mut drop_reader: EventReader<ServerMessage>,
    unit_q: Query<(Entity, &Unit), Without<Corpse>>,
) {
    // SCItemDrop
}

pub(crate) fn receive_item_store(
    mut store_reader: EventReader<ServerMessage>,
    unit_q: Query<(Entity, &Unit), Without<Corpse>>,
) {
    // SCItemStore
}

pub(crate) fn receive_zone_load(
    mut load_reader: EventReader<ServerMessage>,
    mut load_zone_writer: EventWriter<LoadZone>,
) {
    for event in load_reader.read() {
        let ServerMessage::SCZoneLoad(msg) = event else {
            continue;
        };

        info!("load zone");

        load_zone_writer.send(LoadZone(msg.0));

        return;
    }
}

pub(crate) fn receive_zone_unload(mut unload_reader: EventReader<ServerMessage>) {
    for event in unload_reader.read() {
        // SCZoneUnload
    }
}
