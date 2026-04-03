use crate::app::App;
use crate::components::*;
use ratatui::prelude::Color;

impl App {
    pub fn get_player_id(&self) -> Option<hecs::Entity> {
        self.world
            .query::<&Player>()
            .iter()
            .next()
            .map(|(id, _)| id)
    }

    pub fn get_player_stats(&self) -> (i32, i32) {
        let player_id = self.get_player_id().expect("Player not found");
        let base_stats = self
            .world
            .get::<&CombatStats>(player_id)
            .expect("Player has no CombatStats");
        let mut power = base_stats.power;
        let mut defense = base_stats.defense;

        // Add equipment bonuses
        for (id, (_eq, backpack)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
            if backpack.owner == player_id {
                if let Ok(weapon) = self.world.get::<&Weapon>(id) {
                    power += weapon.power_bonus;
                }
                if let Ok(armor) = self.world.get::<&Armor>(id) {
                    defense += armor.defense_bonus;
                }
                if let Ok(strength) = self.world.get::<&Strength>(id) {
                    power += strength.amount;
                }
            }
        }
        (power, defense)
    }

    pub fn get_item_name(&self, item_id: hecs::Entity) -> String {
        if let Ok(name) = self.world.get::<&Name>(item_id) {
            if self.world.get::<&ObfuscatedName>(item_id).is_ok()
                && !self.identified_items.contains(&name.0)
            {
                if let Ok(obfuscated) = self.world.get::<&ObfuscatedName>(item_id) {
                    return obfuscated.0.clone();
                }
            }
            return name.0.clone();
        }
        "Unnamed Item".to_string()
    }

    pub fn identify_item(&mut self, item_id: hecs::Entity) {
        if let Ok(name) = self.world.get::<&Name>(item_id) {
            self.identified_items.insert(name.0.clone());
        }
    }

    pub fn refresh_player_render(&mut self) {
        let player_id = self.get_player_id().expect("Player not found");
        if let Ok(mut render) = self.world.get::<&mut Renderable>(player_id) {
            render.fg = Color::Yellow;
            render.glyph = '@';
        }

        let mut light_to_apply = LightSource {
            range: 2,
            base_range: 2,
            color: (150, 150, 100),
            remaining_turns: None,
            flicker: false,
        };

        for (id, (_eq, backpack)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
            if backpack.owner == player_id {
                if let Ok(light) = self.world.get::<&LightSource>(id) {
                    light_to_apply = *light;
                    break;
                }
            }
        }

        self.world.insert_one(player_id, light_to_apply).ok();
    }

    pub fn add_player_xp(&mut self, xp: i32) {
        let player_id = self.get_player_id().expect("Player not found");
        if let Ok(mut exp) = self.world.get::<&mut Experience>(player_id) {
            exp.xp += xp;
            if exp.xp >= exp.next_level_xp {
                self.state = crate::app::RunState::LevelUp;
                self.log
                    .push("Congratulations! You leveled up!".to_string());
                exp.level += 1;
                exp.xp -= exp.next_level_xp;
                exp.next_level_xp = (exp.next_level_xp as f32 * 1.5) as i32;
            }
        }
    }
}
