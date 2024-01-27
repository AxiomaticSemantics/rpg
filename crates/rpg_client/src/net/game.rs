use crate::{
    game::{
        actor::{self, animation::AnimationState, player::Player, spawn_actor},
        assets::RenderResources,
        metadata::MetadataResources,
    },
    net::account::RpgAccount,
    state::AppState,
};

use bevy::{
    animation::RepeatAnimation,
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        query::With,
        schedule::NextState,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    log::{debug, info},
    math::Vec3,
    transform::components::Transform,
};

use rpg_core::{
    class::Class,
    stat::StatChange,
    unit::{UnitInfo, UnitKind},
};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    item::GroundItemDrops,
    unit::{Corpse, Unit, UnitBundle, Villain, VillainBundle},
};

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
    for event in join_events.read() {
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
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
    mut spawn_events: EventReader<MessageEvent<SCPlayerSpawn>>,
    mut account_q: Query<(Entity, &mut RpgAccount)>,
) {
    for event in spawn_events.read() {
        info!("spawning");

        let spawn_msg = event.message();

        let (entity, mut account) = account_q.single_mut();

        let transform = Transform::from_translation(spawn_msg.position)
            .looking_to(spawn_msg.direction, Vec3::Y);

        let character_record = account
            .0
            .get_character_from_slot(account.0.info.selected_slot.unwrap())
            .unwrap();

        let (unit, storage, passive_tree) = {
            (
                character_record.character.unit.clone(),
                character_record.character.storage.clone(),
                character_record.character.passive_tree.clone(),
            )
        };

        actor::spawn_actor(
            entity,
            &mut commands,
            &metadata,
            &renderables,
            unit,
            Some(storage),
            Some(passive_tree),
            None,
        );

        state.set(AppState::Game);

        spawn_events.clear();
        return;
    }
}

pub(crate) fn receive_player_move(
    mut move_events: EventReader<MessageEvent<SCMovePlayer>>,
    mut player_q: Query<(&mut Transform, &Unit), With<Player>>,
) {
    for event in move_events.read() {
        let move_msg = event.message();

        // info!("move: {move_msg:?}");
        let (mut transform, player) = player_q.single_mut();
        transform.translation = move_msg.0;
    }
}

pub(crate) fn receive_player_move_end(
    mut move_events: EventReader<MessageEvent<SCMovePlayerEnd>>,
    mut player_q: Query<(&mut Transform, &Unit), With<Player>>,
) {
    for event in move_events.read() {
        let move_msg = event.message();

        // info!("move: {move_msg:?}");
        let (mut transform, player) = player_q.single_mut();
        transform.translation = move_msg.0;
    }
}

pub(crate) fn receive_player_rotation(
    mut rotation_events: EventReader<MessageEvent<SCRotPlayer>>,
    mut player_q: Query<(&mut Transform, &Unit), With<Player>>,
) {
    for event in rotation_events.read() {
        let rot_msg = event.message();

        // info!("rot: {rot_msg:?}");
        let (mut transform, player) = player_q.single_mut();
        transform.look_to(rot_msg.0, Vec3::Y);
    }
}

pub(crate) fn receive_stat_update(
    mut update_events: EventReader<MessageEvent<SCStatUpdate>>,
    mut player_q: Query<&mut Unit, With<Player>>,
) {
    for event in update_events.read() {
        let update_msg = event.message();

        info!("stat update: {:?}", update_msg.0);

        let mut player = player_q.single_mut();

        player
            .stats
            .vitals
            .get_mut_stat_from_id(update_msg.0.id)
            .unwrap()
            .value = update_msg.0.total;
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
            match &update.change {
                StatChange::Gain(v) => {}
                StatChange::Loss(v) => {}
            }

            player
                .stats
                .vitals
                .get_mut_stat_from_id(update.id)
                .unwrap()
                .value = update.total;
            info!("stat update: {update:?}");
        }
    }
}

pub(crate) fn receive_spawn_item(
    mut spawn_reader: EventReader<MessageEvent<SCSpawnItem>>,
    mut player_q: Query<&Unit, With<Player>>,
) {
    for event in spawn_reader.read() {
        let spawn_msg = event.message();

        info!("item drop: {:?}", spawn_msg.0);
    }
}

pub(crate) fn receive_spawn_items(
    mut spawn_reader: EventReader<MessageEvent<SCSpawnItems>>,
    mut player_q: Query<&Unit, With<Player>>,
) {
    for event in spawn_reader.read() {
        let spawn_msg = event.message();

        info!("item drops: {:?}", spawn_msg.0);
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
            None,
            &metadata.rpg,
        );

        let transform = Transform::from_translation(spawn_msg.position)
            .looking_to(spawn_msg.direction, Vec3::Y);
        spawn_actor(
            Entity::PLACEHOLDER,
            &mut commands,
            &metadata,
            &renderables,
            unit,
            None,
            None,
            Some(transform),
        );
    }
}

pub(crate) fn receive_villain_death(
    mut ground_items: ResMut<GroundItemDrops>,
    mut death_reader: EventReader<MessageEvent<SCVillainDeath>>,
    mut villain_q: Query<(&Unit, &mut AnimationState), With<Villain>>,
) {
    for event in death_reader.read() {
        let death_msg = event.message();

        info!("villain eath {death_msg:?}");

        for (villain, mut villain_anim) in &mut villain_q {
            if villain.uid != death_msg.uid {
                continue;
            }

            *villain_anim = AnimationState {
                repeat: RepeatAnimation::Never,
                paused: false,
                index: 1,
            };
        }

        ground_items.0.push(death_msg.drops.clone());
    }
}

pub(crate) fn receive_hero_death(mut death_reader: EventReader<MessageEvent<SCHeroDeath>>) {
    for event in death_reader.read() {
        let death_msg = event.message();

        info!("hero death {death_msg:?}");
    }
}

pub(crate) fn receive_despawn_corpse(
    mut commands: Commands,
    mut despawn_reader: EventReader<MessageEvent<SCDespawnCorpse>>,
    mut unit_q: Query<(Entity, &Unit), With<Corpse>>,
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
