use super::room::RoomSpawn;

use crate::game::{assets::RenderResources, plugin::GameSessionCleanup};

use rpg_world::{
    edge::{Edge, EdgeFlags},
    room::Room,
    zone::{self, Connection, ConnectionKind, Kind, SizeInfo, ZoneId},
    zone_path::ZonePath,
};
use util::cleanup::CleanupStrategy;

use bevy::{
    asset::Assets,
    ecs::{
        component::Component,
        system::{Commands, Res, ResMut, Resource},
    },
    log::debug,
    math::{uvec2, Quat, UVec2, Vec2, Vec3},
    pbr::{PbrBundle, StandardMaterial},
    render::mesh::{shape::Quad, Mesh},
    transform::components::Transform,
    utils::default,
};

use std::f32::consts::{FRAC_PI_2, PI};

#[derive(Component)]
pub struct Ground;

#[derive(Debug, Default)]
pub struct ZoneDebugOptions {
    pub room_debug: bool,
    pub tile_debug: bool,
    pub tile_edge_debug: bool,
}

#[derive(Resource)]
pub struct Zone {
    pub zone: zone::Zone,
    pub debug_options: Option<ZoneDebugOptions>,
}

pub fn setup(
    mut commands: Commands,
    renderables: Res<RenderResources>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let path = ZonePath::generate();

    //println!("curve {curve:?}");
    let size_info = SizeInfo::new(uvec2(8, 8), uvec2(4, 4), uvec2(4, 4));
    let mut zone = zone::Zone::new(ZoneId(0), 1234, size_info, Kind::Overworld, path);
    zone.create_rooms();
    zone.set_tile_path();

    let debug_options = Some(ZoneDebugOptions {
        room_debug: true,
        tile_debug: true,
        tile_edge_debug: true,
    });

    let mut zone = Zone {
        zone,
        debug_options,
    };

    build_zone(&mut zone, &mut commands, &renderables, &mut meshes);

    commands.insert_resource(zone);
}

pub fn cleanup(mut zone: ResMut<Zone>) {
    zone.zone.rooms.clear();
}

fn build_zone(
    zone: &mut Zone,
    commands: &mut Commands,
    renderables: &RenderResources,
    meshes: &mut Assets<Mesh>,
) {
    let room_world_size = zone.zone.size_info.room_world_size();
    let world_offset = zone.zone.size_info.zone_world_offset();

    /*
    let tile_edge_debug_mesh = meshes.add(
        Quad {
            size: vec2(zone.zone.size_info.tile_size.x as f32, 0.5),
            ..default()
        }
        .into(),
    );
    */

    let mut room_plane = Mesh::from(Quad {
        size: room_world_size.as_vec2(),
        ..default()
    });

    room_plane.generate_tangents().unwrap();

    let room_plane = meshes.add(room_plane);

    /*
    let tile_debug_mesh = meshes.add(
        Quad {
            size: zone.zone.size_info.tile_size.as_vec2() * 0.66,
            ..default()
        }
        .into(),
    );
    */

    match zone.zone.kind {
        Kind::Overworld => {
            // Hedge = 4m
            let mut count = 0;

            zone.zone.rooms.iter().for_each(|room| {
                let room_world_offset = room.position * room_world_size;
                let room_world_pos = room_world_offset + room_world_size / 2;
                let room_world_float = Vec2::new(
                    world_offset.x + room_world_pos.x as f32,
                    world_offset.y + room_world_pos.y as f32,
                );

                /* spawn_debug_connections(zone, room) */

                commands.spawn((
                    GameSessionCleanup,
                    CleanupStrategy::Despawn,
                    Ground,
                    PbrBundle {
                        mesh: room_plane.clone(),
                        material: renderables.materials["tile"].clone_weak(),
                        transform: Transform::from_translation(Vec3::new(
                            room_world_float.x,
                            0.001,
                            room_world_float.y,
                        ))
                        .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                        ..default()
                    },
                ));
            });

            /*
            spawn_debug_tiles(
                &mut commands,
                &renderables,
                &world_offset,
            );
            */

            println!("spawned {count} walls");

            for room in zone.zone.rooms.iter() {
                for _ in 0..room.props {
                    //room.spawn_random_prop(commands, &renderables, zone);
                }
                for tile in &room.tiles {
                    count += room.spawn_wall_section(commands, zone, renderables, tile.index);
                }
            }
        }
        Kind::OverworldTown => {}
        Kind::UnderworldTown => {}
        Kind::Underworld => {}
    }
}

