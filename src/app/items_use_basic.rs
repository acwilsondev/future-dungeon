use crate::app::{App, RunState};
use crate::components::*;
use rand::Rng;

impl App {
    pub fn use_item(&mut self, item_id: hecs::Entity) {
        let Some(player_id) = self.get_player_id() else {
            log::error!("Player not found during use_item");
            return;
        };
        let item_name = self.get_item_name(item_id);
        let real_name = self
            .world
            .get::<&Name>(item_id)
            .map(|n| n.0.clone())
            .unwrap_or("Item".to_string());

        if real_name == "Identification Scroll" {
            self.state = RunState::ShowIdentify;
            self.targeting_item = Some(item_id);
            self.inventory_cursor = 0;
            self.log.push("Select an item to identify...".to_string());
            return;
        }

        let tome_info = self
            .world
            .get::<&Tome>(item_id)
            .ok()
            .map(|t| (t.spell_name.clone(), t.level));
        if let Some((_spell_name, level)) = tome_info {
            if !self.identified_items.contains(&real_name) {
                let cha_mod = self
                    .world
                    .get::<&Attributes>(player_id)
                    .map(|a| Attributes::get_modifier(a.charisma))
                    .unwrap_or(0);
                let dc = 10 + level as i32;
                let roll = self.rng.random_range(1..=20);
                if roll + cha_mod >= dc {
                    self.identified_items.insert(real_name.clone());
                    self.log
                        .push(format!("You decipher the Tome: {}.", real_name));
                    self.begin_study_tome(item_id);
                } else {
                    self.log
                        .push("The Tome's script eludes you for now.".to_string());
                }
            } else {
                self.begin_study_tome(item_id);
            }
            return;
        }

        let player_pos = self
            .world
            .get::<&Position>(player_id)
            .ok()
            .map(|p| *p)
            .unwrap_or(Position { x: 0, y: 0 });

        let mut handled = false;

        let potion_heal = self
            .world
            .get::<&Potion>(item_id)
            .ok()
            .map(|p| p.heal_amount);
        if let Some(heal_amount) = potion_heal {
            if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                stats.hp = (stats.hp + heal_amount).min(stats.max_hp);
            }
            self.log.push(format!(
                "You drink the {}, healing for {} HP.",
                item_name, heal_amount
            ));
            self.generate_noise(player_pos.x, player_pos.y, 1.0);
            handled = true;
        }

        let poison_effect = self.world.get::<&Poison>(item_id).ok().map(|p| *p);
        if let Some(poison) = poison_effect {
            let _ = self.world.insert_one(player_id, poison);
            self.log
                .push(format!("You are poisoned by the {}!", item_name));
            handled = true;
        }

