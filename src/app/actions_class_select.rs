use crate::app::App;
use crate::components::*;

impl App {
    pub fn apply_class_selection(&mut self) {
        let Some(player_id) = self.get_player_id() else {
            log::error!("apply_class_selection: player not found");
            return;
        };

        match self.class_selection {
            0 => {
                let attrs = Attributes {
                    strength: 15,
                    dexterity: 13,
                    constitution: 14,
                    intelligence: 8,
                    wisdom: 12,
                    charisma: 10,
                };
                self.world.insert_one(player_id, attrs).ok();
                let hp = 24 + Attributes::get_modifier(attrs.constitution);
                self.world
                    .insert_one(
                        player_id,
                        CombatStats {
                            hp,
                            max_hp: hp,
                            defense: 0,
                            power: 5,
                        },
                    )
                    .ok();
                self.world
                    .insert_one(
                        player_id,
                        Class {
                            class: CharacterClass::Fighter,
                        },
                    )
                    .ok();

                let starting_items = [
                    "Chainmail",
                    "Shield",
                    "Torch",
                    "Longsword",
                    "Carbine",
                    "Health Potion",
                ];
                for item_name in starting_items {
                    if let Some(item_raw) = self
                        .content
                        .items
                        .iter()
                        .find(|i| i.name == item_name)
                        .cloned()
                    {
                        let item_id = crate::spawner::spawn_item_in_backpack(
                            &mut self.world,
                            player_id,
                            &item_raw,
                        );
                        self.identified_items.insert(item_name.to_string());
                        if item_name == "Longsword"
                            || item_name == "Shield"
                            || item_name == "Chainmail"
                            || item_name == "Torch"
                            || item_name == "Carbine"
                        {
                            self.equip_item(item_id);
                        }
                    }
                }
            }
            1 | 2 => {
                let is_nihil = self.class_selection == 1;
                let (class, pool, spell_name) = if is_nihil {
                    (
                        CharacterClass::Nihil,
                        ManaPool {
                            current_orange: 0,
                            max_orange: 0,
                            current_purple: 1,
                            max_purple: 1,
                            regen_cooldown: ManaPool::MANA_REGEN_INTERVAL,
                        },
                        "Venom Dart",
                    )
                } else {
                    (
                        CharacterClass::Solari,
                        ManaPool {
                            current_orange: 1,
                            max_orange: 1,
                            current_purple: 0,
                            max_purple: 0,
                            regen_cooldown: ManaPool::MANA_REGEN_INTERVAL,
                        },
                        "Magic Missile",
                    )
                };

                let attrs = Attributes {
                    strength: 8,
                    dexterity: 12,
                    constitution: 13,
                    intelligence: 10,
                    wisdom: 12,
                    charisma: 15,
                };
                self.world.insert_one(player_id, attrs).ok();
                let hp = 18 + Attributes::get_modifier(attrs.constitution);
                self.world
                    .insert_one(
                        player_id,
                        CombatStats {
                            hp,
                            max_hp: hp,
                            defense: 0,
                            power: 3,
                        },
                    )
                    .ok();
                self.world.insert_one(player_id, Class { class }).ok();
                self.world.insert_one(player_id, pool).ok();

                let mut book = Spellbook::default();
                match self.content.find_spell(spell_name) {
                    Ok(spell) => book.spells.push(spell),
                    Err(e) => log::error!("starting spell '{}' missing: {}", spell_name, e),
                }
                self.world.insert_one(player_id, book).ok();

                let starting_items = [
                    "Leather Armor",
                    "Torch",
                    "Dagger",
                    "Service Pistol",
                    "Health Potion",
                ];
                for item_name in starting_items {
                    if let Some(item_raw) = self
                        .content
                        .items
                        .iter()
                        .find(|i| i.name == item_name)
                        .cloned()
                    {
                        let item_id = crate::spawner::spawn_item_in_backpack(
                            &mut self.world,
                            player_id,
                            &item_raw,
                        );
                        self.identified_items.insert(item_name.to_string());
                        if item_name == "Dagger"
                            || item_name == "Leather Armor"
                            || item_name == "Torch"
                            || item_name == "Service Pistol"
                        {
                            self.equip_item(item_id);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_nihil_initiate_class() {
        let mut app = App::new_test(42);
        app.generate_level(Vec::new());
        let player_id = app
            .get_player_id()
            .expect("player exists after generate_level");
        app.class_selection = 1;
        app.apply_class_selection();

        let class = app.world.get::<&Class>(player_id).unwrap();
        assert_eq!(class.class, CharacterClass::Nihil);
        drop(class);

        let pool = app.world.get::<&ManaPool>(player_id).unwrap();
        assert_eq!(pool.current_purple, 1);
        assert_eq!(pool.max_purple, 1);
        assert_eq!(pool.current_orange, 0);
        assert_eq!(pool.max_orange, 0);
        drop(pool);

        let book = app.world.get::<&Spellbook>(player_id).unwrap();
        assert_eq!(book.spells.len(), 1);
        assert_eq!(book.spells[0].title, "Venom Dart");

        let mut has_pistol = false;
        for (id, (name, backpack)) in app.world.query::<(&Name, &InBackpack)>().iter() {
            if backpack.owner == player_id && name.0 == "Service Pistol" {
                has_pistol = true;
                assert!(app.world.get::<&Equipped>(id).is_ok());
            }
        }
        assert!(
            has_pistol,
            "Nihil Initiate should start with Service Pistol"
        );
    }

    #[test]
    fn test_apply_solari_initiate_class() {
        let mut app = App::new_test(42);
        app.generate_level(Vec::new());
        let player_id = app
            .get_player_id()
            .expect("player exists after generate_level");
        app.class_selection = 2;
        app.apply_class_selection();

        let class = app.world.get::<&Class>(player_id).unwrap();
        assert_eq!(class.class, CharacterClass::Solari);
        drop(class);

        let pool = app.world.get::<&ManaPool>(player_id).unwrap();
        assert_eq!(pool.current_orange, 1);
        assert_eq!(pool.max_orange, 1);
        assert_eq!(pool.current_purple, 0);
        assert_eq!(pool.max_purple, 0);
        drop(pool);

        let book = app.world.get::<&Spellbook>(player_id).unwrap();
        assert_eq!(book.spells.len(), 1);
        assert_eq!(book.spells[0].title, "Magic Missile");

        let mut has_pistol = false;
        for (id, (name, backpack)) in app.world.query::<(&Name, &InBackpack)>().iter() {
            if backpack.owner == player_id && name.0 == "Service Pistol" {
                has_pistol = true;
                assert!(app.world.get::<&Equipped>(id).is_ok());
            }
        }
        assert!(
            has_pistol,
            "Solari Initiate should start with Service Pistol"
        );
    }
}
