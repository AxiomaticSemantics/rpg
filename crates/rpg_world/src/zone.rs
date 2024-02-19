use crate::{
    edge::{Edge, EdgeFlags},
    metadata::Metadata,
    room::Room,
    tile::Tile,
    zone_path::ZonePath,
};

use fastrand::Rng;

use bevy_math::{uvec2, UVec2, Vec2, Vec3};
use serde_derive::{Deserialize as De, Serialize as Ser};

use std::collections::VecDeque;

#[derive(Ser, De, Debug, Copy, Clone, PartialEq)]
pub enum Biome {
    Temperate,
    Savana,
}

#[derive(Ser, De, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ZoneId(pub u16);

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct TownInfo {
    pub spawn_position: UVec2,
}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct OverworldInfo {}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub struct UnderworldInfo {}

#[derive(Ser, De, Debug, Clone, PartialEq)]
pub enum ZoneInfo {
    OverworldTown(TownInfo),
    UnderworldTown(TownInfo),
    Overworld(OverworldInfo),
    Underworld(UnderworldInfo),
}

#[derive(Debug)]
pub struct Zone {
    pub id: ZoneId,
    pub seed: u64,
    pub size: ZoneSize,
    pub kind: Kind,
    pub biome: Biome,
    pub info: ZoneInfo,
    pub connections: Vec<Connection>,
    pub room_route: Vec<UVec2>,
    pub tile_route: Vec<UVec2>,
    pub path: ZonePath,
    pub rooms: Vec<Room>,
}

impl Zone {
    pub fn new(
        id: ZoneId,
        seed: u64,
        size: ZoneSize,
        kind: Kind,
        biome: Biome,
        path: ZonePath,
        metadata: &Metadata,
    ) -> Self {
        let mut room_route = vec![];
        let mut tile_route = vec![];

        for position in path.0.front().unwrap().iter_positions(256).into_iter() {
            let tile_position = uvec2(
                ((position.x).floor() as u32).clamp(0, 31),
                ((position.y).floor() as u32).clamp(0, 31),
            );

            let room_position = tile_position / metadata.zone.size_info.room.x;

            //println!("position {position} {x} {y} {i_position} to route");
            if !room_route.iter().any(|v| *v == room_position) {
                //println!("pushing {room_position} to route");
                room_route.push(room_position);
            }

            let Some(last): Option<&UVec2> = tile_route.iter().last() else {
                tile_route.push(tile_position);
                continue;
            };

            if !tile_route.iter().any(|v| *v == tile_position) {
                if !tile_position.cmpeq(*last).any() {
                    tile_route.push(if last.x > tile_position.x {
                        *last - uvec2(1, 0)
                    } else {
                        *last + uvec2(1, 0)
                    });
                }
                tile_route.push(tile_position);
            }
        }

        let room_size_vec = size.extent;

        let rooms = Vec::with_capacity((room_size_vec.x * room_size_vec.y) as usize);

        assert!(tile_route.len() > 1);
        let front = tile_route.first().unwrap();
        let back = tile_route.last().unwrap();

        let connections = vec![
            Connection::new(ConnectionKind::Edge(Edge::Left), *front),
            Connection::new(ConnectionKind::Edge(Edge::Bottom), *back),
        ];

        let info = match kind {
            Kind::Overworld => ZoneInfo::Overworld(OverworldInfo {}),
            Kind::Underworld => ZoneInfo::Underworld(UnderworldInfo {}),
            _ => panic!("expected a non-town zone kind"),
        };

        Self {
            id,
            seed,
            size,
            kind,
            biome,
            info,
            connections,
            room_route,
            tile_route,
            path,
            rooms,
        }
    }

    pub fn create_town(id: ZoneId, seed: u64, metadata: &Metadata) -> Self {
        let town_meta = &metadata.zone.towns[&id];
        let zone_info = match town_meta.kind {
            Kind::OverworldTown => ZoneInfo::OverworldTown(TownInfo {
                spawn_position: town_meta.spawn_position,
            }),
            Kind::UnderworldTown => ZoneInfo::UnderworldTown(TownInfo {
                spawn_position: town_meta.spawn_position,
            }),
            _ => panic!("town expected"),
        };

        Self {
            id,
            seed,
            size: ZoneSize::new(town_meta.size),
            biome: town_meta.biome,
            kind: town_meta.kind,
            info: zone_info,
            connections: vec![],
            room_route: vec![],
            tile_route: vec![],
            path: ZonePath(VecDeque::new()),
            rooms: vec![],
        }
    }

    pub fn create_rooms(&mut self, metadata: &Metadata) {
        let room_size_vec = self.size.extent;
        for y in 0..room_size_vec.y {
            for x in 0..room_size_vec.x {
                let room = Room::new(metadata, uvec2(x, y), 2);
                self.rooms.push(room);
            }
        }
    }

    pub fn get_tile_from_position(&mut self, metadata: &Metadata, position: &UVec2) -> &Tile {
        let room_position = *position / metadata.zone.size_info.tile;
        let room_inner = *position % metadata.zone.size_info.room;

        //println!("{position} room_pos {room_position} room_inner {room_inner}");
        let room = self
            .rooms
            .iter()
            .find(|v| v.position == room_position)
            .expect("OOB");

        &room.tiles[(room_inner.y * 4 + room_inner.x) as usize]
    }

    pub fn get_tile_from_position_mut(
        &mut self,
        metadata: &Metadata,
        position: &UVec2,
    ) -> &mut Tile {
        let room_position = *position / metadata.zone.size_info.tile;
        let room_inner = *position % metadata.zone.size_info.room;

        //println!("{position} room_pos {room_position} room_inner {room_inner}");
        let room = self
            .rooms
            .iter_mut()
            .find(|v| v.position == room_position)
            .expect("OOB");

        let index = (room_inner.y * metadata.zone.size_info.tile.x + room_inner.x) as usize;
        assert!(index < room.tiles.len());

        &mut room.tiles[index]
    }