        let strength_effect = self.world.get::<&Strength>(item_id).ok().map(|s| *s);
        if let Some(strength) = strength_effect {
            let _ = self.world.insert_one(player_id, strength);
            if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                stats.power += strength.amount;
            }
            self.log
                .push(format!("The {} makes you feel much stronger!", item_name));
            handled = true;
        }

        let speed_effect = self.world.get::<&Speed>(item_id).ok().map(|s| *s);
        if let Some(speed) = speed_effect {
            let _ = self.world.insert_one(player_id, speed);
            self.log
                .push(format!("The {} makes you feel incredibly fast!", item_name));
            handled = true;
        }

        if self.world.get::<&Ranged>(item_id).is_ok()
            || (self.world.get::<&RangedWeapon>(item_id).is_ok()
                && self.world.get::<&Equipped>(item_id).is_ok())
        {
            if let Ok(rw) = self.world.get::<&RangedWeapon>(item_id) {
                // Check for correct ammo type based on power source
                let has_ammo = match rw.power_source {
                    WeaponPowerSource::Ammo => self
                        .world
                        .query::<(&Ammunition, &InBackpack)>()
                        .iter()
                        .any(|(_, (_, backpack))| backpack.owner == player_id),
                    WeaponPowerSource::HeavyAmmo => self
                        .world
                        .query::<(&HeavyAmmo, &InBackpack)>()
                        .iter()
                        .any(|(_, (_, backpack))| backpack.owner == player_id),
                    WeaponPowerSource::Heat => true, // Heat based weapons don't need inventory ammo
                };

                if !has_ammo {
                    let msg = match rw.power_source {
                        WeaponPowerSource::HeavyAmmo => format!("You have no heavy ammunition for your {}!", item_name),
                        _ => format!("You have no ammunition for your {}!", item_name),
                    };
                    self.log.push(msg);
                    return;
                }
            }
            if let Ok(player_pos) = self.world.get::<&Position>(player_id) {
                self.targeting_cursor = (player_pos.x, player_pos.y);
                self.targeting_item = Some(item_id);
                self.state = RunState::ShowTargeting;
                self.log.push(format!("Select target for {}...", item_name));
            }
            return;
        }

        if self.world.get::<&Equippable>(item_id).is_ok() {
            if self.world.get::<&Equipped>(item_id).is_ok() {
                self.unequip_item(item_id);
            } else {
                self.equip_item(item_id);
            }
            handled = true;
        }

        if handled {
            self.identify_item(item_id);
            if self.world.get::<&Consumable>(item_id).is_ok() {
                if let Err(e) = self.world.despawn(item_id) {
                    log::error!("Failed to despawn consumable item after use: {}", e);
                }
                self.state = RunState::MonsterTurn;
            }
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
    fn test_use_health_potion() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            CombatStats {
                hp: 5,
                max_hp: 20,
                defense: 0,
                power: 5,
            },
            Position { x: 0, y: 0 },
        ));
        let potion = app.world.spawn((
            Item,
            Name("Health Potion".to_string()),
            Potion { heal_amount: 10 },
            Consumable,
            InBackpack { owner: player },
        ));

        app.use_item(potion);

        let stats = app.world.get::<&CombatStats>(player).unwrap();
        assert_eq!(stats.hp, 15);
        assert!(app.world.get::<&Item>(potion).is_err()); // Consumed
        assert_eq!(app.state, RunState::MonsterTurn);
    }

    #[test]
    fn test_use_poison_item() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            CombatStats {
                hp: 20,
                max_hp: 20,
                defense: 0,
                power: 5,
            },
            Position { x: 0, y: 0 },
        ));
        let bad_item = app.world.spawn((
            Item,
            Name("Bad Mushroom".to_string()),
            Poison {
                damage: 2,
                turns: 5,
            },
            Consumable,
            InBackpack { owner: player },
        ));

        app.use_item(bad_item);

        assert!(app.world.get::<&Poison>(player).is_ok());
        assert!(app.world.get::<&Item>(bad_item).is_err());
    }

    #[test]
    fn test_use_strength_potion() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            CombatStats {
                hp: 20,
                max_hp: 20,
                defense: 0,
                power: 5,
            },
            Position { x: 0, y: 0 },
        ));
        let strength_potion = app.world.spawn((
            Item,
            Name("Potion of Strength".to_string()),
            Strength {
                amount: 2,
                turns: 10,
            },
            Consumable,
            InBackpack { owner: player },
        ));

        app.use_item(strength_potion);

        let stats = app.world.get::<&CombatStats>(player).unwrap();
        assert_eq!(stats.power, 7);
        assert!(app.world.get::<&Strength>(player).is_ok());
    }

    #[test]
    fn test_use_identify_scroll() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let scroll = app.world.spawn((
            Item,
            Name("Identification Scroll".to_string()),
            Consumable,
            InBackpack { owner: player },
        ));

        app.use_item(scroll);

        assert_eq!(app.state, RunState::ShowIdentify);
        assert_eq!(app.targeting_item, Some(scroll));
    }

    #[test]
    fn test_use_speed_potion() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let speed_potion = app.world.spawn((
            Item,
            Name("Potion of Speed".to_string()),
            Speed { turns: 10 },
            Consumable,
            InBackpack { owner: player },
        ));

        app.use_item(speed_potion);
        assert!(app.world.get::<&Speed>(player).is_ok());
    }

    #[test]
    fn test_use_ranged_no_ammo() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let bow = app.world.spawn((
            Item,
            Name("Bow".to_string()),
            RangedWeapon {
                range: 8,
                range_increment: 12,
                damage_bonus: 2,
                ..Default::default()
            },
            Equippable {
                slot: EquipmentSlot::AnyHand,
            },
            InBackpack { owner: player },
        ));

        app.state = RunState::AwaitingInput;
        app.use_item(bow);
        // Should equip now
        assert!(app.world.get::<&Equipped>(bow).is_ok());
        assert!(app.log.last().unwrap().contains("You equip"));
        assert_eq!(app.state, RunState::AwaitingInput);

        // Try using again, now it should check for ammo
        app.use_item(bow);
        assert!(app.log.last().unwrap().contains("no ammunition for your Bow"));
    }

    #[test]
    fn test_use_tome_unidentified_failed_check() {
        let mut app = setup_test_app();
        app.state = RunState::AwaitingInput;
        let player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 1,
            },
        ));
        let tome = app.world.spawn((
            Item,
            Name("Tome of Nothing".to_string()),
            ObfuscatedName("Strange Orange Tome".to_string()),
            Tome {
                spell_name: "Nothing".to_string(),
                color: ManaColor::Orange,
                level: 10,
            },
            InBackpack { owner: player },
        ));

        app.use_item(tome);
        assert!(!app.identified_items.contains("Tome of Nothing"));
        assert_eq!(app.state, RunState::AwaitingInput);
        assert!(app.world.get::<&Tome>(tome).is_ok());
    }

    #[test]
    fn test_use_tome_identified_begins_study() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
        ));
        let tome = app.world.spawn((
            Item,
            Name("Tome of Firebolt".to_string()),
            Tome {
                spell_name: "Firebolt".to_string(),
                color: ManaColor::Orange,
                level: 1,
            },
            InBackpack { owner: player },
        ));
        app.identified_items.insert("Tome of Firebolt".to_string());
        app.use_item(tome);
        assert_eq!(app.state, RunState::ShowStudyTome);
        assert_eq!(app.study_tome_entity, Some(tome));
    }

    #[test]
    fn test_equip_from_use() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let sword = app.world.spawn((
            Item,
            Name("Sword".to_string()),
            Equippable {
                slot: EquipmentSlot::MainHand,
            },
            InBackpack { owner: player },
        ));

        app.use_item(sword);
        assert!(app.world.get::<&Equipped>(sword).is_ok());

        app.use_item(sword);
        assert!(app.world.get::<&Equipped>(sword).is_err());
    }
}
