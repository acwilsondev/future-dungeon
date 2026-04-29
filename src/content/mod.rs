mod loader;
mod types;

pub use types::*;

use crate::components::{
    Attribute, BakedStatusEffect, DamageType, Dice, EffectInstruction, EffectMetadata,
    EffectOpCode, EffectShape, ManaCost, Spell, TargetSelection, TargetSpec,
};

impl Content {
    pub fn player_defaults(&self) -> std::borrow::Cow<'_, RawPlayerDefaults> {
        match &self.player {
            Some(p) => std::borrow::Cow::Borrowed(p),
            None => std::borrow::Cow::Owned(RawPlayerDefaults::default()),
        }
    }

    pub fn active_floor_events(&self, floor: u16) -> Vec<FloorEventKind> {
        self.floor_events
            .iter()
            .filter(|e| e.trigger.matches(floor))
            .map(|e| e.kind.clone())
            .collect()
    }

    pub fn monsters_by_tag<'a>(
        &'a self,
        tag: &str,
        level: u16,
        branch_str: &str,
        biome: &crate::components::Biome,
    ) -> Vec<&'a RawMonster> {
        self.monsters
            .iter()
            .filter(|m| m.tags.iter().any(|t| t == tag))
            .filter(|m| level >= m.min_floor && level <= m.max_floor)
            .filter(|m| {
                m.branches
                    .as_ref()
                    .is_none_or(|b| b.iter().any(|s| s == branch_str))
            })
            .filter(|m| m.biomes.as_ref().is_none_or(|b| b.contains(biome)))
            .collect()
    }

    #[allow(dead_code)]
    pub fn items_by_tag<'a>(
        &'a self,
        tag: &str,
        level: u16,
        branch_str: &str,
        biome: &crate::components::Biome,
    ) -> Vec<&'a RawItem> {
        self.items
            .iter()
            .filter(|i| i.tags.iter().any(|t| t == tag))
            .filter(|i| level >= i.min_floor && level <= i.max_floor)
            .filter(|i| {
                i.branches
                    .as_ref()
                    .is_none_or(|b| b.iter().any(|s| s == branch_str))
            })
            .filter(|i| i.biomes.as_ref().is_none_or(|b| b.contains(biome)))
            .collect()
    }

    pub fn features_by_tag<'a>(&'a self, tag: &str, branch_str: &str) -> Vec<&'a RawFeature> {
        self.features
            .iter()
            .filter(|f| f.tags.iter().any(|t| t == tag))
            .filter(|f| {
                f.branches
                    .as_ref()
                    .is_none_or(|b| b.iter().any(|s| s == branch_str))
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn lore_by_faction<'a>(&'a self, faction: &str) -> Vec<&'a LoreSnippet> {
        self.lore
            .iter()
            .filter(|ls| ls.faction == faction)
            .collect()
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
    fn test_load_nonexistent_dir_returns_err() {
        let result = Content::load_from_dir(std::path::Path::new("this_dir_does_not_exist_xyz/"));
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

    #[test]
    fn test_features_by_tag_loads_from_dir() {
        let content =
            Content::load_from_dir(std::path::Path::new("content/")).expect("content/ must load");
        assert!(
            !content.features.is_empty(),
            "features should load from YAML"
        );
        let doors = content.features_by_tag("door", "Main");
        assert!(!doors.is_empty(), "should find door-tagged features");
        // Branch filtering: trap in Gardens should not appear for Main
        let main_traps = content.features_by_tag("trap", "Main");
        let garden_traps = content.features_by_tag("trap", "Gardens");
        assert!(!main_traps.is_empty(), "Main traps should exist");
        assert!(!garden_traps.is_empty(), "Garden traps should exist");
        // The garden poison spore should not appear in Main
        assert!(main_traps.iter().all(|f| f.name != "Poison Spore"));
        assert!(garden_traps.iter().all(|f| f.name == "Poison Spore"));
    }

    #[test]
    fn test_monsters_by_tag_filters_correctly() {
        let content =
            Content::load_from_dir(std::path::Path::new("content/")).expect("content/ must load");
        let biome = crate::components::Biome::Dungeon;
        let melee = content.monsters_by_tag("melee", 1, "Main", &biome);
        assert!(!melee.is_empty(), "should find melee-tagged monsters");
        assert!(melee.iter().all(|m| m.tags.contains(&"melee".to_string())));
    }

    #[test]
    fn test_lore_loads_and_filters_by_faction() {
        let content =
            Content::load_from_dir(std::path::Path::new("content/")).expect("content/ must load");
        assert!(!content.lore.is_empty(), "lore snippets should load");
        let nihil = content.lore_by_faction("Nihil");
        assert!(!nihil.is_empty(), "Nihil lore should exist");
        assert!(nihil.iter().all(|l| l.faction == "Nihil"));
    }

    #[test]
    fn test_player_defaults_load() {
        let content =
            Content::load_from_dir(std::path::Path::new("content/")).expect("content/ must load");
        let d = content.player_defaults();
        assert_eq!(d.max_hp, 30);
        assert_eq!(d.viewshed, 8);
        assert_eq!(d.aegis, 5);
    }

    #[test]
    fn test_duplicate_player_section_is_error() {
        use std::fs;
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let yaml_a = r#"
player:
  max_hp: 30
  defense: 2
  power: 5
  viewshed: 8
  hearing_range: 15
  light_range: 2
  aegis: 5
  str: 10
  dex: 10
  con: 10
  int: 10
  wis: 10
  cha: 10
items:
  - name: "Amulet of the Ancients"
    glyph: '"'
    color: [255, 215, 0]
    price: 5000
    spawn_chance: 0.0
    min_floor: 10
    max_floor: 10
    ammo: false
    consumable: false
monsters: []
spells: []
lore: []
"#;
        let yaml_b = r#"
player:
  max_hp: 99
  defense: 0
  power: 0
  viewshed: 1
  hearing_range: 1
  light_range: 1
  aegis: 1
  str: 1
  dex: 1
  con: 1
  int: 1
  wis: 1
  cha: 1
monsters: []
items: []
spells: []
lore: []
"#;
        fs::write(dir.path().join("a.yaml"), yaml_a).unwrap();
        fs::write(dir.path().join("b.yaml"), yaml_b).unwrap();
        let result = Content::load_from_dir(dir.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("duplicate [player]"));
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
    fn test_load_from_dir_merges_files() {
        use std::fs;
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let amulet_yaml = r#"
items:
  - name: "Amulet of the Ancients"
    glyph: '"'
    color: [255, 215, 0]
    price: 5000
    spawn_chance: 0.0
    min_floor: 10
    max_floor: 10
    ammo: false
    consumable: false
monsters: []
spells: []
"#;
        let extra_yaml = r#"
monsters:
  - name: "Test Rat"
    glyph: r
    color: [100, 100, 100]
    hp: 3
    defense: 0
    power: 1
    viewshed: 5
    spawn_chance: 1.0
    min_floor: 1
    max_floor: 99
    personality: Brave
    faction: Animals
    xp_reward: 1
items: []
spells: []
"#;
        fs::write(dir.path().join("base.yaml"), amulet_yaml).unwrap();
        fs::write(dir.path().join("extra.yaml"), extra_yaml).unwrap();
        let content = Content::load_from_dir(dir.path()).expect("load_from_dir failed");
        assert_eq!(content.items.len(), 1);
        assert_eq!(content.monsters.len(), 1);
        assert_eq!(content.items[0].name, "Amulet of the Ancients");
        assert_eq!(content.monsters[0].name, "Test Rat");
    }

    #[test]
    fn test_load_from_dir_duplicate_name_is_error() {
        use std::fs;
        use tempfile::TempDir;
        let dir = TempDir::new().unwrap();
        let yaml_a = r#"
items:
  - name: "Amulet of the Ancients"
    glyph: '"'
    color: [255, 215, 0]
    price: 5000
    spawn_chance: 0.0
    min_floor: 10
    max_floor: 10
    ammo: false
    consumable: false
  - name: "Duplicate Item"
    glyph: '!'
    color: [255, 0, 0]
    price: 1
    spawn_chance: 0.1
    min_floor: 1
    max_floor: 5
    ammo: false
    consumable: true
monsters: []
spells: []
"#;
        let yaml_b = r#"
items:
  - name: "Duplicate Item"
    glyph: '!'
    color: [0, 255, 0]
    price: 2
    spawn_chance: 0.2
    min_floor: 1
    max_floor: 5
    ammo: false
    consumable: true
monsters: []
spells: []
"#;
        fs::write(dir.path().join("a.yaml"), yaml_a).unwrap();
        fs::write(dir.path().join("b.yaml"), yaml_b).unwrap();
        let result = Content::load_from_dir(dir.path());
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("duplicate") || msg.contains("Duplicate Item"),
            "error: {msg}"
        );
    }

    #[test]
    fn test_live_content_spells_bake() {
        let content = Content::load_from_dir(std::path::Path::new("content/"))
            .expect("content/ must load cleanly");
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

    #[test]
    fn test_live_content_v09_weapons() {
        use crate::components::WeaponPowerSource;
        let content = Content::load_from_dir(std::path::Path::new("content/"))
            .expect("content/ must load cleanly");
        let find = |name: &str| {
            content
                .items
                .iter()
                .find(|i| i.name == name)
                .unwrap_or_else(|| panic!("missing item: {name}"))
        };

        // Scattergun — Scatter, Heat.
        let rw = find("Scattergun")
            .ranged_weapon
            .as_ref()
            .expect("Scattergun ranged_weapon");
        assert!(rw.scatter, "Scattergun must have scatter flag");
        assert_eq!(rw.power_source().unwrap(), WeaponPowerSource::Heat);

        // Carbine — Burst 3, Heat.
        let rw = find("Carbine")
            .ranged_weapon
            .as_ref()
            .expect("Carbine ranged_weapon");
        assert_eq!(rw.burst_count, Some(3));
        assert_eq!(rw.power_source().unwrap(), WeaponPowerSource::Heat);

        // Heavy Rifle — Shredding, HeavyAmmo.
        let rw = find("Heavy Rifle")
            .ranged_weapon
            .as_ref()
            .expect("Heavy Rifle ranged_weapon");
        assert!(rw.shredding);
        assert_eq!(rw.power_source().unwrap(), WeaponPowerSource::HeavyAmmo);

        // Tachyon Lance — Tachyonic + Efficient Cooldown, Heat.
        let rw = find("Tachyon Lance")
            .ranged_weapon
            .as_ref()
            .expect("Tachyon Lance ranged_weapon");
        assert!(rw.tachyonic);
        assert!(rw.efficient_cooldown);
        assert_eq!(rw.power_source().unwrap(), WeaponPowerSource::Heat);

        // Phoenix Repeater — Elemental: Fire, Heat.
        let rw = find("Phoenix Repeater")
            .ranged_weapon
            .as_ref()
            .expect("Phoenix Repeater ranged_weapon");
        assert_eq!(rw.element_type().unwrap(), Some(DamageType::Fire));

        // Monk's Crook — Medium melee profile, Heat.
        let crook = find("Monk's Crook");
        let w = crook.weapon.as_ref().expect("Monk's Crook weapon profile");
        assert_eq!(w.weight, crate::components::WeaponWeight::Medium);
        assert!(crook.ranged_weapon.is_some());

        // Heavy Ammo — heavy_ammo marker and stack count.
        let ammo = find("Heavy Ammo");
        assert!(ammo.heavy_ammo, "Heavy Ammo must carry heavy_ammo marker");
        assert!(ammo.stack.is_some(), "Heavy Ammo must define a stack count");
        assert!(ammo.consumable, "Heavy Ammo must be consumable");
    }
}
