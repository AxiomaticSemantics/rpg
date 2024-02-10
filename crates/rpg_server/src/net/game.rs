use super::server::{NetworkParamsRO, NetworkParamsRW};
use crate::{
    account::AccountInstance,
    assets::MetadataResources,
    game::{
        item::GroundItem,
        plugin::{AabbResources, GameState, PlayerIdInfo},
        skill::SkillOwner,
    },
    state::AppState,
    world::RpgWorld,
};

use bevy::{
    ecs::{
        entity::Entity,
        event::EventReader,
        query::{With, Without},
        schedule::NextState,
        system::{Commands, Query, Res, ResMut},
    },
    hierarchy::DespawnRecursiveExt,
    log::{error, info},
    math::Vec3,
    transform::components::Transform,
};

use lightyear::prelude::server::*;
use lightyear::prelude::*;

use rpg_core::{game_mode::GameMode, storage::Storage};
use rpg_network_protocol::protocol::*;
use rpg_util::{
    actions::{Action, ActionData, Actions, AttackData, State},
    item::UnitStorage,
    skill::{get_skill_origin, SkillSlots, SkillUse, Skills},
    unit::{Corpse, Hero, HeroBundle, Unit, UnitBundle},
};
use rpg_world::zone::{ZoneId, ZoneInfo};
use util::math::AabbComponent;

pub(crate) fn receive_player_join(
    mut server_state: ResMut<GameState>,
    mut join_reader: EventReader<MessageEvent<CSPlayerJoin>>,
    net_params: NetworkParamsRO,
) {
    for event in join_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        info!("player joined");
    }
}

pub(crate) fn receive_player_leave(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut leave_reader: EventReader<MessageEvent<CSPlayerLeave>>,
    mut game_state: ResMut<GameState>,
    mut net_params: NetworkParamsRW,
    player_q: Query<&Unit>,
    skill_q: Query<(Entity, &SkillOwner), With<SkillUse>>,
) {
    for event in leave_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        game_state.players.retain(|p| p.client_id != client_id);

        let player = player_q.get(client.entity).unwrap();

        net_params.server.send_message_to_target::<Channel1, _>(
            SCPlayerLeave(player.uid),
            NetworkTarget::AllExcept(vec![client_id]),
        );

        // despawn any active skills that the player has cast
        for (entity, owner) in &skill_q {
            if owner.entity == client.entity {
                commands.entity(entity).despawn_recursive();
            }
        }

        // remove game play components from the client's entity
        commands
            .entity(client.entity)
            .remove::<(HeroBundle, Transform, AabbComponent)>();

        if game_state.players.is_empty() {
            info!("no players remain, ending game");
            state.set(AppState::CleanupSimulation);
        } else {
            info!("player left");
        }
    }
}

