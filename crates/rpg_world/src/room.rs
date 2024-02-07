use crate::{tile::Tile, zone::Zone};

use bevy_math::UVec2;

#[derive(Debug)]
pub struct Room {
    pub position: UVec2,
    pub props: u32,
    pub tiles: Vec<Tile>,
}

impl Room {
    pub fn new(zone: &Zone, position: UVec2, props: u32) -> Self {
        let tile_count: u8 = (zone.size_info.tile_size.x * zone.size_info.tile_size.y) as u8;
        let mut tiles = Vec::with_capacity(tile_count as usize);
        for i in 0..tile_count {
            tiles.push(Tile::new(i));
        }

        Self {
            position,
            props,
            tiles,
        }
    }
}

/*
impl IndexedPosition<u32, UVec2> for Room {
    fn get_position_fixed(&self, zone_info: &ZoneInfo, index: u32) -> UVec2 {
        UVec2::new(
            index / zone_info.room_size.x as u32,
            index % zone_info.room_size.x as u32,
        )
    }

    fn get_position(&self, index: usize) -> UVec2 {
        UVec2::new(
            index as u32 / self.size.x as u32,
            index as u32 % self.size.x,
        )
    }

    fn get_fixed_index(&self, position: UVec2) -> u32 {
        (position.x + position.y * self.size.x) as u32
    }

    fn get_index(&self, position: UVec2) -> usize {
        (position.x + position.y * self.size.x) as usize
    }
}*/
