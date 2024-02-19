use crate::game::{assets::RenderResources, metadata::MetadataResources, prop};

use bevy::{
    ecs::system::Commands,
    hierarchy::BuildChildren,
    log::debug,
    math::{Quat, Vec3},
    pbr::{PointLight, PointLightBundle},
    render::color::Color,
    utils::default,
};

use rpg_world::{
    edge::{Edge, EdgeFlags},
    room::Room,
    zone::Zone,
};

use fastrand::Rng;

pub trait RoomSpawn {
    fn spawn_wall_section(
        &self,
        commands: &mut Commands,
        metadata: &MetadataResources,
        renderables: &RenderResources,
        zone: &Zone,
        tile: u8,
    ) -> usize;

    fn spawn_random_prop(
        &self,
        commands: &mut Commands,
        metadata: &MetadataResources,
        renderables: &RenderResources,
        zone: &Zone,
        rng: &mut Rng,
    );
}

impl RoomSpawn for Room {
    fn spawn_wall_section(
        &self,
        commands: &mut Commands,
        metadata: &MetadataResources,
        renderables: &RenderResources,
        zone: &Zone,
        tile: u8,
    ) -> usize {
        let tile = &self.tiles[tile as usize];
        let tile_position = tile.position();

        let room_world_size = zone.size.room_world_size(&metadata.world);
        let tile_offset =
            self.position * room_world_size + metadata.world.zone.size_info.tile * tile_position;
        let world_offset = zone.size.zone_world_offset(&metadata.world);

        // debug!("room pos {} world_off {world_offset} room_world_size {room_world_size} tile_off {tile_offset}", self.position);

        let mut count = 0;
        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            if tile.get_edge(edge).has_edge_flag(EdgeFlags::Barrier) {
                let rotation = match edge {
                    Edge::Top | Edge::Bottom => None,
                    _ => Some(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                };

                let pos = world_offset + (tile_offset + tile.get_edge_offset(edge)).as_vec2();

                count += 1;
                // debug!("spawn wall at {x} {y}");
                prop::spawn(
                    commands,
                    metadata,
                    renderables,
                    "wall_hedge_1",
                    Vec3::new(pos.x, 0., pos.y),
                    rotation,
                );
            }
        }

        count
    }

    fn spawn_random_prop(
        &self,
        commands: &mut Commands,
        metadata: &MetadataResources,
        renderables: &RenderResources,
        zone: &Zone,
        rng: &mut Rng,
    ) {
        use std::f32::consts;

        let key = match rng.usize(0..2) {
            0 => "rock_1",
            _ => "ground_lamp_1",
        };
        let rot_y = consts::TAU * (0.5 - rng.f32());

        let position = zone.generate_position(rng, &metadata.world, self);

        let id = prop::spawn(
            commands,
            metadata,
            renderables,
            key,
            position,
            Some(Quat::from_rotation_y(rot_y)),
        );

        if key == "ground_lamp_1" {
            let point = commands
                .spawn(PointLightBundle {
                    point_light: PointLight {
                        color: Color::rgb(0.96, 0.92, 0.73),
                        intensity: 600.,
                        ..default()
                    },
                    ..default()
                })
                .id();

            commands.entity(point).set_parent(id);
        }
    }
}
