use crate::edge::{Edge, EdgeFlags};

use bevy_math::{uvec2, UVec2};

#[derive(Debug)]
pub enum TileFlags {
    None = 0x00,
    Blocked = 0x01,
    Walkable = 0x01 << 1,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct TileEdge {
    pub edge: Edge,
    pub flags: u8,
    pub edge_flags: u8,
}

impl TileEdge {
    pub fn new(edge: Edge, edge_flags: EdgeFlags) -> Self {
        Self {
            edge,
            flags: 1 << edge as u8,
            edge_flags: edge_flags as u8,
        }
        //println!("flags: {flags:#010b}");
    }

    pub fn get_edge_flags(&self) -> u8 {
        self.edge_flags
    }

    pub fn get_tile_flags(&self) -> u8 {
        self.flags
    }

    pub fn set_tile_flag(&mut self, tile_flag: TileFlags) {
        self.flags |= tile_flag as u8;
    }

    pub fn set_edge_flag(&mut self, edge_flag: EdgeFlags) {
        self.edge_flags |= edge_flag as u8;
    }

    pub fn has_edge_flag(&self, edge_flag: EdgeFlags) -> bool {
        self.edge_flags & edge_flag as u8 != 0
    }

    pub fn clear_edge_flags(&mut self) {
        self.edge_flags = 0;
    }

    pub fn edge_flags_empty(&self) -> bool {
        self.edge_flags == 0
    }

    pub fn clear_tile_flags(&mut self) {
        self.flags = 0;
    }

    pub fn tile_flags_empty(&self) -> bool {
        self.flags == 0
    }

    pub fn has_tile_flag(&self, tile_flag: TileFlags) -> bool {
        self.flags & tile_flag as u8 != 0
    }
}

#[derive(Debug)]
pub struct Tile {
    pub index: u8,
    pub edges: [TileEdge; 4],
}

impl Tile {
    pub fn new(index: u8) -> Self {
        Self {
            index,
            ..Default::default()
        }
    }

    pub fn position(&self) -> UVec2 {
        UVec2::new((self.index % 4) as u32, (self.index / 4) as u32)
    }

    pub fn get_edge_offset(&self, edge: Edge) -> UVec2 {
        match edge {
            Edge::Bottom => uvec2(2, 4),
            Edge::Top => uvec2(2, 0),
            Edge::Left => uvec2(0, 2),
            Edge::Right => uvec2(4, 2),
        }
    }

    pub fn index(position: UVec2) -> u8 {
        assert!(position.cmpge(UVec2::ZERO).all());
        assert!(position.cmplt(UVec2::splat(4)).all());

        position.x as u8 + position.y as u8 * 4
    }

    pub fn get_edge(&self, edge: Edge) -> &TileEdge {
        &self.edges[edge as usize]
    }

    pub fn get_edge_mut(&mut self, edge: Edge) -> &mut TileEdge {
        &mut self.edges[edge as usize]
    }

    pub fn set_edge_flag_if(&mut self, edge: Edge, if_flag: EdgeFlags, edge_flag: EdgeFlags) {
        if self.edges[edge as usize].edge_flags & if_flag as u8 != 0 {
            self.set_edge_flag(edge, edge_flag);
        }
    }

    pub fn set_edge_flag_if_not(&mut self, edge: Edge, not_flag: EdgeFlags, edge_flag: EdgeFlags) {
        if self.edges[edge as usize].edge_flags & not_flag as u8 != 0 {
            self.set_edge_flag(edge, edge_flag);
        }
    }
    pub fn set_edge_flag(&mut self, edge: Edge, edge_flag: EdgeFlags) {
        self.edges[edge as usize].edge_flags |= edge_flag as u8;
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            index: 0,
            edges: [
                TileEdge::new(Edge::Top, EdgeFlags::None),
                TileEdge::new(Edge::Bottom, EdgeFlags::None),
                TileEdge::new(Edge::Left, EdgeFlags::None),
                TileEdge::new(Edge::Right, EdgeFlags::None),
            ],
        }
    }
}
