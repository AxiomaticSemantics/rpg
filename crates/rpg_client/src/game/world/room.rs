use super::zone::Zone;

use crate::game::{assets::RenderResources, prop};

use bevy::{
    ecs::system::Commands,
    log::debug,
    math::{Quat, Vec3},
};

use rpg_world::{
    edge::{Edge, EdgeFlags},
    room::Room,
};

pub trait RoomSpawn {
    fn spawn_wall_section(
        &self,
        commands: &mut Commands,
        zone: &Zone,
        renderables: &RenderResources,
        tile: u8,
    ) -> usize;

    fn spawn_random_prop(
        &self,
        commands: &mut Commands,
        renderables: &RenderResources,
        zone: &mut Zone,
    );
}

impl RoomSpawn for Room {
    fn spawn_wall_section(
        &self,
        commands: &mut Commands,
        zone: &Zone,
        renderables: &RenderResources,
        tile: u8,
    ) -> usize {
        let tile = &self.tiles[tile as usize];
        let tile_position = tile.position();

        let room_world_size = zone.zone.size_info.room_world_size();
        let tile_offset =
            self.position * room_world_size + zone.zone.size_info.tile_size * tile_position;
        let world_offset = zone.zone.size_info.zone_world_offset();

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
        renderables: &RenderResources,
        zone: &mut Zone,
    ) {
        use std::f32::consts;

        let key = match zone.zone.rng.usize(0..2) {
            0 => "rock_1",
            _ => "ground_lamp_1",
        };
        let rot_y = consts::TAU * (0.5 - zone.zone.rng.f32());

        prop::spawn(
            commands,
            renderables,
            key,
            zone.zone.generate_position(self),
            Some(Quat::from_rotation_y(rot_y)),
        );
    }
}
