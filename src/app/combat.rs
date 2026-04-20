use crate::app::{App, DamageRoute, VisualEffect};
use crate::components::*;
use bracket_pathfinding::prelude::*;
use rand::Rng;
use ratatui::prelude::Color;

pub struct AttackResult {
    pub hit: bool,
    pub critical: bool,
    /// Raw pre-mitigation damage. Mitigation (Aegis, AV) is applied in `apply_damage`.
    pub damage: i32,
    pub attacker_name: String,
    pub target_name: String,
    pub attack_roll: i32,
    pub attack_mod: i32,
    pub dodge_dc: i32,
    pub damage_dice_roll: i32,
    pub damage_mod: i32,
    pub target_av: i32,
    pub route: DamageRoute,
    pub poison: Option<Poison>,
    pub confusion: Option<Confusion>,
}

impl App {
    pub fn get_max_dex_bonus(&self, entity: hecs::Entity) -> Option<i32> {
        let mut min_max = None;
        for (id, (_eq, armor)) in self.world.query::<(&Equipped, &Armor)>().iter() {
            if let Ok(backpack) = self.world.get::<&InBackpack>(id) {
                if backpack.owner == entity {
                    if let Some(limit) = armor.max_dex_bonus {
                        min_max = Some(min_max.map(|m| i32::min(m, limit)).unwrap_or(limit));
                    }
                }
            }
        }
        min_max
    }

    pub fn resolve_attack(
        &mut self,
        attacker: hecs::Entity,
        target: hecs::Entity,
        specific_weapon: Option<hecs::Entity>,
        disadvantage_count: u32,
        is_ranged: bool,
    ) -> AttackResult {
        let attacker_name = self
            .world
            .get::<&Name>(attacker)
            .map(|n| n.0.clone())
            .unwrap_or("Someone".to_string());
        let target_name = self
            .world
            .get::<&Name>(target)
            .map(|n| n.0.clone())
            .unwrap_or("Something".to_string());

        // 1. Determine Attacker Modifier and Damage Dice
        let mut attr_mod;
        let mut damage_dice = (1, 4); // Default 1d4
        let mut power_bonus = 0;

        let weapon_entity = specific_weapon.or_else(|| self.get_equipped_weapon_entity(attacker));
        let ranged_weapon =
            weapon_entity.and_then(|id| self.world.get::<&RangedWeapon>(id).ok().map(|rw| *rw));
        let weapon = weapon_entity.and_then(|id| self.world.get::<&Weapon>(id).ok().map(|w| *w));

        if let Some(rw) = ranged_weapon {
            if is_ranged {
                // Ranged Attack
                attr_mod = self.get_dex_modifier(attacker);
                damage_dice = (1, 6); // Default ranged damage
                if let Some(w) = weapon {
                    damage_dice = (w.damage_n_dice, w.damage_die_type);
                }
                power_bonus = rw.damage_bonus;
            } else {
                // Improvised Melee with Ranged Weapon
                attr_mod = self.get_attribute_modifier(attacker, |a| a.strength);
                if let Some(w) = weapon {
                    damage_dice = (w.damage_n_dice, w.damage_die_type);
                    power_bonus = w.power_bonus;
                }
            }
        } else if let Some(w) = weapon {
            // Melee Attack with non-ranged weapon
            attr_mod = match w.weight {
                WeaponWeight::Light => self.get_dex_modifier(attacker),
                _ => self.get_attribute_modifier(attacker, |a| a.strength),
            };
            if w.two_handed {
                attr_mod = (attr_mod as f32 * 1.5) as i32;
            }
            damage_dice = (w.damage_n_dice, w.damage_die_type);
            power_bonus = w.power_bonus;
        } else {
            // Unarmed
            attr_mod = self.get_attribute_modifier(attacker, |a| a.strength);
            if let Ok(stats) = self.world.get::<&CombatStats>(attacker) {
                power_bonus = stats.power;
            }
        }

        // 2. Attack Roll (1d20 + mod)
        let mut rolls = Vec::new();
        for _ in 0..=disadvantage_count {
            rolls.push(self.rng.random_range(1..=20));
        }
        let roll = *rolls.iter().min().expect("rolls is never empty");

        let mut hit = false;
        let mut critical = false;

        // Target Dodge DC (10 + capped DEX mod, +2 if ranged and partial cover intervenes)
        let target_dex_mod = self.get_dex_modifier(target);
        let mut dodge_dc = 10 + target_dex_mod;
        if is_ranged && self.has_partial_cover_between(attacker, target) {
            dodge_dc += 2;
        }

        if roll == 20 {
            hit = true;
            critical = true;
        } else if roll == 1 {
            hit = false;
        } else if roll + attr_mod >= dodge_dc {
            hit = true;
        }

        // 3. Damage Calculation (raw — mitigation happens in apply_damage)
        let mut damage = 0;
        let mut weapon_roll = 0;
        let mut target_av = 0;
        let mut poison = None;
        let mut confusion = None;

        if hit {
            let mut n_dice = damage_dice.0;
            if critical {
                n_dice *= 2;
            }
            for _ in 0..n_dice {
                weapon_roll += self.rng.random_range(1..=damage_dice.1);
            }

            target_av = self.get_target_av(target);
            damage = (weapon_roll + attr_mod + power_bonus).max(0);

            let effect_source = weapon_entity.unwrap_or(attacker);
            poison = self.world.get::<&Poison>(effect_source).ok().map(|p| *p);
            confusion = self.world.get::<&Confusion>(effect_source).ok().map(|c| *c);
        }

        AttackResult {
            hit,
            critical,
            damage,
            attacker_name,
            target_name,
            attack_roll: roll,
            attack_mod: attr_mod,
            dodge_dc,
            damage_dice_roll: weapon_roll,
            damage_mod: attr_mod + power_bonus,
            target_av,
            route: if is_ranged {
                DamageRoute::Projectile
            } else {
                DamageRoute::Contact
            },
            poison,
            confusion,
        }
    }

