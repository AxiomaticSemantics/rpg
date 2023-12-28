use crate::{class::Class, metadata::Metadata, stat::Stat};
use std::{borrow::Cow, collections::HashMap, fmt};

use glam::Vec2;
use petgraph::{
    algo,
    graph::{EdgeIndex, Graph, NodeIndex},
    Undirected,
};
use serde_derive::{Deserialize as De, Serialize as Ser};

pub const ZONES: usize = 7;
pub const ZONE_SECTIONS: usize = 7;
pub const SECTION_CLUSTERS: usize = 7;

pub const CLUSTER_NODES: usize = 7;
pub const SECTION_NODES: usize = CLUSTER_NODES * SECTION_CLUSTERS;
pub const ZONE_NODES: usize = SECTION_NODES * ZONE_SECTIONS;
pub const TREE_NODES: usize = ZONES * ZONE_NODES;

pub const HEX_INNER: Vec2 = Vec2::ZERO;
pub const HEX_DOWN: Vec2 = Vec2::new(0.0, -1.0);
pub const HEX_DOWN_LEFT: Vec2 = Vec2::new(-0.866025, -0.5);
pub const HEX_UP_LEFT: Vec2 = Vec2::new(-0.866025, 0.5);
pub const HEX_UP: Vec2 = Vec2::new(0.0, 1.0);
pub const HEX_UP_RIGHT: Vec2 = Vec2::new(0.866025, 0.5);
pub const HEX_DOWN_RIGHT: Vec2 = Vec2::new(0.866025, -0.5);

pub const HEX_DOWN_LEFT_GAP: Vec2 = Vec2::new(-0.5, -0.866025); // Str<Gap>StrDex
pub const HEX_LEFT: Vec2 = Vec2::new(-1.0, 0.0); // StrDex<Gap>Dex
pub const HEX_UP_LEFT_GAP: Vec2 = Vec2::new(-0.5, 0.866025); // Dex<Gap>DexInt
pub const HEX_UP_RIGHT_GAP: Vec2 = Vec2::new(0.5, 0.866025); // DexInt<Gap>Int
pub const HEX_RIGHT: Vec2 = Vec2::new(1.0, 0.0); // Int<Gap>IntStr
pub const HEX_DOWN_RIGHT_GAP: Vec2 = Vec2::new(-0.5, -0.866025); // IntStr<Gap>Str

pub const HEX_NODES: usize = 7;

pub trait HexGrid {
    fn unit_position(&self) -> Vec2;
    //    fn get_neighbour(&self, other: &Self) -> &Self;
}

#[derive(Ser, De, Debug, Clone, Copy)]
pub enum HexPosition {
    Down,
    DownLeft,
    UpLeft,
    Up,
    UpRight,
    DownRight,
    Inner,
}

impl HexGrid for HexPosition {
    fn unit_position(&self) -> Vec2 {
        match self {
            Self::Inner => HEX_INNER,
            Self::Down => HEX_DOWN,
            Self::DownLeft => HEX_DOWN_LEFT,
            Self::UpLeft => HEX_UP_LEFT,
            Self::Up => HEX_UP,
            Self::UpRight => HEX_UP_RIGHT,
            Self::DownRight => HEX_DOWN_RIGHT,
        }
    }
}

impl HexPosition {
    pub fn is_ring_pos(&self) -> bool {
        !matches!(self, Self::Inner)
    }

    pub fn is_even(&self) -> bool {
        self.to_index() % 2 == 0
    }

    pub fn to_index(&self) -> usize {
        match self {
            Self::Down => 0,
            Self::DownLeft => 1,
            Self::UpLeft => 2,
            Self::Up => 3,
            Self::UpRight => 4,
            Self::DownRight => 5,
            Self::Inner => 6,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Down,
            1 => Self::DownLeft,
            2 => Self::UpLeft,
            3 => Self::Up,
            4 => Self::UpRight,
            5 => Self::DownRight,
            _ => Self::Inner,
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::DownLeft => Self::UpRight,
            Self::UpLeft => Self::DownRight,
            Self::Up => Self::Down,
            Self::UpRight => Self::DownLeft,
            Self::DownRight => Self::UpLeft,
            Self::Inner => Self::Inner,
        }
    }
}

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug, Ser, De)]
pub struct NodeId(pub u16);

impl NodeId {
    pub fn from_coordinates(zone: u16, section: u16, cluster: u16, node: u16) -> Self {
        Self(
            zone * ZONE_NODES as u16
                + section * SECTION_NODES as u16
                + cluster * CLUSTER_NODES as u16
                + node,
        )
    }

    pub fn get_zone(&self) -> u16 {
        self.0 % ZONE_NODES as u16
    }

    pub fn get_section(&self) -> u16 {
        self.0 % SECTION_NODES as u16
    }

    pub fn get_cluster(&self) -> u16 {
        self.0 % CLUSTER_NODES as u16
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Ser, De, PartialEq, Eq, Clone, Copy)]
pub enum NodeKind {
    Root,
    Minor,
    Major,
}

#[derive(Ser, De, Debug, Clone)]
pub struct Node {
    pub name: Cow<'static, str>,
    pub id: NodeId,
    pub kind: NodeKind,
    pub position: Vec2,
    pub connections: Vec<NodeId>,
    pub modifiers: Option<Vec<Stat>>,
}

impl Node {
    pub fn get_size(&self, node_info: &NodeInfo) -> f32 {
        match self.kind {
            NodeKind::Root => node_info.root_size,
            NodeKind::Major => node_info.major_size,
            NodeKind::Minor => node_info.minor_size,
        }
    }
}

#[derive(Default, Debug, Clone, Ser, De)]
pub struct NodeInfo {
    pub root_size: f32,
    pub major_size: f32,
    pub minor_size: f32,
}

#[derive(Default, Clone, Ser, De, Debug)]
pub struct PassiveTreeTable {
    pub node_info: NodeInfo,
    pub nodes: Vec<Node>,
    pub graph: Graph<NodeId, u32, Undirected>,
    pub graph_indices: HashMap<NodeId, NodeIndex>,
    pub passive_indices: HashMap<NodeId, usize>,
    //pub edge_indices: HashMap<EdgeNodes, EdgeIndex>,
}

impl PassiveTreeTable {
    pub fn build_graph(&mut self, metadata: &Metadata) {
        let mut graph = Graph::<NodeId, u32, Undirected>::new_undirected();
        let mut graph_indices = HashMap::<NodeId, NodeIndex>::new();
        let mut passive_indices = HashMap::<NodeId, usize>::new();

        // build indices
        for node in &metadata.passive_tree.nodes {
            let index = graph.add_node(node.id);
            graph_indices.insert(node.id, index);
            passive_indices.insert(node.id, graph_indices.len() - 1);
        }

        // add edges
        for node in &metadata.passive_tree.nodes {
            for connection in node.connections.iter() {
                let lhs = node.id.min(*connection);
                let rhs = node.id.max(*connection);
                graph.add_edge(graph_indices[&lhs], graph_indices[&rhs], 1);

                //edge_indices.insert(EdgeNodes { lhs, rhs }, connection_index);
            }
        }

        self.graph = graph;
        self.graph_indices = graph_indices;
        self.passive_indices = passive_indices;
    }
}

#[derive(Ser, De, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeNodes {
    pub lhs: NodeId,
    pub rhs: NodeId,
}

#[derive(Ser, De, Debug)]
pub struct PassiveSkillGraph {
    pub allocated_nodes: Vec<NodeId>,
    pub allocated_edges: Vec<EdgeNodes>,
}