    pub fn make_tile_barriers(&mut self, metadata: &Metadata, next_edge: Edge, position: &UVec2) {
        let tile = self.get_tile_from_position_mut(metadata, position);
        match next_edge {
            Edge::Top => {
                tile.set_edge_flag_if_empty(Edge::Bottom, EdgeFlags::Barrier);
                tile.set_edge_flag_if_empty(Edge::Left, EdgeFlags::Barrier);
                tile.set_edge_flag_if_empty(Edge::Right, EdgeFlags::Barrier);
            }
            Edge::Bottom => {
                tile.set_edge_flag_if_empty(Edge::Top, EdgeFlags::Barrier);
                tile.set_edge_flag_if_empty(Edge::Left, EdgeFlags::Barrier);
                tile.set_edge_flag_if_empty(Edge::Right, EdgeFlags::Barrier);
            }
            Edge::Left => {
                tile.set_edge_flag_if_empty(Edge::Top, EdgeFlags::Barrier);
                tile.set_edge_flag_if_empty(Edge::Bottom, EdgeFlags::Barrier);
                tile.set_edge_flag_if_empty(Edge::Right, EdgeFlags::Barrier);
            }
            Edge::Right => {
                tile.set_edge_flag_if_empty(Edge::Top, EdgeFlags::Barrier);
                tile.set_edge_flag_if_empty(Edge::Bottom, EdgeFlags::Barrier);
                tile.set_edge_flag_if_empty(Edge::Left, EdgeFlags::Barrier);
            }
        }
    }

    pub fn set_tile_path(&mut self, metadata: &Metadata) {
        let len = self.tile_route.len();

        let mut tile_route = self.tile_route.clone();

        let mut path_iter = tile_route.iter_mut().enumerate().peekable();
        while let Some((index, position)) = path_iter.next() {
            if index == 0 {
                if let ConnectionKind::Edge(edge) = self.connections[0].kind {
                    let tile = self.get_tile_from_position_mut(metadata, position);
                    tile.set_edge_flag(edge, EdgeFlags::Open);
                }
            } else if index == len - 1 {
                let edge = match self.connections[1].kind {
                    ConnectionKind::Edge(edge) => {
                        let tile = self.get_tile_from_position_mut(metadata, position);
                        let tile_edge = tile.get_edge_mut(edge);

                        if !tile_edge.edge_flags_empty() {
                            tile_edge.clear_edge_flags();
                        }

                        tile_edge.set_edge_flag(EdgeFlags::Open);

                        edge
                    }
                    _ => {
                        panic!("not yet")
                    }
                };

                self.make_tile_barriers(metadata, edge, position);
                continue;
            }

            if let Some(next) = path_iter.peek() {
                let next_edge = if next.1.x == position.x {
                    if next.1.y > position.y {
                        Edge::Bottom
                    } else {
                        Edge::Top
                    }
                } else if next.1.y == position.y {
                    if next.1.x > position.x {
                        Edge::Right
                    } else {
                        Edge::Left
                    }
                } else {
                    //panic!("this should never happen");
                    continue; //Edge::Top
                };

                let next_tile = self.get_tile_from_position_mut(metadata, next.1);
                next_tile.set_edge_flag(next_edge.opposite(), EdgeFlags::Open);

                let tile = self.get_tile_from_position_mut(metadata, position);
                tile.set_edge_flag(next_edge, EdgeFlags::Open);
                self.make_tile_barriers(metadata, next_edge, position);
            }
            // println!("pos {position}");
        }

        self.tile_route = tile_route;
    }

    pub fn generate_position(&self, rng: &mut Rng, metadata: &Metadata, room: &Room) -> Vec3 {
        let room_world_size = self.size.room_world_size(metadata);
        let world_offset = self.size.zone_world_offset(metadata);
        let world_pos = world_offset + (room.position * room_world_size).as_vec2();

        Vec3::new(
            world_pos.x + rng.u32(1..(room_world_size.x - 1)) as f32,
            0.,
            world_pos.y + rng.u32(1..(room_world_size.y - 1)) as f32,
        )
    }
}

#[derive(Ser, De, Debug, Copy, Clone, PartialEq, Eq)]
pub enum Kind {
    OverworldTown,
    UnderworldTown,
    Overworld,
    Underworld,
}

#[derive(Debug, PartialEq)]
pub struct Connection {
    pub kind: ConnectionKind,
    /// Zone relative position
    pub position: UVec2,
}

impl Connection {
    pub fn new(kind: ConnectionKind, position: UVec2) -> Self {
        Self { kind, position }
    }
}

#[derive(Debug, PartialEq)]
pub enum ConnectionKind {
    Edge(Edge),
    Interior(UVec2),
}

#[derive(Default, Debug)]
pub struct ZoneSize {
    pub extent: UVec2,
    pub half_extent: UVec2,
}

impl ZoneSize {
    pub fn new(extent: UVec2) -> Self {
        Self {
            extent,
            half_extent: extent / 2,
        }
    }

    #[inline(always)]
    pub fn zone_world_offset(&self, metadata: &Metadata) -> Vec2 {
        -(self.half_extent.as_vec2()) * self.room_world_size(metadata).as_vec2()
    }

    #[inline(always)]
    pub fn zone_world_size(&self, metadata: &Metadata) -> UVec2 {
        self.extent * self.room_world_size(metadata)
    }

    #[inline(always)]
    pub fn room_world_size(&self, metadata: &Metadata) -> UVec2 {
        metadata.zone.size_info.room * metadata.zone.size_info.tile
    }
}