    pub fn make_saving_throw(
        &mut self,
        entity: hecs::Entity,
        dc: i32,
        kind: SavingThrowKind,
    ) -> bool {
        let modifier = match kind {
            SavingThrowKind::Strength => self.get_attribute_modifier(entity, |a| a.strength),
            SavingThrowKind::Dexterity => self.get_dex_modifier(entity),
            SavingThrowKind::Constitution => {
                self.get_attribute_modifier(entity, |a| a.constitution)
            }
            SavingThrowKind::Intelligence => {
                self.get_attribute_modifier(entity, |a| a.intelligence)
            }
            SavingThrowKind::Wisdom => self.get_attribute_modifier(entity, |a| a.wisdom),
            SavingThrowKind::Charisma => self.get_attribute_modifier(entity, |a| a.charisma),
        };

        let roll = self.rng.random_range(1..=20);
        let success = roll + modifier >= dc;

        let name = self
            .world
            .get::<&Name>(entity)
            .map(|n| n.0.clone())
            .unwrap_or("Someone".to_string());
        let kind_str = match kind {
            SavingThrowKind::Strength => "STR",
            SavingThrowKind::Dexterity => "DEX",
            SavingThrowKind::Constitution => "CON",
            SavingThrowKind::Intelligence => "INT",
            SavingThrowKind::Wisdom => "WIS",
            SavingThrowKind::Charisma => "CHA",
        };

        if success {
            self.log.push(format!(
                "{} makes a {} save! (Roll: {}+{} vs DC:{})",
                name, kind_str, roll, modifier, dc
            ));
        } else {
            self.log.push(format!(
                "{} fails a {} save! (Roll: {}+{} vs DC:{})",
                name, kind_str, roll, modifier, dc
            ));
        }

        success
    }

    pub fn get_equipped_weapon_entity(&self, entity: hecs::Entity) -> Option<hecs::Entity> {
        for (id, (eq, _weapon)) in self.world.query::<(&Equipped, &Weapon)>().iter() {
            if let Ok(backpack) = self.world.get::<&InBackpack>(id) {
                if backpack.owner == entity && eq.slot == EquipmentSlot::MainHand {
                    return Some(id);
                }
            }
        }
        None
    }

    pub fn get_off_hand_weapon(&self, entity: hecs::Entity) -> Option<hecs::Entity> {
        for (id, (eq, _weapon)) in self.world.query::<(&Equipped, &Weapon)>().iter() {
            if let Ok(backpack) = self.world.get::<&InBackpack>(id) {
                if backpack.owner == entity && eq.slot == EquipmentSlot::OffHand {
                    return Some(id);
                }
            }
        }
        None
    }

    pub fn get_dex_modifier(&self, entity: hecs::Entity) -> i32 {
        let mut m = self.get_attribute_modifier(entity, |a| a.dexterity);
        if let Some(limit) = self.get_max_dex_bonus(entity) {
            m = m.min(limit);
        }
        m
    }

    pub fn get_attribute_modifier<F>(&self, entity: hecs::Entity, f: F) -> i32
    where
        F: Fn(&Attributes) -> i32,
    {
        if let Ok(attr) = self.world.get::<&Attributes>(entity) {
            Attributes::get_modifier(f(&attr))
        } else {
            0
        }
    }