fn spawn_debug_connections(commands: &mut Commands, zone: &Zone, room: &Room) {
    let connections: Vec<_> = zone
        .zone
        .connections
        .iter()
        .filter(|v| v.position / 4 == room.position)
        .collect();

    if !connections.is_empty() && zone.debug_options.as_ref().is_some_and(|v| v.room_debug) {
        debug!("room has connection: {connections:?} {}", room.position);
        /*
        commands.spawn((
            GameSessionCleanup,
            CleanupStrategy::Despawn,
            Ground,
            PbrBundle {
                mesh: room_plane.clone(),
                material: renderables.materials["aura_red"].clone_weak(),
                transform: Transform::from_translation(Vec3::new(
                    world_position.x as f32,
                    0.001,
                    world_position.y as f32,
                ))
                .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                ..default()
            },
        ));
        */
    } else if let Some(pos) = zone.zone.room_route.iter().find(|v| **v == room.position) {
        /*
        commands.spawn((
            GameSessionCleanup,
            CleanupStrategy::Despawn,
            Ground,
            PbrBundle {
                mesh: room_plane.clone(),
                material: renderables.materials["aura_red"].clone_weak(),
                transform: Transform::from_translation(Vec3::new(
                    world_position.x as f32,
                    0.001,
                    world_position.y as f32,
                ))
                .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                ..default()
            },
        ));
        */
    }
}

fn spawn_debug_tiles(
    commands: &mut Commands,
    renderables: &RenderResources,
    zone: &Zone,
    room: &Room,
    room_world_offset: &UVec2,
    world_offset: &Vec2,
) {
    let tile_size = zone.zone.size_info.tile_size;
    for tile in &room.tiles {
        let tile_position = tile.position();
        let tile_spawn = *room_world_offset + tile_position * tile_size;
        //println!("room world {room_world_offset} {tile_position} {tile_spawn}");

        /*
        commands.spawn((
            GameSessionCleanup,
            CleanupStrategy::Despawn,
            TileNode,
            PbrBundle {
                mesh: tile_plane.clone(),
                material: renderables.materials["aura_red"].clone_weak(),
                transform: Transform::from_translation(Vec3::new(
                    world_offset.x + tile_spawn.x as f32 + 2.,
                    0.002,
                    world_offset.y + tile_spawn.y as f32 + 2.,
                ))
                .with_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                ..default()
            },
        ));
        */

        for edge in tile.edges {
            //println!("edge {edge:?}");

            let key = if edge.edge_flags & EdgeFlags::Open as u8 != 0 {
                "tile_green"
            } else if edge.edge_flags & EdgeFlags::Barrier as u8 != 0 {
                "tile_red"
            } else {
                continue;
            };

            let y = if key == "tile_green" { 0.004 } else { 0.005 };

            let edge_info = if edge.edge == Edge::Top {
                let pos = Vec3::new(
                    world_offset.x + tile_spawn.x as f32 + 2.,
                    y,
                    world_offset.y + tile_spawn.y as f32 + 0.25,
                );

                Some((key, pos, PI))
            } else if edge.edge == Edge::Bottom {
                let pos = Vec3::new(
                    world_offset.x + tile_spawn.x as f32 + 2.,
                    y,
                    world_offset.y + tile_spawn.y as f32 + 3.75,
                );

                Some((key, pos, PI))
            } else if edge.edge == Edge::Left {
                let pos = Vec3::new(
                    world_offset.x + tile_spawn.x as f32 + 0.25,
                    y,
                    world_offset.y + tile_spawn.y as f32 + 2.,
                );

                Some((key, pos, FRAC_PI_2))
            } else if edge.edge == Edge::Right {
                let pos = Vec3::new(
                    world_offset.x + tile_spawn.x as f32 + 3.75,
                    y,
                    world_offset.y + tile_spawn.y as f32 + 2.,
                );

                Some((key, pos, FRAC_PI_2))
            } else {
                None
            };

            if let Some((key, pos, rot)) = edge_info {
                let mut transform = Transform::from_translation(pos);
                transform.rotate_x(-FRAC_PI_2);
                transform.rotate_y(rot);

                /*
                commands.spawn((
                    GameSessionCleanup,
                    CleanupStrategy::Despawn,
                    TileNode,
                    PbrBundle {
                        mesh: tile_edge.clone(),
                        material: renderables.materials[key].clone_weak(),
                        //material: renderables.materials["aura_red"].clone_weak(),
                        transform,
                        ..default()
                    },
                ));*/
            }
        }

        let key = if zone
            .zone
            .connections
            .iter()
            .any(|v| v.position == *room_world_offset / 4 + tile_position)
        {
            "tile_red"
        } else if zone
            .zone
            .tile_route
            .iter()
            .any(|pos| *pos == *room_world_offset / 4 + tile_position)
        {
            "tile_purple"
        } else {
            "tile_orange"
        };
    }
}
