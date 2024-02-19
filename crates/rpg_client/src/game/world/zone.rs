use super::{room::RoomSpawn, LoadZone, RpgWorld};

use crate::game::{
    assets::RenderResources, metadata::MetadataResources, plugin::GameSessionCleanup, prop,
};

use rpg_world::zone::Zone;
use util::cleanup::CleanupStrategy;

use bevy::{
    asset::Assets,
    ecs::{
        component::Component,
        event::EventReader,
        system::{Commands, Res, ResMut},
    },
    hierarchy::BuildChildren,
    log::debug,
    math::{primitives::Rectangle, Quat, Vec2, Vec3},
    pbr::{PbrBundle, PointLight, PointLightBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
    utils::default,
};

use fastrand::Rng;

use std::f32::consts::FRAC_PI_2;

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct TileNode;

pub fn load_zone(
    mut commands: Commands,
    metadata: Res<MetadataResources>,
    renderables: Res<RenderResources>,
    mut rpg_world: ResMut<RpgWorld>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut load_reader: EventReader<LoadZone>,
) {
    // FIXME temp hack
    if rpg_world.active_zone.is_some() {
        return;
    }

    for event in load_reader.read() {
        debug!("loading zone");
        let zone_id = event.0;
        let mut zone = Zone::create_town(zone_id, 1234, &metadata.world);
        zone.create_rooms(&metadata.world);

        build_zone(
            &mut commands,
            &metadata,
            &renderables,
            &mut meshes,
            &mut zone,
            &mut rpg_world.rng,
        );
        rpg_world.active_zone = Some(zone_id);
        rpg_world.zones.insert(zone_id, zone);
    }
    /*
    let path = ZonePath::generate();
    let size_info = SizeInfo::new(uvec2(8, 8), uvec2(4, 4), uvec2(4, 4));
    zone.set_tile_path();
    */
}

fn build_zone(
    commands: &mut Commands,
    metadata: &MetadataResources,
    renderables: &RenderResources,
    meshes: &mut Assets<Mesh>,
    zone: &mut Zone,
    rng: &mut Rng,
) {
    let room_world_size = zone.size.room_world_size(&metadata.world);
    let world_offset = zone.size.zone_world_offset(&metadata.world);

    /*
    let tile_edge_debug_mesh = meshes.add(
        Quad {
            size: vec2(zone.zone.size_info.tile_size.x as f32, 0.5),
            ..default()
        }
        .into(),
    );
    */

    let mut room_plane = Mesh::from(Rectangle::new(
        room_world_size.x as f32,
        room_world_size.y as f32,
    ));
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

    /*match zone.kind {
    Kind::Overworld => {}
    Kind::OverworldTown => {}
    Kind::UnderworldTown => {}
    Kind::Underworld => {}*/
    // Hedge = 4m

    zone.rooms.iter().for_each(|room| {
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
                    0.0,
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

    let zone_info = &metadata.world.zone.towns[&zone.id];
    if let Some(waypoint) = &zone_info.waypoint {
        prop::spawn(
            commands,
            metadata,
            renderables,
            "waypoint",
            Vec3::new(waypoint.position.x as f32, 0.0, waypoint.position.y as f32),
            None,
        );
    }

    for prop in &zone_info.props {
        use std::f32::consts;

        let rot_y = consts::TAU * (0.5 - rng.f32());

        let position = prop.position;

        let id = prop::spawn(
            commands,
            metadata,
            renderables,
            prop.key.as_str(),
            Vec3::new(position.x as f32, 0.5, position.y as f32),
            Some(Quat::from_rotation_y(rot_y)),
        );

        if prop.key == "ground_lamp_1" {
            let point = commands
                .spawn(PointLightBundle {
                    point_light: PointLight {
                        color: Color::rgb(0.96, 0.92, 0.78),
                        intensity: 666.,
                        ..default()
                    },
                    ..default()
                })
                .id();

            commands.entity(point).set_parent(id);
        }

        // room.spawn_random_prop(commands, renderables, zone, rng, metadata);
    }

    for room in zone.rooms.iter() {
        for tile in &room.tiles {
            room.spawn_wall_section(commands, metadata, renderables, zone, tile.index);
        }
    }
}

/*
fn spawn_debug_connections(commands: &mut Commands, zone: &Zone, room: &Room) {
    let connections: Vec<_> = zone
        .connections
        .iter()
        .filter(|v| v.position / 4 == room.position)
        .collect();

    if !connections.is_empty() && zone.debug_options.as_ref().is_some_and(|v| v.room_debug) {
        debug!("room has connection: {connections:?} {}", room.position);

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
    } else
    if let Some(pos) = zone.room_route.iter().find(|v| **v == room.position) {
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
    }
}

fn spawn_debug_tiles(
    commands: &mut Commands,
    renderables: &RenderResources,
    metadata: &WorldMetadata,
    zone: &Zone,
    room: &Room,
    room_world_offset: &UVec2,
    world_offset: &Vec2,
) {
    let tile_size = metadata.zone.size_info.tile;
    for tile in &room.tiles {
        let tile_position = tile.position();
        let tile_spawn = *room_world_offset + tile_position * tile_size;
        //println!("room world {room_world_offset} {tile_position} {tile_spawn}");

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
            .connections
            .iter()
            .any(|v| v.position == *room_world_offset / 4 + tile_position)
        {
            "tile_red"
        } else if zone
            .tile_route
            .iter()
            .any(|pos| *pos == *room_world_offset / 4 + tile_position)
        {
            "tile_purple"
        } else {
            "tile_orange"
        };
    }
}*/