    /// Returns true if partial cover intervenes between attacker and target:
    /// either on the target's own tile or on the tile immediately before it
    /// along the Bresenham line from attacker to target.
    pub fn has_partial_cover_between(&self, attacker: hecs::Entity, target: hecs::Entity) -> bool {
        let Ok(a_pos) = self.world.get::<&Position>(attacker) else {
            return false;
        };
        let Ok(t_pos) = self.world.get::<&Position>(target) else {
            return false;
        };
        let (ax, ay) = (a_pos.x, a_pos.y);
        let (tx, ty) = (t_pos.x, t_pos.y);
        drop(a_pos);
        drop(t_pos);

        if ax == tx && ay == ty {
            return self.tile_has_partial_cover(tx, ty);
        }

        let points = line2d(LineAlg::Bresenham, Point::new(ax, ay), Point::new(tx, ty));
        if points.len() < 2 {
            return self.tile_has_partial_cover(tx, ty);
        }

        // Target tile always qualifies; penultimate tile on the line also qualifies.
        if self.tile_has_partial_cover(tx, ty) {
            return true;
        }
        let penult = points[points.len() - 2];
        self.tile_has_partial_cover(penult.x as u16, penult.y as u16)
    }

    fn tile_has_partial_cover(&self, x: u16, y: u16) -> bool {
        for (_id, (pos, _cover)) in self.world.query::<(&Position, &PartialCover)>().iter() {
            if pos.x == x && pos.y == y {
                return true;
            }
        }
        false
    }

    pub fn get_target_av(&self, entity: hecs::Entity) -> i32 {
        let mut def = self
            .world
            .get::<&CombatStats>(entity)
            .map(|s| s.defense)
            .unwrap_or(0);

        // Sum all equipped armor/shields
        for (id, (_eq, armor)) in self.world.query::<(&Equipped, &Armor)>().iter() {
            if let Ok(backpack) = self.world.get::<&InBackpack>(id) {
                if backpack.owner == entity {
                    def += armor.defense_bonus;
                }
            }
        }
        def
    }

    pub fn apply_attack_result(
        &mut self,
        target: hecs::Entity,
        res: &AttackResult,
        x: u16,
        y: u16,
    ) {
        if !res.hit {
            if res.attack_roll == 1 {
                self.log.push(format!(
                    "{} critically misses {}! (Roll: 1)",
                    res.attacker_name, res.target_name
                ));
            } else {
                self.log.push(format!(
                    "{} misses {} (Roll: {}+{} vs DC:{})",
                    res.attacker_name,
                    res.target_name,
                    res.attack_roll,
                    res.attack_mod,
                    res.dodge_dc
                ));
            }
            return;
        }

        let outcome = self.apply_damage(target, res.damage, res.route);

        let crit_str = if res.critical { "CRITICAL HIT! " } else { "" };
        let applied = outcome.hp_damage + outcome.aegis_damage;
        let aegis_tag = if outcome.aegis_damage > 0 {
            format!(" [{} to Aegis]", outcome.aegis_damage)
        } else {
            String::new()
        };
        self.log.push(format!(
            "{}{} hits {} for {} damage!{} (Roll:{}+{} vs DC:{}, Dmg:{}+{} DR:{})",
            crit_str,
            res.attacker_name,
            res.target_name,
            applied,
            aegis_tag,
            res.attack_roll,
            res.attack_mod,
            res.dodge_dc,
            res.damage_dice_roll,
            res.damage_mod,
            res.target_av
        ));

        self.effects.push(VisualEffect::Flash {
            x,
            y,
            glyph: if res.critical { '!' } else { '*' },
            fg: if res.critical {
                Color::Yellow
            } else {
                Color::Red
            },
            bg: None,
            duration: if res.critical { 10 } else { 5 },
        });

        let is_player_god = self.world.get::<&Player>(target).is_ok() && self.god_mode;
        if is_player_god && res.damage > 0 {
            self.log
                .push("Debug: Player is in God Mode! No damage taken.".to_string());
        }

        // Apply status effects
        if let Some(poison) = res.poison {
            if self.world.get::<&Poison>(target).is_err() {
                let is_player = self.world.get::<&Player>(target).is_ok();
                if is_player && self.god_mode {
                    // Skip
                } else if !self.make_saving_throw(target, 12, SavingThrowKind::Constitution) {
                    self.log.push(format!("{} is poisoned!", res.target_name));
                    let _ = self.world.insert_one(target, poison);
                } else {
                    self.log
                        .push(format!("{} resists the poison!", res.target_name));
                }
            }
        }

        if let Some(confusion) = res.confusion {
            if self.world.get::<&Confusion>(target).is_err() {
                if !self.make_saving_throw(target, 12, SavingThrowKind::Intelligence) {
                    self.log.push(format!("{} is confused!", res.target_name));
                    let _ = self.world.insert_one(target, confusion);
                } else {
                    self.log
                        .push(format!("{} resists the confusion!", res.target_name));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    fn setup_test_app() -> App {
        let mut app = App::new_random().expect("content.json must be present for tests");
        app.world = World::new();
        app
    }

    #[test]
    fn test_resolve_attack_hit() {
        let mut app = setup_test_app();
        let attacker = app.world.spawn((
            Name("Attacker".to_string()),
            Attributes {
                strength: 20,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 0,
            },
        ));
        let target = app.world.spawn((
            Name("Target".to_string()),
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 0,
            },
        ));

        let res = app.resolve_attack(attacker, target, None, 0, false);
        assert_eq!(res.attacker_name, "Attacker");
        assert_eq!(res.target_name, "Target");
        assert!(res.attack_roll >= 1 && res.attack_roll <= 20);
        assert_eq!(res.attack_mod, 5); // STR 20
        assert_eq!(res.dodge_dc, 10); // 10 + DEX 10 (0)
    }

    #[test]
    fn test_zero_defense_damage_at_least_one() {
        let mut app = setup_test_app();
        let attacker = app.world.spawn((
            Name("Attacker".to_string()),
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 0,
            },
        ));
        let target = app.world.spawn((
            Name("Target".to_string()),
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 100,
                max_hp: 100,
                defense: 0,
                power: 0,
            },
        ));

        // Run many attacks; whenever one hits, damage must be >= 1
        for _ in 0..50 {
            let res = app.resolve_attack(attacker, target, None, 0, false);
            if res.hit {
                assert!(res.damage >= 1, "damage must be at least 1 on a hit");
            }
        }
    }

    fn spawn_cover(app: &mut App, x: u16, y: u16) {
        app.world
            .spawn((Position { x, y }, PartialCover, Name("Debris".to_string())));
    }

    fn spawn_plain_fighter(app: &mut App, name: &str, x: u16, y: u16) -> hecs::Entity {
        app.world.spawn((
            Name(name.to_string()),
            Position { x, y },
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 0,
            },
        ))
    }

