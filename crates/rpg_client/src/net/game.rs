use crate::{game::actor::player::Player, net::account::RpgAccount, state::AppState};

use bevy::{
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        schedule::NextState,
        system::{Query, ResMut},
    },
    log::debug,
    math::Vec3,
    transform::components::Transform,
};

use rpg_network_protocol::protocol::*;
use rpg_util::unit::Unit;

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

        let (mut transform, player) = player_q.single_mut();
        transform.look_to(rot_msg.0, Vec3::Y);
    }
}
