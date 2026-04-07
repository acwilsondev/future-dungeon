use crate::components::FactionKind;
use crate::components::Personality;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawBossPhase {
    pub hp_threshold_pct: f32, // 0.0 to 1.0
    pub action: crate::components::BossPhaseAction,
}

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
    pub is_boss: Option<bool>,
    pub phases: Option<Vec<RawBossPhase>>,
    pub guaranteed_loot: Option<String>,
    pub branches: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawLightSource {
    pub range: i32,
    pub color: (u8, u8, u8),
    pub turns: Option<i32>,
    pub flicker: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawWeapon {
    pub power_bonus: i32,
    pub weight: crate::components::WeaponWeight,
    pub n_dice: i32,
    pub die_type: i32,
    pub two_handed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawArmor {
    pub defense_bonus: i32,
    pub max_dex_bonus: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RawItem {
    pub name: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub spawn_chance: f32,
    pub min_floor: u16,
    pub max_floor: u16,
    pub price: i32,
    pub potion: Option<i32>,
    pub weapon: Option<RawWeapon>,
    pub armor: Option<RawArmor>,
    pub ranged: Option<i32>,
    pub ranged_weapon: Option<(i32, i32)>,
    pub aoe: Option<i32>,
    pub confusion: Option<i32>,
    pub poison: Option<(i32, i32)>,
    pub ammo: bool,
    pub consumable: bool,
    pub obfuscated_name: Option<String>,
    pub cursed: Option<bool>,
    pub slot: Option<crate::components::EquipmentSlot>,
    pub branches: Option<Vec<String>>,
    pub light: Option<RawLightSource>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Content {
    pub monsters: Vec<RawMonster>,
    pub items: Vec<RawItem>,
}
