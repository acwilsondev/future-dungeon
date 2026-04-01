use crate::app::{Map, TileType};
use rand::Rng;
use std::cmp::{max, min};

#[derive(Clone, Copy)]
pub struct Rect {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x1: x, x2: x + w, y1: y, y2: y + h }
    }

    pub fn intersects(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}

pub struct MapBuilder {
    pub map: Map,
    pub rooms: Vec<Rect>,
    pub player_start: (u16, u16),
}

impl MapBuilder {
    pub fn new(width: u16, height: u16) -> Self {
        let map = Map::new(width, height);
        Self { map, rooms: Vec::new(), player_start: (0, 0) }
    }

    pub fn build(&mut self) {
        let mut rng = rand::thread_rng();
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 4;
        const MAX_SIZE: i32 = 10;

        for _ in 0..MAX_ROOMS {
            let w = rng.gen_range(MIN_SIZE..MAX_SIZE);
            let h = rng.gen_range(MIN_SIZE..MAX_SIZE);
            let x = rng.gen_range(1..self.map.width as i32 - w - 1);
            let y = rng.gen_range(1..self.map.height as i32 - h - 1);
            let new_room = Rect::new(x, y, w, h);

            let mut ok = true;
            for other_room in &self.rooms {
                if new_room.intersects(other_room) {
                    ok = false;
                    break;
                }
            }

            if ok {
                self.apply_room_to_map(&new_room);
                if !self.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = self.rooms[self.rooms.len() - 1].center();
                    if rng.gen_bool(0.5) {
                        self.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        self.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        self.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        self.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }
                self.rooms.push(new_room);
            }
        }

        let start = self.rooms[0].center();
        self.player_start = (start.0 as u16, start.1 as u16);
    }

    fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1..room.y2 {
            for x in room.x1..room.x2 {
                let idx = (y as u16 * self.map.width + x as u16) as usize;
                self.map.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = (y as u16 * self.map.width + x as u16) as usize;
            if idx < self.map.tiles.len() {
                self.map.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = (y as u16 * self.map.width + x as u16) as usize;
            if idx < self.map.tiles.len() {
                self.map.tiles[idx] = TileType::Floor;
            }
        }
    }
}
