use crate::map::{Map, TileType};
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

pub enum LevelTheme {
    Rooms,
    Caves,
    BossArena,
}

pub struct MapBuilder {
    pub map: Map,
    pub rooms: Vec<Rect>,
    pub player_start: (u16, u16),
    pub monster_spawns: Vec<(u16, u16)>,
    pub boss_spawn: Option<(u16, u16)>,
    pub item_spawns: Vec<(u16, u16)>,
    pub door_spawns: Vec<(u16, u16)>,
    pub trap_spawns: Vec<(u16, u16)>,
    pub stairs_down: (u16, u16),
    pub stairs_up: (u16, u16),
    pub theme: LevelTheme,
}

impl MapBuilder {
    pub fn new(width: u16, height: u16) -> Self {
        let map = Map::new(width, height);
        Self { 
            map, 
            rooms: Vec::new(), 
            player_start: (0, 0), 
            monster_spawns: Vec::new(), 
            boss_spawn: None,
            item_spawns: Vec::new(),
            door_spawns: Vec::new(),
            trap_spawns: Vec::new(),
            stairs_down: (0, 0),
            stairs_up: (0, 0),
            theme: LevelTheme::Rooms,
        }
    }

    pub fn build(&mut self, depth: u16) {
        if depth == 3 || depth == 6 {
            self.theme = LevelTheme::BossArena;
        } else if depth % 3 == 0 {
            self.theme = LevelTheme::Caves;
        } else {
            self.theme = LevelTheme::Rooms;
        }

        match self.theme {
            LevelTheme::Rooms => {
                self.build_rooms();
                self.place_doors();
            }
            LevelTheme::Caves => self.build_caves(),
            LevelTheme::BossArena => self.build_boss_arena(),
        }
    }

    fn build_boss_arena(&mut self) {
        // Large central room
        let w = 20;
        let h = 15;
        let x = (self.map.width as i32 - w) / 2;
        let y = (self.map.height as i32 - h) / 2;
        let arena = Rect::new(x, y, w, h);
        self.apply_room_to_map(&arena);
        self.rooms.push(arena);

        // Pillars
        for py in [y + 4, y + h - 5] {
            for px in [x + 5, x + w - 6] {
                let idx = (py as u16 * self.map.width + px as u16) as usize;
                self.map.tiles[idx] = TileType::Wall;
            }
        }

        let center = arena.center();
        self.player_start = (arena.x1 as u16 + 2, center.1 as u16);
        self.stairs_up = (arena.x1 as u16 + 1, center.1 as u16);
        self.boss_spawn = Some((arena.x2 as u16 - 5, center.1 as u16));
        self.stairs_down = (arena.x2 as u16 - 2, center.1 as u16);
    }

    fn is_legal_door(&self, x: u16, y: u16) -> bool {
        if x == 0 || x >= self.map.width - 1 || y == 0 || y >= self.map.height - 1 {
            return false;
        }

        // Must be a floor tile (from tunnel)
        if self.map.get_tile(x, y) != TileType::Floor {
            return false;
        }

        let left = self.map.get_tile(x - 1, y);
        let right = self.map.get_tile(x + 1, y);
        let up = self.map.get_tile(x, y - 1);
        let down = self.map.get_tile(x, y + 1);

        // Rule: 2 orthogonal wall tiles AND 2 orthogonal floor tiles
        // Specifically, they must be opposite to be a door in a wall.
        if (left == TileType::Wall && right == TileType::Wall && up == TileType::Floor && down == TileType::Floor) ||
           (left == TileType::Floor && right == TileType::Floor && up == TileType::Wall && down == TileType::Wall) {
            return true;
        }

        false
    }

    fn place_doors(&mut self) {
        let mut rng = rand::thread_rng();
        let mut candidates = Vec::new();

        for room in &self.rooms {
            // Check boundaries of the room for potential door spots
            // Top wall
            for x in room.x1..room.x2 {
                if self.is_legal_door(x as u16, (room.y1 - 1) as u16) {
                    candidates.push((x as u16, (room.y1 - 1) as u16));
                }
            }
            // Bottom wall
            for x in room.x1..room.x2 {
                if self.is_legal_door(x as u16, room.y2 as u16) {
                    candidates.push((x as u16, room.y2 as u16));
                }
            }
            // Left wall
            for y in room.y1..room.y2 {
                if self.is_legal_door((room.x1 - 1) as u16, y as u16) {
                    candidates.push(((room.x1 - 1) as u16, y as u16));
                }
            }
            // Right wall
            for y in room.y1..room.y2 {
                if self.is_legal_door(room.x2 as u16, y as u16) {
                    candidates.push((room.x2 as u16, y as u16));
                }
            }
        }

        candidates.sort();
        candidates.dedup();

        for (x, y) in candidates {
            if rng.gen_bool(0.7) {
                self.door_spawns.push((x, y));
            }
        }
    }

