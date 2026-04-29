use crate::components::{FactionKind, Personality};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawBossPhase {
    pub hp_threshold_pct: f32, // 0.0 to 1.0
    pub action: crate::components::BossPhaseAction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawMonster {
    pub name: String,
    #[serde(default)]
    pub description: String,
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
    pub confusion: Option<i32>,
    pub poison: Option<(i32, i32)>,
    pub is_boss: Option<bool>,
    pub phases: Option<Vec<RawBossPhase>>,
    pub guaranteed_loot: Option<String>,
    pub branches: Option<Vec<String>>,
    pub biomes: Option<Vec<crate::components::Biome>>,
    #[serde(default)]
    pub tags: Vec<String>,
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

/// Struct form of a ranged weapon in content YAML. Accepts either the
/// legacy 3-tuple `[range, increment, damage_bonus]` via a custom
/// deserializer, or the struct form with optional power-source / heat
/// fields introduced in v0.9-gunplay.
#[derive(Serialize, Clone, Debug)]
pub struct RawRangedWeapon {
    pub range: i32,
    pub range_increment: i32,
    pub damage_bonus: i32,
    #[serde(default)]
    pub power_source: Option<String>,
    #[serde(default)]
    pub heat_capacity: Option<u32>,
    #[serde(default)]
    pub heat_per_shot: Option<u32>,
    #[serde(default)]
    pub efficient_cooldown: bool,
    #[serde(default)]
    pub burst_count: Option<u32>,
    #[serde(default)]
    pub scatter: bool,
    #[serde(default)]
    pub shredding: bool,
    #[serde(default)]
    pub tachyonic: bool,
    #[serde(default)]
    pub element: Option<String>,
}

impl<'de> Deserialize<'de> for RawRangedWeapon {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Form {
            Tuple(i32, i32, i32),
            Full {
                range: i32,
                range_increment: i32,
                damage_bonus: i32,
                #[serde(default)]
                power_source: Option<String>,
                #[serde(default)]
                heat_capacity: Option<u32>,
                #[serde(default)]
                heat_per_shot: Option<u32>,
                #[serde(default)]
                efficient_cooldown: bool,
                #[serde(default)]
                burst_count: Option<u32>,
                #[serde(default)]
                scatter: bool,
                #[serde(default)]
                shredding: bool,
                #[serde(default)]
                tachyonic: bool,
                #[serde(default)]
                element: Option<String>,
            },
        }

        let form = Form::deserialize(deserializer)?;
        Ok(match form {
            Form::Tuple(range, range_increment, damage_bonus) => RawRangedWeapon {
                range,
                range_increment,
                damage_bonus,
                power_source: None,
                heat_capacity: None,
                heat_per_shot: None,
                efficient_cooldown: false,
                burst_count: None,
                scatter: false,
                shredding: false,
                tachyonic: false,
                element: None,
            },
            Form::Full {
                range,
                range_increment,
                damage_bonus,
                power_source,
                heat_capacity,
                heat_per_shot,
                efficient_cooldown,
                burst_count,
                scatter,
                shredding,
                tachyonic,
                element,
            } => RawRangedWeapon {
                range,
                range_increment,
                damage_bonus,
                power_source,
                heat_capacity,
                heat_per_shot,
                efficient_cooldown,
                burst_count,
                scatter,
                shredding,
                tachyonic,
                element,
            },
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RawItem {
    pub name: String,
    #[serde(default)]
    pub description: String,
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
    pub ranged_weapon: Option<RawRangedWeapon>,
    pub aoe: Option<i32>,
    pub confusion: Option<i32>,
    pub poison: Option<(i32, i32)>,
    pub ammo: bool,
    pub consumable: bool,
    pub obfuscated_name: Option<String>,
    pub cursed: Option<bool>,
    pub slot: Option<crate::components::EquipmentSlot>,
    pub branches: Option<Vec<String>>,
    pub biomes: Option<Vec<crate::components::Biome>>,
    pub light: Option<RawLightSource>,
    #[serde(default)]
    pub levitation: bool,
    #[serde(default)]
    pub regeneration: bool,
    #[serde(default)]
    pub heavy_ammo: bool,
    #[serde(default)]
    pub stack: Option<u32>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawManaCost {
    pub orange: u32,
    pub purple: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawTargetSpec {
    pub range: Option<u32>,
    pub selection: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawStatusEffect {
    #[serde(rename = "type")]
    pub status_type: String,
    pub duration: Option<u32>,
    #[serde(default)]
    pub magnitude: Option<String>,
    #[serde(default)]
    pub recovery_save: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawSpellEffect {
    #[serde(rename = "type")]
    pub effect_type: String,
    pub shape: String,
    #[serde(default)]
    pub radius: Option<u32>,
    #[serde(default)]
    pub application_save: Option<String>,
    #[serde(default, rename = "damageType")]
    pub damage_type: Option<String>,
    #[serde(default)]
    pub status: Option<RawStatusEffect>,
    #[serde(default)]
    pub magnitude: Option<String>,
    #[serde(default, rename = "statusType")]
    pub status_type: Option<String>,
    #[serde(default, rename = "xComponent")]
    pub x_component: Option<i32>,
    #[serde(default, rename = "yComponent")]
    pub y_component: Option<i32>,
    #[serde(default, rename = "entityType")]
    pub entity_type: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawSpell {
    pub title: String,
    pub description: String,
    pub mana_cost: RawManaCost,
    pub targeting: RawTargetSpec,
    pub effects: Vec<RawSpellEffect>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawPlayerDefaults {
    pub max_hp: i32,
    pub defense: i32,
    pub power: i32,
    pub viewshed: i32,
    pub hearing_range: i32,
    pub light_range: i32,
    pub aegis: i32,
    pub str: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
}

impl Default for RawPlayerDefaults {
    fn default() -> Self {
        Self {
            max_hp: 30,
            defense: 2,
            power: 5,
            viewshed: 8,
            hearing_range: 15,
            light_range: 2,
            aegis: 5,
            str: 10,
            dex: 10,
            con: 10,
            int: 10,
            wis: 10,
            cha: 10,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoreSnippet {
    pub id: String,
    pub text: String,
    pub faction: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawFeature {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    /// Spawns a Door component (opens on player contact).
    #[serde(default)]
    pub door: bool,
    /// Spawns a PartialCover component (ranged attack penalty for adjacent targets).
    #[serde(default)]
    pub cover: bool,
    /// Spawns a Trap with this spike damage. Hidden until triggered.
    #[serde(default)]
    pub trap_damage: Option<i32>,
    /// Spawns a Poison component with this damage-per-turn. Implies a visible trap.
    #[serde(default)]
    pub poison_damage: Option<i32>,
    /// Duration in turns for the poison effect (default 5).
    #[serde(default)]
    pub poison_turns: Option<i32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub branches: Option<Vec<String>>,
    pub spawn_chance: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FloorTrigger {
    /// Match exactly this floor number.
    pub at: Option<u16>,
    /// Match every N floors (floor % every == offset).
    pub every: Option<u16>,
    /// Offset for the every trigger (default 0).
    #[serde(default)]
    pub offset: u16,
}

impl FloorTrigger {
    pub fn matches(&self, floor: u16) -> bool {
        if let Some(at) = self.at {
            return floor == at;
        }
        if let Some(every) = self.every {
            return floor > 0 && floor % every == self.offset;
        }
        false
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FloorEventKind {
    /// Replace the floor with a safe merchant area; no monsters are spawned.
    MerchantHaven,
    /// Spawn a Reset Shrine in a random room.
    ResetShrine,
    /// Spawn the Amulet of the Ancients (win-condition item).
    AmuletSpawn,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawFloorEvent {
    pub trigger: FloorTrigger,
    pub kind: FloorEventKind,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Content {
    #[serde(default)]
    pub monsters: Vec<RawMonster>,
    #[serde(default)]
    pub items: Vec<RawItem>,
    #[serde(default)]
    pub spells: Vec<RawSpell>,
    #[serde(default)]
    pub lore: Vec<LoreSnippet>,
    #[serde(default)]
    pub features: Vec<RawFeature>,
    #[serde(default)]
    pub floor_events: Vec<RawFloorEvent>,
    #[serde(default)]
    pub player: Option<RawPlayerDefaults>,
}
