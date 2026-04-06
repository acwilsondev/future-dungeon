use crate::app::{App, RunState};
use crate::components::*;

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
            || self.world.get::<&RangedWeapon>(item_id).is_ok()
        {
            if self.world.get::<&RangedWeapon>(item_id).is_ok() {
                // Check for ammo
                let has_ammo = self
                    .world
                    .query::<(&Ammunition, &InBackpack)>()
                    .iter()
                    .any(|(_, (_, backpack))| backpack.owner == player_id);
                if !has_ammo {
                    self.log
                        .push("You have no ammunition for this weapon!".to_string());
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
