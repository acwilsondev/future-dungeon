use serde::{Deserialize, Serialize};
use crate::components::FactionKind;
use crate::components::Personality;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawMonster {
    pub name: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
    pub viewshed: i32,
    pub spawn_chance: f32,
    pub min_floor: u16,
    pub max_floor: u16,
    pub personality: Personality,
    pub faction: FactionKind,
    pub xp_reward: i32,
    pub ranged: Option<u16>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawItem {
    pub name: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub spawn_chance: f32,
    pub min_floor: u16,
    pub max_floor: u16,
    pub price: i32,
    pub potion: Option<i32>,
    pub weapon: Option<i32>,
    pub armor: Option<i32>,
    pub ranged: Option<i32>,
    pub ranged_weapon: Option<(i32, i32)>,
    pub aoe: Option<i32>,
    pub confusion: Option<i32>,
    pub poison: Option<(i32, i32)>,
    pub ammo: bool,
    pub consumable: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Content {
    pub monsters: Vec<RawMonster>,
    pub items: Vec<RawItem>,
}
