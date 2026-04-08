use crate::components::{Biome, FloorModifier};
use bracket_pathfinding::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    pub biome: Biome,
    pub floor_modifier: FloorModifier,
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
            biome: Biome::Dungeon,
            floor_modifier: FloorModifier::None,
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

    pub fn reinitialize_skipped_fields(&mut self) {
        let size = (self.width * self.height) as usize;
        self.visible = vec![false; size];
        self.light = vec![0.0; size];
        self.sound = vec![0.0; size];
        self.populate_blocked_and_opaque();
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        if idx >= self.opaque.len() {
            return true;
        }
        self.opaque[idx]
    }
}
