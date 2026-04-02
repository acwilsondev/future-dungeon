use serde::{Deserialize, Serialize};
use bracket_pathfinding::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Map {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<TileType>,
    pub revealed: Vec<bool>,
    #[serde(skip)]
    pub visible: Vec<bool>,
    #[serde(skip)]
    pub blocked: Vec<bool>,
    #[serde(skip)]
    pub opaque: Vec<bool>,
    #[serde(skip)]
    pub light: Vec<f32>,
    #[serde(skip)]
    pub sound: Vec<f32>,
}

impl Map {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            tiles: vec![TileType::Wall; (width * height) as usize],
            revealed: vec![false; (width * height) as usize],
            visible: vec![false; (width * height) as usize],
            blocked: vec![false; (width * height) as usize],
            opaque: vec![false; (width * height) as usize],
            light: vec![0.0; (width * height) as usize],
            sound: vec![0.0; (width * height) as usize],
        }
    }

    pub fn get_tile(&self, x: u16, y: u16) -> TileType {
        if x >= self.width || y >= self.height {
            return TileType::Wall;
        }
        self.tiles[(y * self.width + x) as usize]
    }

    pub fn populate_blocked_and_opaque(&mut self) {
        let size = self.width as usize * self.height as usize;
        self.blocked = vec![false; size];
        self.opaque = vec![false; size];
        for (i, tile) in self.tiles.iter().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
            self.opaque[i] = *tile == TileType::Wall;
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        if idx >= self.opaque.len() { return true; }
        self.opaque[idx]
    }
}
