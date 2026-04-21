use crate::components::{
    Attribute, BakedStatusEffect, DamageType, Dice, EffectInstruction, EffectMetadata,
    EffectOpCode, EffectShape, FactionKind, ManaCost, Personality, Spell, TargetSelection,
    TargetSpec,
};
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
    pub confusion: Option<i32>,
    pub poison: Option<(i32, i32)>,
    pub is_boss: Option<bool>,
    pub phases: Option<Vec<RawBossPhase>>,
    pub guaranteed_loot: Option<String>,
    pub branches: Option<Vec<String>>,
    pub biomes: Option<Vec<crate::components::Biome>>,
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

/// Struct form of a ranged weapon in `content.json`. Accepts either the
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

impl RawRangedWeapon {
    pub fn power_source(&self) -> anyhow::Result<crate::components::WeaponPowerSource> {
        use crate::components::WeaponPowerSource;
        match self.power_source.as_deref() {
            None | Some("ammo") => Ok(WeaponPowerSource::Ammo),
            Some("heavy") => Ok(WeaponPowerSource::HeavyAmmo),
            Some("heat") => Ok(WeaponPowerSource::Heat),
            Some(other) => anyhow::bail!("unknown weapon power source: {}", other),
        }
    }

    pub fn element_type(&self) -> anyhow::Result<Option<crate::components::DamageType>> {
        match self.element.as_deref() {
            None => Ok(None),
            Some(s) => parse_damage_type(s).map(Some),
        }
    }
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Content {
    pub monsters: Vec<RawMonster>,
    pub items: Vec<RawItem>,
    #[serde(default)]
    pub spells: Vec<RawSpell>,
}

const REQUIRED_ITEMS: &[&str] = &["Amulet of the Ancients", "Identification Scroll"];

impl Content {
    pub fn load_from_str(s: &str) -> anyhow::Result<Self> {
        let content: Self = serde_json::from_str(s)?;
        content.validate()?;
        Ok(content)
    }

    pub fn load_from_path(path: &str) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(path)?;
        Self::load_from_str(&s)
    }

    pub fn load() -> anyhow::Result<Self> {
        Self::load_from_path("content.json")
    }

    fn validate(&self) -> anyhow::Result<()> {
        for name in REQUIRED_ITEMS {
            if !self.items.iter().any(|i| i.name == *name) {
                anyhow::bail!("content.json is missing required item: \"{}\"", name);
            }
        }
        for raw in &self.spells {
            raw.validate()?;
        }
        Ok(())
    }

    /// Bake all spells into ECS-ready components.
    #[allow(dead_code)]
    pub fn bake_spells(&self) -> anyhow::Result<Vec<Spell>> {
        self.spells.iter().map(|r| r.bake()).collect()
    }

    pub fn find_spell(&self, name: &str) -> anyhow::Result<Spell> {
        for raw in &self.spells {
            if raw.title == name {
                return raw.bake();
            }
        }
        anyhow::bail!("spell not found: {}", name)
    }
}

pub fn parse_dice_string(s: &str) -> anyhow::Result<Dice> {
    let trimmed = s.trim();
    // Flat integer form: "50"
    if let Ok(flat) = trimmed.parse::<i32>() {
        return Ok(Dice::flat(flat));
    }
    // Full form: "2d6+3" or "2d6-1" or "1d10"
    let lower = trimmed.to_ascii_lowercase();
    let (count_str, rest) = match lower.split_once('d') {
        Some(parts) => parts,
        None => anyhow::bail!("invalid dice string: {}", s),
    };
    let count: u32 = count_str
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid dice count in {}", s))?;
    let (sides, bonus): (u32, i32) = if let Some(idx) = rest.find(['+', '-']) {
        let sides: u32 = rest[..idx]
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid dice sides in {}", s))?;
        let bonus: i32 = rest[idx..]
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid dice bonus in {}", s))?;
        (sides, bonus)
    } else {
        let sides: u32 = rest
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid dice sides in {}", s))?;
        (sides, 0)
    };
    Ok(Dice {
        count,
        sides,
        bonus,
    })
}

fn parse_attribute(s: &str) -> anyhow::Result<Attribute> {
    match s.to_ascii_uppercase().as_str() {
        "STR" => Ok(Attribute::Strength),
        "DEX" => Ok(Attribute::Dexterity),
        "CON" => Ok(Attribute::Constitution),
        "INT" => Ok(Attribute::Intelligence),
        "WIS" => Ok(Attribute::Wisdom),
        "CHA" => Ok(Attribute::Charisma),
        _ => anyhow::bail!("unknown attribute: {}", s),
    }
}