    #[test]
    fn test_partial_cover_raises_ranged_dodge_dc() {
        let mut app = setup_test_app();
        let attacker = spawn_plain_fighter(&mut app, "Shooter", 1, 5);
        let target = spawn_plain_fighter(&mut app, "Target", 5, 5);
        // Cover on the penultimate tile between attacker and target
        spawn_cover(&mut app, 4, 5);

        let res = app.resolve_attack(attacker, target, None, 0, true);
        assert_eq!(res.dodge_dc, 12, "partial cover should add +2 to dodge DC");
    }

    #[test]
    fn test_partial_cover_on_target_tile_raises_dodge_dc() {
        let mut app = setup_test_app();
        let attacker = spawn_plain_fighter(&mut app, "Shooter", 1, 5);
        let target = spawn_plain_fighter(&mut app, "Target", 5, 5);
        // Cover on target's own tile also qualifies
        spawn_cover(&mut app, 5, 5);

        let res = app.resolve_attack(attacker, target, None, 0, true);
        assert_eq!(res.dodge_dc, 12);
    }

    #[test]
    fn test_partial_cover_on_far_side_gives_no_bonus() {
        let mut app = setup_test_app();
        let attacker = spawn_plain_fighter(&mut app, "Shooter", 1, 5);
        let target = spawn_plain_fighter(&mut app, "Target", 5, 5);
        // Cover on the far side of target — not between attacker and target
        spawn_cover(&mut app, 6, 5);

        let res = app.resolve_attack(attacker, target, None, 0, true);
        assert_eq!(res.dodge_dc, 10, "cover beyond target should not add DC");
    }

    #[test]
    fn test_partial_cover_ignored_for_melee() {
        let mut app = setup_test_app();
        let attacker = spawn_plain_fighter(&mut app, "Bumper", 4, 5);
        let target = spawn_plain_fighter(&mut app, "Target", 5, 5);
        spawn_cover(&mut app, 4, 5);

        let res = app.resolve_attack(attacker, target, None, 0, false);
        assert_eq!(res.dodge_dc, 10, "melee attacks ignore partial cover");
    }

    #[test]
    fn test_overkill_damage_does_not_panic() {
        let mut app = setup_test_app();
        let attacker = app.world.spawn((
            Name("Giant".to_string()),
            Attributes {
                strength: 30,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 100,
                max_hp: 100,
                defense: 0,
                power: 100,
            },
        ));
        let target = app.world.spawn((
            Name("Rat".to_string()),
            Attributes {
                strength: 1,
                dexterity: 1,
                constitution: 1,
                intelligence: 1,
                wisdom: 1,
                charisma: 1,
            },
            CombatStats {
                hp: 1,
                max_hp: 1,
                defense: 0,
                power: 0,
            },
        ));

        let res = app.resolve_attack(attacker, target, None, 0, false);
        // Overkill should not panic; if it hit, damage > target hp is fine
        if res.hit {
            assert!(res.damage >= 1);
        }
        // Apply it without panicking
        app.apply_attack_result(target, &res, 0, 0);
    }
}
