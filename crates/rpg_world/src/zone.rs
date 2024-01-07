use crate::{
    edge::{Edge, EdgeFlags},
    room::Room,
    tile::Tile,
};

use fastrand::Rng;

use glam::{uvec2, UVec2, Vec2, Vec3};
use serde_derive::{Deserialize as De, Serialize as Ser};

use std::collections::VecDeque;

#[derive(Ser, De, Copy, Clone, PartialEq, Eq, Debug)]
pub struct ZoneId(pub u16);

#[derive(Debug)]
pub struct Zone {
    pub id: ZoneId,
    pub size_info: SizeInfo,
    pub kind: Kind,
    pub connections: Vec<Connection>,
    pub room_route: Vec<UVec2>,
    pub tile_route: Vec<UVec2>,
    pub curves: VecDeque<Vec<Vec3>>,
    pub rooms: Vec<Room>,
    pub rng: Rng,
}

impl Zone {
    pub fn new(
        id: ZoneId,
        seed: u64,
        size_info: SizeInfo,
        kind: Kind,
        curves: VecDeque<Vec<Vec3>>,
    ) -> Self {
        let mut room_route = vec![];
        let mut tile_route = vec![];

        for &position in curves.front().unwrap().iter() {
            let tile_position = uvec2(
                ((position.x).floor() as u32).clamp(0, 31),
                ((position.z).floor() as u32).clamp(0, 31),
            );

            let room_position = tile_position / size_info.room_size.x;

            //println!("position {position} {x} {y} {i_position} to route");
            if !room_route.iter().any(|v| *v == room_position) {
                //println!("pushing {room_position} to route");
                room_route.push(room_position);
            }

            let Some(back): Option<&UVec2> = tile_route.iter().last() else {
                tile_route.push(tile_position);
                continue;
            };

            if !tile_route.iter().any(|v| *v == tile_position) {
                if !tile_position.cmpeq(*back).any() {
                    tile_route.push(if back.x > tile_position.x {
                        *back - uvec2(1, 0)
                    } else {
                        *back + uvec2(1, 0)
                    });
                }
                tile_route.push(tile_position);
            }
        }

        for &position in curves.back().unwrap().iter() {
            let tile_position = uvec2(
                ((position.x).floor() as u32).clamp(0, 31),
                ((position.z).floor() as u32).clamp(0, 31),
            );

            let room_position = tile_position / 4;

            //println!("position {position} {x} {y} {i_position} to route");
            if !room_route.iter().any(|v| *v == room_position) {
                //println!("pushing {room_position} to route");
                room_route.push(room_position);
            }

            let Some(back): Option<&UVec2> = tile_route.iter().last() else {
                tile_route.push(tile_position);
                continue;
            };

            if !tile_route.iter().any(|v| *v == tile_position) {
                if !tile_position.cmpeq(*back).any() {
                    tile_route.push(if back.x > tile_position.x {
                        *back - uvec2(1, 0)
                    } else {
                        *back + uvec2(1, 0)
                    });
                }
                tile_route.push(tile_position);
            }
        }

        let room_size_vec = size_info.extent;

        let rooms = Vec::with_capacity((room_size_vec.x * room_size_vec.y) as usize);

        assert!(tile_route.len() > 1);
        let front = tile_route.first().unwrap();
        let back = tile_route.last().unwrap();

        let connections = vec![
            Connection::new(ConnectionKind::Edge(Edge::Left), *front),
            Connection::new(ConnectionKind::Edge(Edge::Bottom), *back),
        ];

        Self {
            id,
            size_info,
            kind,
            connections,
            room_route,
            tile_route,
            curves,
            rooms,
            rng: Rng::with_seed(seed),
        }
    }

    pub fn create_rooms(&mut self) {
        let room_size_vec = self.size_info.extent;
        for y in 0..room_size_vec.y {
            for x in 0..room_size_vec.x {
                let room = Room::new(self, uvec2(x, y), 2);
                self.rooms.push(room);
            }
        }
    }

    pub fn get_tile_from_position(&mut self, position: &UVec2) -> &Tile {
        let room_position = *position / self.size_info.tile_size;
        let room_inner = *position % 4;

        //println!("{position} room_pos {room_position} room_inner {room_inner}");
        let room = self
            .rooms
            .iter()
            .find(|v| v.position == room_position)
            .expect("OOB");

        &room.tiles[(room_inner.y * 4 + room_inner.x) as usize]
    }

    pub fn get_tile_from_position_mut(&mut self, position: &UVec2) -> &mut Tile {
        let room_position = *position / self.size_info.tile_size;
        let room_inner = *position % 4;

        //println!("{position} room_pos {room_position} room_inner {room_inner}");
        let room = self
            .rooms
            .iter_mut()
            .find(|v| v.position == room_position)
            .expect("OOB");

        &mut room.tiles[(room_inner.y * 4 + room_inner.x) as usize]
    }