/// This is received when the client has completed loading the world and is ready to be spawned
pub(crate) fn receive_player_loaded(
    mut commands: Commands,
    mut ready_reader: EventReader<MessageEvent<CSClientReady>>,
    mut net_params: NetworkParamsRW,
    aabbs: Res<AabbResources>,
    game_state: Res<GameState>,
    rpg_world: Res<RpgWorld>,
    account_q: Query<&AccountInstance, Without<Unit>>,
    hero_q: Query<&Transform, With<Unit>>,
) {
    for event in ready_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let account = account_q.get(client.entity).unwrap();

        let id_info = game_state
            .get_id_info_from_account_id(account.0.info.id)
            .unwrap();

        let character = account
            .get_character_from_uid(id_info.character_id)
            .unwrap();

        info!("spawning player: {:?} {id_info:?}", account.0.info);

        let unit = character.character.unit.clone();
        let aabb = aabbs.aabbs["hero"];

        let zone_info = &rpg_world.zones[&ZoneId(0)].zone.as_ref().unwrap().info;
        let mut spawn_position = if let ZoneInfo::OverworldTown(info) = zone_info {
            info.spawn_position
        } else if let ZoneInfo::UnderworldTown(info) = zone_info {
            info.spawn_position
        } else {
            panic!("this should never happen {zone_info:?}");
        };

        let mut position_is_valid = true;
        let mut attempts = 0;
        let max_attempts = 16;

        while attempts < max_attempts {
            position_is_valid = true;

            if attempts % 2 != 0 {
                spawn_position.x += 2.0;
            } else if attempts % 4 != 0 {
                spawn_position.y += 2.0;
            }

            for hero_transform in &hero_q {
                if hero_transform.translation.distance(spawn_position) < 2.0 {
                    position_is_valid = false;
                    break;
                }
            }

            if position_is_valid {
                break;
            }

            attempts += 1;
        }

        if !position_is_valid {
            error!("unable to find a valid spawn position");
            return;
        }

        net_params.server.send_message_to_target::<Channel1, _>(
            SCPlayerSpawn {
                position: spawn_position,
            },
            NetworkTarget::Only(vec![client_id]),
        );

        net_params.server.send_message_to_target::<Channel1, _>(
            SCSpawnHero {
                uid: unit.uid,
                position: spawn_position,
                name: unit.name.clone(),
                class: unit.class,
                level: unit.level,
                deaths: None,
                skills: character.character.skills.clone(),
                skill_slots: character.character.skill_slots.clone(),
            },
            NetworkTarget::AllExcept(vec![client_id]),
        );

        commands.entity(client.entity).insert((
            AabbComponent(aabb),
            Transform::from_translation(spawn_position),
            HeroBundle {
                unit: UnitBundle::new(
                    Unit(unit),
                    Skills(character.character.skills.clone()),
                    SkillSlots::new(character.character.skill_slots.clone()),
                ),
                hero: Hero,
            },
        ));
        // TODO ensure the player is spawned in a town
    }
}

pub(crate) fn receive_player_revive(
    mut commands: Commands,
    server_state: Res<GameState>,
    mut join_reader: EventReader<MessageEvent<CSPlayerRevive>>,
    mut net_params: NetworkParamsRW,
    mut player_q: Query<(&mut Unit, &mut Transform), (With<Hero>, With<Corpse>)>,
) {
    for event in join_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        if server_state.options.mode == GameMode::Hardcore {
            // TODO disconnect and logout or such
            info!("hardcore player attempted to revive!");
            continue;
        }

        // TODO implement xp_loss
        let (mut hero, mut transform) = player_q.get_mut(client.entity).unwrap();
        hero.stats.vitals.stats.get_mut("Hp").unwrap().value =
            hero.stats.vitals.stats["HpMax"].value;

        transform.translation = Vec3::ZERO;

        let hero_info = hero.info.hero_mut();
        let deaths = hero_info.deaths.unwrap();
        *hero_info.deaths.as_mut().unwrap() = deaths + 1;
        commands.entity(client.entity).remove::<Corpse>();

        net_params.server.send_message_to_target::<Channel1, _>(
            SCPlayerRevive {
                position: Vec3::ZERO,
                deaths: hero_info.deaths.unwrap(),
                hp: *hero.stats.vitals.stats["HpMax"].value.u32(),
                xp_total: 0,
                xp_loss: 0,
            },
            NetworkTarget::Only(vec![client_id]),
        );

        net_params.server.send_message_to_target::<Channel1, _>(
            SCHeroRevive(transform.translation),
            NetworkTarget::AllExcept(vec![client_id]),
        );
        info!("player revive");
    }
}

/// Move player
// TODO
// - FIXME this needs to be sent to all characters in range of each player
//   that moves for now a naive approach will be taken
// - FIXME add action request, move message send to action handler
pub(crate) fn receive_movement(
    mut movement_reader: EventReader<MessageEvent<CSMovePlayer>>,
    net_params: NetworkParamsRO,
    mut player_q: Query<&mut Actions, With<Hero>>,
) {
    for event in movement_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let mut actions = player_q.get_mut(client.entity).unwrap();
        actions.request(Action::new(ActionData::Move(Vec3::NEG_Z), None, true));
        //info!("player move request");
    }
}

pub(crate) fn receive_movement_end(
    mut movement_reader: EventReader<MessageEvent<CSMovePlayerEnd>>,
    net_params: NetworkParamsRO,
    mut player_q: Query<&mut Actions, With<Hero>>,
) {
    for event in movement_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let mut actions = player_q.get_mut(client.entity).unwrap();
        if let Some(action) = &mut actions.movement {
            action.state = State::Finalize;
            //info!("end player move request");
        }
    }
}

