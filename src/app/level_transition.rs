use crate::app::{App, EntitySnapshot, LevelData, RunState};
use crate::components::*;

impl App {
    pub fn go_to_level(&mut self, destination: (u16, Branch)) {
        let from = (self.dungeon_level, self.current_branch);

        if self.dungeon_level <= 1 && destination.0 < 1 {
            if self.escaping {
                self.state = RunState::Victory;
                self.log
                    .push("You escape the dungeon with the Amulet! You win!".to_string());
            } else {
                self.log
                    .push("You cannot go further up without the Amulet!".to_string());
            }
            return;
        }

        self.pack_entities();
        let current_entities = self.entities.clone();
        let traveling_entities: Vec<EntitySnapshot> = current_entities
            .iter()
            .filter(|e| e.is_player || e.in_backpack)
            .cloned()
            .collect();
        let level_entities: Vec<EntitySnapshot> = current_entities
            .into_iter()
            .filter(|e| !e.is_player && !e.in_backpack)
            .collect();

        self.levels.insert(
            (from.0, from.1),
            LevelData {
                map: self.map.clone(),
                entities: level_entities,
            },
        );

        let going_down = destination.0 > self.dungeon_level;

        self.dungeon_level = destination.0;
        self.current_branch = destination.1;

        if let Some(level_data) = self.levels.get(&(self.dungeon_level, self.current_branch)) {
            self.map = level_data.map.clone();
            self.entities = level_data.entities.clone();
            self.entities.extend(traveling_entities);
            if self.unpack_entities().is_ok() {
                let mut stairs_pos = (0, 0);

                if going_down {
                    for (_, (pos, stairs)) in self.world.query::<(&Position, &UpStairs)>().iter() {
                        if stairs.destination == from {
                            stairs_pos = (pos.x, pos.y);
                            break;
                        }
                        stairs_pos = (pos.x, pos.y); // Fallback
                    }
                } else {
                    for (_, (pos, stairs)) in self.world.query::<(&Position, &DownStairs)>().iter()
                    {
                        if stairs.destination == from {
                            stairs_pos = (pos.x, pos.y);
                            break;
                        }
                        stairs_pos = (pos.x, pos.y); // Fallback
                    }
                }

                let mut player_query = self.world.query::<(&mut Position, &Player)>();
                if let Some((_, (pos, _))) = player_query.iter().next() {
                    pos.x = stairs_pos.0;
                    pos.y = stairs_pos.1;
                }
            }
        } else {
            self.generate_level(traveling_entities);
        }

        let branch_name = match self.current_branch {
            Branch::Main => "Main Dungeon",
            Branch::Gardens => "Overgrown Gardens",
            Branch::Vaults => "Frozen Vaults",
        };

        if going_down {
            self.log.push(format!(
                "You descend to level {} of {}.",
                self.dungeon_level, branch_name
            ));
        } else {
            self.log.push(format!(
                "You ascend to level {} of {}.",
                self.dungeon_level, branch_name
            ));
        }
    }

    pub fn try_level_transition(&mut self) {
        let player_pos = {
            let Some(player_id) = self.get_player_id() else {
                return;
            };
            let Ok(pos) = self.world.get::<&Position>(player_id) else {
                return;
            };
            *pos
        };
        let mut transition_down = None;
        let mut transition_up = None;

        for (_, (pos, stairs)) in self.world.query::<(&Position, &DownStairs)>().iter() {
            if pos.x == player_pos.x && pos.y == player_pos.y {
                transition_down = Some(stairs.destination);
            }
        }
        for (_, (pos, stairs)) in self.world.query::<(&Position, &UpStairs)>().iter() {
            if pos.x == player_pos.x && pos.y == player_pos.y {
                transition_up = Some(stairs.destination);
            }
        }

        if let Some(dest) = transition_down {
            self.go_to_level(dest);
        } else if let Some(dest) = transition_up {
            self.go_to_level(dest);
        } else {
            self.log.push("There are no stairs here.".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::TileType;
    use hecs::World;

    fn setup_test_app() -> App {
        let mut app = App::new_test(42);
        app.world = World::new();
        app.map = crate::map::Map::new(80, 50);
        for t in app.map.tiles.iter_mut() {
            *t = TileType::Floor;
        }
        app.update_blocked_and_opaque();
        app
    }

    #[test]
    fn test_level_transition_persistence() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Position { x: 10, y: 10 },
            Player,
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 5,
            },
            Gold { amount: 0 },
            Renderable {
                glyph: '@',
                fg: ratatui::prelude::Color::Yellow,
            },
            RenderOrder::Player,
        ));

        app.world.spawn((
            Item,
            Name("Test Sword".to_string()),
            Weapon {
                power_bonus: 2,
                weight: WeaponWeight::Medium,
                damage_n_dice: 1,
                damage_die_type: 6,
                two_handed: false,
            },
            InBackpack { owner: player },
            Equippable {
                slot: EquipmentSlot::MainHand,
            },
            Equipped {
                slot: EquipmentSlot::MainHand,
            },
            Renderable {
                glyph: '/',
                fg: ratatui::prelude::Color::White,
            },
            RenderOrder::Item,
        ));

        // Sanity check
        assert_eq!(app.world.query::<&Player>().iter().count(), 1);
        assert_eq!(app.world.query::<&InBackpack>().iter().count(), 1);

        app.go_to_level((2, Branch::Main));

        // After transition, player and item should still exist
        let mut player_query = app.world.query::<&Player>();
        let (new_player_id, _) = player_query
            .iter()
            .next()
            .unwrap_or_else(|| panic!("Player lost during transition"));

        let mut item_query = app.world.query::<(&Name, &InBackpack, &Equipped)>();
        let mut found_item = false;
        for (_id, (name, backpack, _)) in item_query.iter() {
            if name.0 == "Test Sword" {
                assert_eq!(backpack.owner, new_player_id);
                found_item = true;
                break;
            }
        }
        assert!(found_item, "Item lost during transition");
    }

    #[test]
    fn test_try_level_transition_no_stairs() {
        let mut app = setup_test_app();
        app.world.spawn((Position { x: 10, y: 10 }, Player));
        app.try_level_transition();
        assert!(app.log.last().unwrap().contains("no stairs here"));
    }

    #[test]
    fn test_victory_condition() {
        let mut app = setup_test_app();
        app.dungeon_level = 1;
        app.current_branch = Branch::Main;
        app.escaping = true;
        app.go_to_level((0, Branch::Main));
        assert_eq!(app.state, RunState::Victory);
    }

    #[test]
    fn test_cannot_ascend_without_amulet() {
        let mut app = setup_test_app();
        app.dungeon_level = 1;
        app.current_branch = Branch::Main;
        app.escaping = false;
        app.go_to_level((0, Branch::Main));
        assert!(app.log.last().unwrap().contains("without the Amulet"));
        assert_eq!(app.dungeon_level, 1);
    }

    #[test]
    fn test_go_to_level_up_down() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Position { x: 10, y: 10 }, Player));

        // Going down to level 2
        app.go_to_level((2, Branch::Main));
        assert_eq!(app.dungeon_level, 2);

        // Add up stairs in level 2 pointing to level 1
        app.world.spawn((
            Position { x: 5, y: 5 },
            UpStairs {
                destination: (1, Branch::Main),
            },
        ));

        // Going back up to level 1
        app.go_to_level((1, Branch::Main));
        assert_eq!(app.dungeon_level, 1);
    }
}