    fn build_rooms(&mut self) {
        let mut rng = rand::thread_rng();
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 4;
        const MAX_SIZE: i32 = 10;

        for i in 0..MAX_ROOMS {
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
                if i > 0 && rng.gen_bool(0.15) {
                    self.spawn_vault(&new_room);
                } else {
                    self.apply_room_to_map(&new_room);
                }

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

                    let center = new_room.center();
                    if rng.gen_bool(0.7) {
                        self.monster_spawns.push((center.0 as u16, center.1 as u16));
                    }
                    if rng.gen_bool(0.5) {
                        let item_x = rng.gen_range(new_room.x1..new_room.x2) as u16;
                        let item_y = rng.gen_range(new_room.y1..new_room.y2) as u16;
                        self.item_spawns.push((item_x, item_y));
                    }
                    if rng.gen_bool(0.2) {
                        let trap_x = rng.gen_range(new_room.x1..new_room.x2) as u16;
                        let trap_y = rng.gen_range(new_room.y1..new_room.y2) as u16;
                        self.trap_spawns.push((trap_x, trap_y));
                    }
                } else {
                    let start = new_room.center();
                    self.player_start = (start.0 as u16, start.1 as u16);
                    self.stairs_up = (start.0 as u16, start.1 as u16);
                }
                self.rooms.push(new_room);
            }
        }

        let end = self.rooms[self.rooms.len() - 1].center();
        self.stairs_down = (end.0 as u16, end.1 as u16);
    }

    fn spawn_vault(&mut self, rect: &Rect) {
        for y in rect.y1..rect.y2 {
            for x in rect.x1..rect.x2 {
                let idx = (y as u16 * self.map.width + x as u16) as usize;
                self.map.tiles[idx] = TileType::Floor;
            }
        }
        let center = rect.center();
        let idx = (center.1 as u16 * self.map.width + center.0 as u16) as usize;
        self.map.tiles[idx] = TileType::Wall;
        
        self.monster_spawns.push((center.0 as u16 + 1, center.1 as u16));
        self.monster_spawns.push((center.0 as u16 - 1, center.1 as u16));
        self.item_spawns.push((center.0 as u16, center.1 as u16 + 1));
    }

    fn build_caves(&mut self) {
        let mut rng = rand::thread_rng();
        for tile in self.map.tiles.iter_mut() {
            if rng.gen_bool(0.45) { *tile = TileType::Wall; } else { *tile = TileType::Floor; }
        }

        for _ in 0..5 {
            let mut new_tiles = self.map.tiles.clone();
            for y in 1..self.map.height - 1 {
                for x in 1..self.map.width - 1 {
                    let mut neighbors = 0;
                    for iy in -1..=1 {
                        for ix in -1..=1 {
                            if ix == 0 && iy == 0 { continue; }
                            if self.map.tiles[((y as i32 + iy) * self.map.width as i32 + (x as i32 + ix)) as usize] == TileType::Wall {
                                neighbors += 1;
                            }
                        }
                    }
                    let idx = (y * self.map.width + x) as usize;
                    if neighbors > 4 || neighbors == 0 { new_tiles[idx] = TileType::Wall; } 
                    else { new_tiles[idx] = TileType::Floor; }
                }
            }
            self.map.tiles = new_tiles;
        }

        let mut start_pos = (0, 0);
        while self.map.tiles[(start_pos.1 * self.map.width + start_pos.0) as usize] != TileType::Floor {
            start_pos = (rng.gen_range(1..self.map.width - 1), rng.gen_range(1..self.map.height - 1));
        }
        self.player_start = start_pos;
        self.stairs_up = start_pos;

        let mut end_pos = (0, 0);
        while self.map.tiles[(end_pos.1 * self.map.width + end_pos.0) as usize] != TileType::Floor || end_pos == start_pos {
            end_pos = (rng.gen_range(1..self.map.width - 1), rng.gen_range(1..self.map.height - 1));
        }
        self.stairs_down = end_pos;

        for _ in 0..20 {
            let x = rng.gen_range(1..self.map.width - 1);
            let y = rng.gen_range(1..self.map.height - 1);
            if self.map.tiles[(y * self.map.width + x) as usize] == TileType::Floor {
                self.monster_spawns.push((x, y));
            }
        }
        for _ in 0..10 {
            let x = rng.gen_range(1..self.map.width - 1);
            let y = rng.gen_range(1..self.map.height - 1);
            if self.map.tiles[(y * self.map.width + x) as usize] == TileType::Floor {
                self.item_spawns.push((x, y));
            }
        }
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
            if idx < self.map.tiles.len() { self.map.tiles[idx] = TileType::Floor; }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = (y as u16 * self.map.width + x as u16) as usize;
            if idx < self.map.tiles.len() { self.map.tiles[idx] = TileType::Floor; }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_builder_rooms() {
        let mut mb = MapBuilder::new(80, 50);
        mb.build(1);
        assert!(!mb.rooms.is_empty() || matches!(mb.theme, LevelTheme::Caves));
    }

    #[test]
    fn test_player_start_on_floor() {
        let mut mb = MapBuilder::new(80, 50);
        mb.build(1);
        let (x, y) = mb.player_start;
        let idx = (y * mb.map.width + x) as usize;
        assert_eq!(mb.map.tiles[idx], TileType::Floor, "Player should start on a floor tile");
    }

    #[test]
    fn test_rect_intersects() {
        let r1 = Rect::new(0, 0, 10, 10);
        let r2 = Rect::new(5, 5, 10, 10);
        let r3 = Rect::new(11, 11, 5, 5);
        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn test_door_placement_legality() {
        let mut mb = MapBuilder::new(80, 50);
        mb.build(1); // Ensure Rooms theme
        if matches!(mb.theme, LevelTheme::Rooms) {
            for (x, y) in &mb.door_spawns {
                assert!(mb.is_legal_door(*x, *y), 
                    "Door at ({}, {}) does not meet legality criteria", x, y);
            }
        }
    }
}