/// Rotate player
pub(crate) fn receive_rotation(
    mut rotation_reader: EventReader<MessageEvent<CSRotPlayer>>,
    net_params: NetworkParamsRO,
    mut player_q: Query<&mut Actions, With<Hero>>,
) {
    for event in rotation_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let rot_msg = event.message();

        let mut actions = player_q.get_mut(client.entity).unwrap();
        actions.request(Action::new(ActionData::LookDir(rot_msg.0), None, true));
    }
}

pub(crate) fn receive_skill_use_direct(
    mut skill_use_reader: EventReader<MessageEvent<CSSkillUseDirect>>,
    net_params: NetworkParamsRO,
    metadata: Res<MetadataResources>,
    mut player_q: Query<(&Transform, &mut Actions), With<Hero>>,
) {
    for event in skill_use_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let (transform, mut actions) = player_q.get_mut(client.entity).unwrap();
        let skill_msg = event.message();
        // info!("skill use direct: {skill_msg:?}");

        let skill_target = get_skill_origin(
            &metadata.0,
            &transform,
            transform.translation, // FIXMEcursor_position.ground,
            skill_msg.0,
        );

        if actions.attack.is_none() && actions.knockback.is_none() {
            actions.request(Action::new(
                ActionData::Attack(AttackData {
                    skill_id: skill_msg.0,
                    user: transform.translation,
                    skill_target,
                }),
                None,
                true,
            ));
        }
    }
}

pub(crate) fn receive_skill_use_targeted(
    mut skill_use_reader: EventReader<MessageEvent<CSSkillUseTargeted>>,
    net_params: NetworkParamsRO,
    metadata: Res<MetadataResources>,
    mut player_q: Query<(&Transform, &mut Actions), With<Hero>>,
) {
    for event in skill_use_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let (transform, mut actions) = player_q.get_mut(client.entity).unwrap();
        let skill_msg = event.message();
        // info!("skill use targeted: {skill_msg:?}");

        let skill_target = get_skill_origin(
            &metadata.0,
            &transform,
            skill_msg.target, // FIXMEcursor_position.ground,
            skill_msg.skill_id,
        );

        if actions.attack.is_none() && actions.knockback.is_none() {
            actions.request(Action::new(
                ActionData::Attack(AttackData {
                    skill_id: skill_msg.skill_id,
                    user: transform.translation,
                    skill_target,
                }),
                None,
                true,
            ));
        }
    }
}

pub(crate) fn receive_item_drop(
    mut drop_reader: EventReader<MessageEvent<CSItemDrop>>,
    mut net_params: NetworkParamsRW,
    mut hero_q: Query<(&Transform, &Unit), With<Hero>>,
) {
    for event in drop_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };
    }
}

pub(crate) fn receive_item_pickup(
    mut commands: Commands,
    mut pickup_reader: EventReader<MessageEvent<CSItemPickup>>,
    mut net_params: NetworkParamsRW,
    mut item_q: Query<(Entity, &mut GroundItem, &Transform)>,
    mut hero_q: Query<(&Transform, &mut UnitStorage), With<Hero>>,
) {
    for event in pickup_reader.read() {
        let client_id = *event.context();
        let client = net_params.context.clients.get(&client_id).unwrap();
        if !client.is_authenticated_player() {
            continue;
        };

        let pickup_msg = event.message();

        let (u_transform, mut u_storage) = hero_q.get_mut(client.entity).unwrap();

        for (i_entity, mut i_item, i_transform) in &mut item_q {
            if i_item.0 != pickup_msg.0 {
                continue;
            }

            if i_transform.translation.distance(u_transform.translation) < 0.5 {
                let Some(slot) = u_storage.0.get_empty_slot_mut() else {
                    break;
                };

                /* FIXME
                slot.item = i_item.0;
                */

                net_params.server.send_message_to_target::<Channel1, _>(
                    SCDespawnItem(pickup_msg.0),
                    NetworkTarget::All,
                );

                info!("ground item pickup");
                commands.entity(i_entity).despawn_recursive();
            }
        }
    }
}
