use crate::{
    game::{
        actor::{player::Player, spawn_actor},
        assets::RenderResources,
        metadata::MetadataResources,
    },
    net::account::RpgAccount,
    state::AppState,
};

use bevy::{
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        schedule::NextState,
        system::{Commands, Query, Res, ResMut},
    },
    log::{debug, info},
    math::Vec3,
    transform::components::Transform,
};

use rpg_core::{
    class::Class,
    unit::{UnitInfo, UnitKind},
};
use rpg_network_protocol::protocol::*;
use rpg_util::unit::{Unit, UnitBundle, Villain, VillainBundle};

use lightyear::client::events::MessageEvent;

pub(crate) fn receive_player_join_success(
    mut state: ResMut<NextState<AppState>>,
    mut join_events: EventReader<MessageEvent<SCPlayerJoinSuccess>>,
) {
    for event in join_events.read() {
        let join_msg = event.message();
        debug!("join success {join_msg:?}");

        state.set(AppState::GameSpawn);
        return;
    }
}

pub(crate) fn receive_player_join_error(
    mut state: ResMut<NextState<AppState>>,
    mut join_events: EventReader<MessageEvent<SCPlayerJoinError>>,
) {
    for event in join_events.read() {
        debug!("join error");
        // TODO Error screen

        state.set(AppState::Menu);
        return;
    }
}

pub(crate) fn receive_player_spawn(
    mut state: ResMut<NextState<AppState>>,
    mut spawn_events: EventReader<MessageEvent<SCPlayerSpawn>>,
    mut player_q: Query<(&mut Transform, &Unit), With<Player>>,
) {
    for event in spawn_events.read() {
        debug!("spawning");

        let spawn_msg = event.message();

        let (mut transform, player) = player_q.single_mut();

        transform.translation = spawn_msg.position;
        transform.look_to(spawn_msg.direction, Vec3::Y);

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

        info!("move: {move_msg:?}");
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

pub(crate) fn receive_stat_updates(
    mut update_events: EventReader<MessageEvent<SCStatUpdates>>,
    mut player_q: Query<&Unit, With<Player>>,
) {
    for event in update_events.read() {
        let update_msg = event.message();

        let mut player = player_q.single_mut();
        for update in &update_msg.updates {
            info!("stat update: {update:?}");
        }
    }
}

pub(crate) fn receive_villain_spawn(
    mut commands: Commands,
    mut spawn_reader: EventReader<MessageEvent<SCSpawnVillain>>,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
) {
    for event in spawn_reader.read() {
        let spawn_msg = event.message();

        info!("spawning villain {spawn_msg:?}");

        let unit = rpg_core::unit::Unit::new(
            spawn_msg.uid,
            spawn_msg.class,
            UnitKind::Villain,
            UnitInfo::Villain(spawn_msg.info.clone()),
            1,
            "Foo",
            None,
            &metadata.rpg,
        );

        let transform = Transform::from_translation(spawn_msg.position)
            .looking_to(spawn_msg.direction, Vec3::Y);
        spawn_actor(
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