    pub fn make_tile_barriers(&mut self, next_edge: Edge, position: &UVec2) {
        let tile = self.get_tile_from_position_mut(position);
        match next_edge {
            Edge::Top => {
                let bottom = tile.get_edge_mut(Edge::Bottom);
                if bottom.edge_flags_empty() {
                    bottom.set_edge_flag(EdgeFlags::Barrier);
                }
                let left = tile.get_edge_mut(Edge::Left);
                if left.edge_flags_empty() {
                    left.set_edge_flag(EdgeFlags::Barrier);
                }

                let right = tile.get_edge_mut(Edge::Right);
                if right.edge_flags_empty() {
                    right.set_edge_flag(EdgeFlags::Barrier);
                }
            }
            Edge::Bottom => {
                let top = tile.get_edge_mut(Edge::Top);
                if top.edge_flags_empty() {
                    top.set_edge_flag(EdgeFlags::Barrier);
                }

                let left = tile.get_edge_mut(Edge::Left);
                if left.edge_flags_empty() {
                    left.set_edge_flag(EdgeFlags::Barrier);
                }

                let right = tile.get_edge_mut(Edge::Right);
                if right.edge_flags_empty() {
                    right.set_edge_flag(EdgeFlags::Barrier);
                }
            }
            Edge::Left => {
                let right = tile.get_edge_mut(Edge::Right);
                if right.edge_flags_empty() {
                    right.set_edge_flag(EdgeFlags::Barrier);
                }

                let top = tile.get_edge_mut(Edge::Top);
                if top.edge_flags_empty() {
                    top.set_edge_flag(EdgeFlags::Barrier);
                }

                let bottom = tile.get_edge_mut(Edge::Bottom);
                if bottom.edge_flags_empty() {
                    bottom.set_edge_flag(EdgeFlags::Barrier);
                }
            }
            Edge::Right => {
                let left = tile.get_edge_mut(Edge::Left);
                if left.edge_flags_empty() {
                    left.set_edge_flag(EdgeFlags::Barrier);
                }

                let top = tile.get_edge_mut(Edge::Top);
                if top.edge_flags_empty() {
                    top.set_edge_flag(EdgeFlags::Barrier);
                }

                let bottom = tile.get_edge_mut(Edge::Bottom);
                if bottom.edge_flags_empty() {
                    bottom.set_edge_flag(EdgeFlags::Barrier);
                }
            }
        }
    }

    pub fn set_tile_path(&mut self) {
        let len = self.tile_route.len();

        let mut tile_route = self.tile_route.clone();

        let mut path_iter = tile_route.iter_mut().enumerate().peekable();
        while let Some((index, position)) = path_iter.next() {
            if index == 0 {
                if let ConnectionKind::Edge(edge) = self.connections[0].kind {
                    let tile = self.get_tile_from_position_mut(position);
                    tile.set_edge_flag(edge, EdgeFlags::Open);
                }
            } else if index == len - 1 {
                let edge = match self.connections[1].kind {
                    ConnectionKind::Edge(edge) => {
                        let tile = self.get_tile_from_position_mut(position);
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

                self.make_tile_barriers(edge, position);
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

                let next_tile = self.get_tile_from_position_mut(next.1);
                next_tile.set_edge_flag(next_edge.opposite(), EdgeFlags::Open);

                let tile = self.get_tile_from_position_mut(position);
                tile.set_edge_flag(next_edge, EdgeFlags::Open);
                self.make_tile_barriers(next_edge, position);
            }
            println!("pos {position}");
        }

        self.tile_route = tile_route;
    }

    pub fn generate_position(&mut self, room: &Room) -> Vec3 {
        let room_world_size = self.size_info.room_world_size();
        let world_offset = self.size_info.zone_world_offset();
        let world_pos = world_offset + (room.position * room_world_size).as_vec2();

        Vec3::new(
            world_pos.x + self.rng.u32(1..(room_world_size.x - 1)) as f32,
            0.,
            world_pos.y + self.rng.u32(1..(room_world_size.y - 1)) as f32,
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Kind {
    Town,
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
pub struct SizeInfo {
    pub extent: UVec2,
    pub half_extent: UVec2,
    pub room_size: UVec2,
    pub tile_size: UVec2,
}

impl SizeInfo {
    pub fn new(extent: UVec2, room_size: UVec2, tile_size: UVec2) -> Self {
        Self {
            extent,
            half_extent: extent / 2,
            room_size,
            tile_size,
        }
    }

    pub fn zone_world_offset(&self) -> Vec2 {
        -(self.half_extent.as_vec2()) * self.room_world_size().as_vec2()
    }

    pub fn zone_world_size(&self) -> UVec2 {
        self.extent * self.room_world_size()
    }

    pub fn room_world_size(&self) -> UVec2 {
        self.room_size * self.tile_size
    }
}
