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

    pub fn get_player_stats(&self) -> (i32, i32, i32) {
        let Some(player_id) = self.get_player_id() else {
            return (0, 0, 0);
        };
        let (power, av) = {
            let mut p = 0;
            let mut a = 0;
            if let Ok(stats) = self.world.get::<&CombatStats>(player_id) {
                p = stats.power;
                a = stats.defense;
            }
            (p, a)
        };

        let mut total_power = power;
        let mut total_av = av;

        // Add attribute bonuses
        let mut str_mod = 0;
        let dex_mod = self.get_dex_modifier(player_id);
        if let Ok(attr) = self.world.get::<&Attributes>(player_id) {
            str_mod = Attributes::get_modifier(attr.strength);
        }

        // Add equipment bonuses
        let mut weapon_equipped = false;

        for (id, (eq, _)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
            if let Ok(backpack) = self.world.get::<&InBackpack>(id) {
                if backpack.owner == player_id {
                    if let Ok(weapon) = self.world.get::<&Weapon>(id) {
                        if eq.slot == EquipmentSlot::MainHand {
                            total_power += weapon.power_bonus;
                            let mut m = str_mod;
                            if weapon.weight == WeaponWeight::Light {
                                m = dex_mod;
                            }
                            if weapon.two_handed {
                                m = (m as f32 * 1.5) as i32;
                            }
                            total_power += m;
                            weapon_equipped = true;
                        }
                    }
                    if let Ok(armor) = self.world.get::<&Armor>(id) {
                        total_av += armor.defense_bonus;
                    }
                    if let Ok(strength) = self.world.get::<&Strength>(id) {
                        total_power += strength.amount;
                    }
                }
            }
        }
        if !weapon_equipped {
            total_power += str_mod;
        }

        total_av += dex_mod;
        let dodge_dc = 10 + dex_mod;

        (total_power, total_av, dodge_dc)
    }

    pub fn recalculate_player_max_hp(&mut self) {
        let Some(player_id) = self.get_player_id() else {
            return;
        };
        let (level, con_mod) = {
            let exp = self
                .world
                .get::<&Experience>(player_id)
                .ok()
                .map(|e| e.level)
                .unwrap_or(1);
            let attr = self
                .world
                .get::<&Attributes>(player_id)
                .ok()
                .map(|a| a.constitution)
                .unwrap_or(10);
            (exp, Attributes::get_modifier(attr))
        };

        if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
            let old_max = stats.max_hp;
            stats.max_hp = 16 + (level * 8) + (level * con_mod);
            let diff = stats.max_hp - old_max;
            if diff > 0 {
                stats.hp += diff;
            }
        }
    }

    pub fn get_entity_name(&self, entity: hecs::Entity) -> String {
        if let Ok(name) = self.world.get::<&Name>(entity) {
            return name.0.clone();
        }
        if self.world.get::<&Player>(entity).is_ok() {
            return "Player".to_string();
        }
        "Something".to_string()
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
        let Some(player_id) = self.get_player_id() else {
            return;
        };
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
        self.update_fov();
    }

    pub fn increment_attribute(&mut self, player_id: hecs::Entity, cursor: usize) {
        if let Ok(mut attr) = self.world.get::<&mut Attributes>(player_id) {
            match cursor {
                0 => {
                    attr.strength += 1;
                    self.log.push("Strength increased!".to_string());
                }
                1 => {
                    attr.dexterity += 1;
                    self.log.push("Dexterity increased!".to_string());
                }
                2 => {
                    attr.constitution += 1;
                    self.log.push("Constitution increased!".to_string());
                }
                3 => {
                    attr.intelligence += 1;
                    self.log.push("Intelligence increased!".to_string());
                }
                4 => {
                    attr.wisdom += 1;
                    self.log.push("Wisdom increased!".to_string());
                }
                5 => {
                    attr.charisma += 1;
                    self.log.push("Charisma increased!".to_string());
                }
                _ => {}
            }
        }
    }

    pub fn add_player_xp(&mut self, xp: i32) {
        let Some(player_id) = self.get_player_id() else {
            return;
        };
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
    fn test_get_item_name_obfuscated() {
        let mut app = setup_test_app();
        let item = app.world.spawn((
            Name("Mysterious Potion".to_string()),
            ObfuscatedName("Bubbling Blue Liquid".to_string()),
        ));

        assert_eq!(app.get_item_name(item), "Bubbling Blue Liquid");

        app.identify_item(item);
        assert_eq!(app.get_item_name(item), "Mysterious Potion");
    }

    #[test]
    fn test_refresh_player_render_light() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Renderable {
                glyph: '@',
                fg: Color::White,
            },
        ));
        let torch = app.world.spawn((
            Item,
            Equipped {
                slot: EquipmentSlot::OffHand,
            },
            InBackpack { owner: player },
            LightSource {
                range: 10,
                base_range: 10,
                color: (255, 255, 255),
                remaining_turns: None,
                flicker: false,
            },
        ));

        app.refresh_player_render();

        {
            let player_light = app.world.get::<&LightSource>(player).unwrap();
            assert_eq!(player_light.range, 10);
        }

        // Remove torch, should revert to default dim light
        app.world.despawn(torch).unwrap();
        app.refresh_player_render();
        {
            let player_light2 = app.world.get::<&LightSource>(player).unwrap();
            assert_eq!(player_light2.range, 2);
        }
    }

    #[test]
    fn test_add_player_xp_levelup() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Experience {
                level: 1,
                xp: 0,
                next_level_xp: 100,
                xp_reward: 0,
            },
        ));

        app.add_player_xp(150);

        let exp = app.world.get::<&Experience>(player).unwrap();
        assert_eq!(exp.level, 2);
        assert_eq!(exp.xp, 50);
        assert_eq!(app.state, crate::app::RunState::LevelUp);
    }
}