fn parse_selection(s: &str) -> anyhow::Result<TargetSelection> {
    match s.to_ascii_lowercase().as_str() {
        "entity" => Ok(TargetSelection::Entity),
        "self" => Ok(TargetSelection::SelfCast),
        "location" => Ok(TargetSelection::Location),
        _ => anyhow::bail!("unknown target selection: {}", s),
    }
}

fn parse_opcode(s: &str) -> anyhow::Result<EffectOpCode> {
    match s {
        "DealDamage" => Ok(EffectOpCode::DealDamage),
        "GrantStatus" => Ok(EffectOpCode::GrantStatus),
        "RemoveStatus" => Ok(EffectOpCode::RemoveStatus),
        "Heal" => Ok(EffectOpCode::Heal),
        "Push" => Ok(EffectOpCode::Push),
        "Teleport" => Ok(EffectOpCode::Teleport),
        "CreateEntity" => Ok(EffectOpCode::CreateEntity),
        _ => anyhow::bail!("unknown effect opcode: {}", s),
    }
}

fn parse_shape(s: &str) -> anyhow::Result<EffectShape> {
    match s.to_ascii_lowercase().as_str() {
        "point" => Ok(EffectShape::Point),
        "circle" => Ok(EffectShape::Circle),
        _ => anyhow::bail!("unknown effect shape: {}", s),
    }
}

fn parse_damage_type(s: &str) -> anyhow::Result<DamageType> {
    match s {
        "Fire" => Ok(DamageType::Fire),
        "Poison" => Ok(DamageType::Poison),
        "Bludgeoning" => Ok(DamageType::Bludgeoning),
        "Slashing" => Ok(DamageType::Slashing),
        "Piercing" => Ok(DamageType::Piercing),
        "Necrotic" => Ok(DamageType::Necrotic),
        _ => anyhow::bail!("unknown damage type: {}", s),
    }
}

impl RawStatusEffect {
    pub fn bake(&self) -> anyhow::Result<BakedStatusEffect> {
        let magnitude = match &self.magnitude {
            Some(m) => Some(parse_dice_string(m)?),
            None => None,
        };
        let recovery_save = match &self.recovery_save {
            Some(s) => Some(parse_attribute(s)?),
            None => None,
        };
        Ok(BakedStatusEffect {
            status_type: self.status_type.clone(),
            duration: self.duration,
            magnitude,
            recovery_save,
        })
    }
}

impl RawSpell {
    pub fn validate(&self) -> anyhow::Result<()> {
        let level = self.mana_cost.orange + self.mana_cost.purple;
        if self.mana_cost.orange > 0 && self.mana_cost.purple > 0 {
            anyhow::bail!(
                "Spell '{}': mixed orange+purple cost is invalid",
                self.title
            );
        }
        if level == 0 {
            anyhow::bail!("Spell '{}': level-0 (free) spells are invalid", self.title);
        }
        Ok(())
    }

