use crate::app::{App, VisualEffect};
use crate::components::*;
use rand::Rng;
use ratatui::prelude::Color;

pub struct AttackResult {
    pub hit: bool,
    pub critical: bool,
    pub damage: i32,
    pub attacker_name: String,
    pub target_name: String,
    pub attack_roll: i32,
    pub attack_mod: i32,
    pub dodge_dc: i32,
    pub damage_dice_roll: i32,
    pub damage_mod: i32,
    pub target_av: i32,
}

impl App {
    pub fn resolve_attack(
        &mut self,
        attacker: hecs::Entity,
        target: hecs::Entity,
        specific_weapon: Option<hecs::Entity>,
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
        let attr_mod;
        let mut damage_dice = (1, 4); // Default 1d4
        let mut power_bonus = 0;

        let weapon = if let Some(weapon_id) = specific_weapon {
            self.world.get::<&Weapon>(weapon_id).ok().map(|w| *w)
        } else {
            self.get_equipped_weapon(attacker)
        };

        if let Some(w) = weapon {
            attr_mod = match w.weight {
                WeaponWeight::Light => self.get_attribute_modifier(attacker, |a| a.dexterity),
                _ => self.get_attribute_modifier(attacker, |a| a.strength),
            };
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
        let roll = self.rng.gen_range(1..=20);
        let mut hit = false;
        let mut critical = false;

        // Target Dodge DC (10 + DEX mod)
        let target_dex_mod = self.get_attribute_modifier(target, |a| a.dexterity);
        let dodge_dc = 10 + target_dex_mod;

        if roll == 20 {
            hit = true;
            critical = true;
        } else if roll == 1 {
            hit = false;
        } else if roll + attr_mod >= dodge_dc {
            hit = true;
        }


        // 3. Damage Calculation
        let mut damage = 0;
        let mut weapon_roll = 0;
        let mut target_av = 0;
        if hit {
            for _ in 0..damage_dice.0 {
                weapon_roll += self.rng.gen_range(1..=damage_dice.1);
            }

            target_av = self.get_target_av(target);
            damage = (weapon_roll + attr_mod + power_bonus - target_av).max(1);

            if critical {
                damage *= 2;
            }
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
        }
    }

    pub fn get_equipped_weapon(&self, entity: hecs::Entity) -> Option<Weapon> {
        for (id, (eq, weapon)) in self.world.query::<(&Equipped, &Weapon)>().iter() {
            if let Ok(backpack) = self.world.get::<&InBackpack>(id) {
                if backpack.owner == entity
                    && (eq.slot == EquipmentSlot::Melee || eq.slot == EquipmentSlot::MainHand)
                {
                    return Some(*weapon);
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

    pub fn get_target_av(&self, entity: hecs::Entity) -> i32 {
        if self.world.get::<&Player>(entity).is_ok() {
            let (_, def) = self.get_player_stats();
            def
        } else {
            let mut def = self
                .world
                .get::<&CombatStats>(entity)
                .map(|s| s.defense)
                .unwrap_or(0);
            if let Ok(attr) = self.world.get::<&Attributes>(entity) {
                def += Attributes::get_modifier(attr.dexterity);
            }
            // Monster equipment
            for (id, (_eq, backpack)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
                if backpack.owner == entity {
                    if let Ok(armor) = self.world.get::<&Armor>(id) {
                        def += armor.defense_bonus;
                    }
                }
            }
            def
        }
    }

    pub fn apply_attack_result(&mut self, target: hecs::Entity, res: &AttackResult, x: u16, y: u16) {
        if !res.hit {
            if res.attack_roll == 1 {
                self.log.push(format!("{} critically misses {}! (Roll: 1)", res.attacker_name, res.target_name));
            } else {
                self.log.push(format!("{} misses {} (Roll: {}+{} vs DC:{})", 
                    res.attacker_name, res.target_name, res.attack_roll, res.attack_mod, res.dodge_dc));
            }
            return;
        }

        let crit_str = if res.critical { "CRITICAL HIT! " } else { "" };
        self.log.push(format!("{}{} hits {} for {} damage! (Roll:{}+{} vs DC:{}, Dmg:{}+{} DR:{})",
            crit_str, res.attacker_name, res.target_name, res.damage,
            res.attack_roll, res.attack_mod, res.dodge_dc,
            res.damage_dice_roll, res.damage_mod, res.target_av
        ));

        self.effects.push(VisualEffect::Flash {
            x,
            y,
            glyph: if res.critical { '!' } else { '*' },
            fg: if res.critical { Color::Yellow } else { Color::Red },
            bg: None,
            duration: if res.critical { 10 } else { 5 },
        });

        if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target) {
            stats.hp -= res.damage;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    fn setup_test_app() -> App {
        let mut app = App::new_test(42);
        app.world = World::new();
        app
    }

    #[test]
    fn test_resolve_attack_hit() {
        let mut app = setup_test_app();
        let attacker = app.world.spawn((
            Name("Attacker".to_string()),
            Attributes { strength: 20, dexterity: 10, constitution: 10, intelligence: 10, wisdom: 10, charisma: 10 },
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 0 },
        ));
        let target = app.world.spawn((
            Name("Target".to_string()),
            Attributes { strength: 10, dexterity: 10, constitution: 10, intelligence: 10, wisdom: 10, charisma: 10 },
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 0 },
        ));

        let res = app.resolve_attack(attacker, target, None);
        assert_eq!(res.attacker_name, "Attacker");
        assert_eq!(res.target_name, "Target");
        assert!(res.attack_roll >= 1 && res.attack_roll <= 20);
        assert_eq!(res.attack_mod, 5); // STR 20
        assert_eq!(res.dodge_dc, 10); // 10 + DEX 10 (0)
    }

    #[test]
    fn test_get_equipped_weapon() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player,));
        let _sword = app.world.spawn((
            Weapon {
                power_bonus: 5,
                weight: WeaponWeight::Medium,
                damage_n_dice: 1,
                damage_die_type: 8,
            },
            Equippable { slot: EquipmentSlot::Melee },
            Equipped { slot: EquipmentSlot::Melee },
            InBackpack { owner: player },
        ));

        let weapon = app.get_equipped_weapon(player);
        assert!(weapon.is_some());
        assert_eq!(weapon.unwrap().power_bonus, 5);
    }
}