    pub fn bake(&self) -> anyhow::Result<Spell> {
        self.validate()?;
        let level = self.mana_cost.orange + self.mana_cost.purple;

        let mut instructions = Vec::with_capacity(self.effects.len());
        for e in &self.effects {
            let opcode = parse_opcode(&e.effect_type)?;
            let shape = parse_shape(&e.shape)?;
            let magnitude = match &e.magnitude {
                Some(m) => Some(parse_dice_string(m)?),
                None => None,
            };
            let application_save = match &e.application_save {
                Some(s) => Some(parse_attribute(s)?),
                None => None,
            };

            let metadata = match opcode {
                EffectOpCode::DealDamage => {
                    let dt = e
                        .damage_type
                        .as_deref()
                        .ok_or_else(|| anyhow::anyhow!("DealDamage requires damageType"))?;
                    EffectMetadata::Damage(parse_damage_type(dt)?)
                }
                EffectOpCode::GrantStatus => {
                    let s = e
                        .status
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("GrantStatus requires status"))?;
                    EffectMetadata::Status(s.bake()?)
                }
                EffectOpCode::RemoveStatus => {
                    let t = e
                        .status_type
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("RemoveStatus requires statusType"))?;
                    EffectMetadata::RemoveStatus(t)
                }
                EffectOpCode::Push | EffectOpCode::Teleport => EffectMetadata::Vector {
                    x: e.x_component.unwrap_or(0),
                    y: e.y_component.unwrap_or(0),
                },
                EffectOpCode::CreateEntity => {
                    let t = e
                        .entity_type
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("CreateEntity requires entityType"))?;
                    EffectMetadata::CreateEntity(t)
                }
                EffectOpCode::Heal => EffectMetadata::None,
            };

            instructions.push(EffectInstruction {
                opcode,
                shape,
                radius: e.radius,
                application_save,
                magnitude,
                metadata,
            });
        }

        Ok(Spell {
            title: self.title.clone(),
            description: self.description.clone(),
            mana_cost: ManaCost {
                orange: self.mana_cost.orange,
                purple: self.mana_cost.purple,
            },
            level,
            targeting: TargetSpec {
                range: self.targeting.range,
                selection: parse_selection(&self.targeting.selection)?,
            },
            instructions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nonexistent_file_returns_err() {
        let result = Content::load_from_path("this_file_does_not_exist_xyz.json");
        assert!(result.is_err());
    }

    #[test]
    fn test_load_bad_json_returns_err() {
        let result = Content::load_from_str("{ not valid json ]]]");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_item_returns_err() {
        let json = r#"{"monsters":[],"items":[]}"#;
        let result = Content::load_from_str(json);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Amulet of the Ancients"));
    }

    fn firebolt_raw() -> RawSpell {
        RawSpell {
            title: "Firebolt".to_string(),
            description: "A small flick of flame.".to_string(),
            mana_cost: RawManaCost {
                orange: 1,
                purple: 0,
            },
            targeting: RawTargetSpec {
                range: Some(6),
                selection: "entity".to_string(),
            },
            effects: vec![RawSpellEffect {
                effect_type: "DealDamage".to_string(),
                shape: "point".to_string(),
                radius: None,
                application_save: Some("DEX".to_string()),
                damage_type: Some("Fire".to_string()),
                status: None,
                magnitude: Some("1d10".to_string()),
                status_type: None,
                x_component: None,
                y_component: None,
                entity_type: None,
            }],
        }
    }

    #[test]
    fn test_parse_dice_string_full() {
        let d = parse_dice_string("2d6+3").unwrap();
        assert_eq!(d.count, 2);
        assert_eq!(d.sides, 6);
        assert_eq!(d.bonus, 3);
    }

    #[test]
    fn test_parse_dice_string_no_bonus() {
        let d = parse_dice_string("1d10").unwrap();
        assert_eq!(d.count, 1);
        assert_eq!(d.sides, 10);
        assert_eq!(d.bonus, 0);
    }

    #[test]
    fn test_parse_dice_string_negative_bonus() {
        let d = parse_dice_string("3d4-2").unwrap();
        assert_eq!(d.count, 3);
        assert_eq!(d.sides, 4);
        assert_eq!(d.bonus, -2);
    }

    #[test]
    fn test_parse_dice_string_flat() {
        let d = parse_dice_string("50").unwrap();
        assert_eq!(d.count, 0);
        assert_eq!(d.sides, 0);
        assert_eq!(d.bonus, 50);
    }

    #[test]
    fn test_parse_dice_string_invalid() {
        assert!(parse_dice_string("hello").is_err());
    }

    #[test]
    fn test_bake_firebolt() {
        let baked = firebolt_raw().bake().unwrap();
        assert_eq!(baked.title, "Firebolt");
        assert_eq!(baked.mana_cost.orange, 1);
        assert_eq!(baked.level, 1);
        assert_eq!(baked.targeting.selection, TargetSelection::Entity);
        assert_eq!(baked.instructions.len(), 1);
        assert_eq!(baked.instructions[0].opcode, EffectOpCode::DealDamage);
        if let EffectMetadata::Damage(t) = &baked.instructions[0].metadata {
            assert_eq!(*t, DamageType::Fire);
        } else {
            panic!("expected Damage metadata");
        }
    }

    #[test]
    fn test_bake_rejects_mixed_color() {
        let mut raw = firebolt_raw();
        raw.mana_cost.orange = 1;
        raw.mana_cost.purple = 1;
        assert!(raw.bake().is_err());
    }

    #[test]
    fn test_live_content_spells_bake() {
        let content =
            Content::load_from_path("content.json").expect("content.json must load cleanly");
        for raw in &content.spells {
            raw.bake()
                .unwrap_or_else(|e| panic!("spell '{}' failed to bake: {}", raw.title, e));
        }
        for name in &["Magic Missile", "Fireball", "Venom Dart"] {
            assert!(
                content.spells.iter().any(|s| s.title == *name),
                "missing spell: {name}"
            );
        }
    }

    #[test]
    fn test_bake_rejects_level_zero() {
        let mut raw = firebolt_raw();
        raw.mana_cost.orange = 0;
        raw.mana_cost.purple = 0;
        assert!(raw.bake().is_err());
    }
}
