use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;

impl App {
    pub fn process_action(&mut self, action: Action) {
        match self.state {
            RunState::MainMenu => self.handle_main_menu_input(action),
            RunState::ShowClassSelection => match action {
                Action::MenuUp => {
                    // if self.class_selection > 0 { self.class_selection -= 1; }
                }
                Action::MenuDown => {
                    // if self.class_selection < max_classes - 1 { self.class_selection += 1; }
                }
                Action::MenuSelect => {
                    self.apply_class_selection();
                    self.state = RunState::AwaitingInput;
                }
                Action::Quit => self.exit = true,
                _ => {}
            },
            RunState::AwaitingInput => self.handle_awaiting_input(action),
            RunState::ShowLogHistory => match action {
                Action::CloseMenu | Action::OpenLogHistory => self.state = RunState::AwaitingInput,
                Action::MenuUp => {
                    if self.log_cursor > 0 {
                        self.log_cursor -= 1;
                    }
                }
                Action::MenuDown => {
                    if self.log_cursor < self.log.len().saturating_sub(1) {
                        self.log_cursor += 1;
                    }
                }
                _ => {}
            },
            RunState::ShowBestiary => match action {
                Action::CloseMenu | Action::OpenBestiary => self.state = RunState::AwaitingInput,
                Action::MenuUp => {
                    if self.bestiary_cursor > 0 {
                        self.bestiary_cursor -= 1;
                    }
                }
                Action::MenuDown => {
                    let count = self.encountered_monsters.len();
                    if count > 0 && self.bestiary_cursor < count - 1 {
                        self.bestiary_cursor += 1;
                    }
                }
                _ => {}
            },
            RunState::ShowInventory => self.handle_inventory_input(action),
            RunState::ShowTargeting => self.handle_targeting_input(action),
            RunState::ShowHelp => {
                if let Action::CloseMenu | Action::OpenHelp = action {
                    self.state = RunState::AwaitingInput;
                }
            }
            RunState::LevelUp => self.handle_level_up_input(action),
            RunState::ShowShop => self.handle_shop_input(action),
            RunState::ShowIdentify => self.handle_identify_input(action),
            RunState::ShowAlchemy => self.handle_alchemy_input(action),
            RunState::ShowResetShrine => self.handle_respec_input(action),
            RunState::ShowDebugConsole => self.handle_debug_console_input(action),
            RunState::Dead | RunState::Victory => {
                if let Action::Quit | Action::CloseMenu = action {
                    self.exit = true;
                }
            }
            _ => {}
        }
    }

    pub fn handle_main_menu_input(&mut self, action: Action) {
        let has_save = crate::persistence::has_save_game();
        let max_options = if has_save { 3 } else { 2 };

        match action {
            Action::MenuUp => {
                if self.main_menu_cursor > 0 {
                    self.main_menu_cursor -= 1;
                }
            }
            Action::MenuDown => {
                if self.main_menu_cursor < max_options - 1 {
                    self.main_menu_cursor += 1;
                }
            }
            Action::MenuSelect => {
                let selection = if !has_save && self.main_menu_cursor == 1 {
                    2 // Exit if no save and selected 2nd option
                } else {
                    self.main_menu_cursor
                };

                match selection {
                    0 => {
                        // New Game
                        self.generate_level(Vec::new());
                        self.state = RunState::ShowClassSelection;
                    }
                    1 => {
                        // Load Game
                        if let Ok(Some(loaded_app)) = crate::persistence::load_game() {
                            *self = loaded_app;
                            self.state = RunState::AwaitingInput;
                        }
                    }
                    2 => {
                        // Exit
                        self.exit = true;
                    }
                    _ => {}
                }
            }
            Action::Quit => self.exit = true,
            _ => {}
        }
    }

    pub fn apply_class_selection(&mut self) {
        use crate::components::*;
        let Some(player_id) = self.get_player_id() else {
            log::error!("apply_class_selection: player not found");
            return;
        };

        if self.class_selection == 0 {
            // Fighter
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

            // Give starting equipment: Longsword, Shield, Chainmail
            let starting_items = ["Chainmail", "Shield", "Torch", "Longsword", "Health Potion"];
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
                    {
                        self.equip_item(item_id);
                    }
                }
            }
        }
    }

    fn handle_awaiting_input(&mut self, action: Action) {
        match action {
            Action::Quit => self.exit = true,
            Action::MovePlayer(dx, dy) => self.move_player(dx, dy),
            Action::PickUpItem => self.pick_up_item(),
            Action::OpenInventory => self.state = RunState::ShowInventory,
            Action::OpenHelp => self.state = RunState::ShowHelp,
            Action::OpenLogHistory => {
                self.state = RunState::ShowLogHistory;
                self.log_cursor = self.log.len().saturating_sub(1);
            }
            Action::OpenBestiary => {
                self.state = RunState::ShowBestiary;
                self.bestiary_cursor = 0;
            }
            Action::TryLevelTransition => self.try_level_transition(),
            Action::Target => self.trigger_ranged_targeting(),
            Action::Wait => self.state = RunState::MonsterTurn,
            Action::ToggleDebugConsole => {
                self.state = RunState::ShowDebugConsole;
                self.debug_console_buffer.clear();
            }
            _ => {}
        }
    }

    pub fn handle_debug_console_input(&mut self, action: Action) {
        match action {
            Action::TypeChar(c) => {
                self.debug_console_buffer.push(c);
            }
            Action::Backspace => {
                self.debug_console_buffer.pop();
            }
            Action::SubmitCommand => {
                let command = self.debug_console_buffer.clone();
                self.execute_debug_command(&command);
                self.debug_console_buffer.clear();
                if self.state == RunState::ShowDebugConsole {
                    self.state = RunState::AwaitingInput;
                }
            }
            Action::ToggleDebugConsole | Action::CloseMenu => {
                self.state = RunState::AwaitingInput;
            }
            _ => {}
        }
    }

    fn execute_debug_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "help" => {
                self.log.push(
                    "Debug Commands: spawn [name], teleport [lvl], heal, reveal, levelup, god"
                        .to_string(),
                );
            }
            "spawn" => {
                if parts.len() > 1 {
                    let item_name = parts[1..].join(" ");
                    if let Some(item_raw) = self
                        .content
                        .items
                        .iter()
                        .find(|i| i.name == item_name)
                        .cloned()
                    {
                        let Some(player_id) = self.get_player_id() else {
                            return;
                        };
                        let Some(pos) = self.world.get::<&Position>(player_id).ok().map(|p| *p) else {
                            return;
                        };
                        crate::spawner::spawn_item(&mut self.world, pos.x, pos.y, &item_raw);
                        self.log.push(format!("Debug: Spawned {}", item_name));
                    } else {
                        self.log
                            .push(format!("Debug: Item '{}' not found", item_name));
                    }
                }
            }
            "teleport" => {
                if parts.len() > 1 {
                    if let Ok(level) = parts[1].parse::<u16>() {
                        let dest = (level, self.current_branch);
                        self.go_to_level(dest);
                        self.log
                            .push(format!("Debug: Teleported to level {}", level));
                    }
                }
            }
            "heal" => {
                let Some(player_id) = self.get_player_id() else {
                    return;
                };
                if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                    stats.hp = stats.max_hp;
                    self.log.push("Debug: Healed player".to_string());
                }
                self.world.remove_one::<Poison>(player_id).ok();
                self.world.remove_one::<Confusion>(player_id).ok();
            }
            "reveal" => {
                for r in self.map.revealed.iter_mut() {
                    *r = true;
                }
                self.log.push("Debug: Revealed map".to_string());
            }
            "levelup" => {
                let Some(player_id) = self.get_player_id() else {
                    return;
                };
                let needed = if let Ok(exp) = self.world.get::<&Experience>(player_id) {
                    Some(exp.next_level_xp - exp.xp)
                } else {
                    None
                };
                if let Some(needed) = needed {
                    self.add_player_xp(needed);
                    self.log.push("Debug: Leveled up player".to_string());
                }
            }
            "god" => {
                self.god_mode = !self.god_mode;
                self.log
                    .push(format!("Debug: God mode set to {}", self.god_mode));
            }
            _ => {
                self.log
                    .push(format!("Debug: Unknown command '{}'", parts[0]));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_app() -> App {
        App::new_test(42)
    }

    #[test]
    fn test_open_close_menus() {
        let mut app = setup_test_app();
        app.state = RunState::AwaitingInput;

        app.process_action(Action::OpenInventory);
        assert_eq!(app.state, RunState::ShowInventory);
        app.process_action(Action::CloseMenu);
        assert_eq!(app.state, RunState::AwaitingInput);

        app.process_action(Action::OpenHelp);
        assert_eq!(app.state, RunState::ShowHelp);
        app.process_action(Action::CloseMenu);
        assert_eq!(app.state, RunState::AwaitingInput);

        app.process_action(Action::OpenLogHistory);
        assert_eq!(app.state, RunState::ShowLogHistory);
        app.process_action(Action::OpenLogHistory); // Toggle off
        assert_eq!(app.state, RunState::AwaitingInput);
    }

    #[test]
    fn test_log_navigation() {
        let mut app = setup_test_app();
        app.log = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        app.state = RunState::ShowLogHistory;
        app.log_cursor = 2;

        app.process_action(Action::MenuUp);
        assert_eq!(app.log_cursor, 1);
        app.process_action(Action::MenuDown);
        assert_eq!(app.log_cursor, 2);
    }

    #[test]
    fn test_bestiary_navigation() {
        let mut app = setup_test_app();
        app.encountered_monsters.insert("Orc".to_string());
        app.encountered_monsters.insert("Goblin".to_string());
        app.state = RunState::ShowBestiary;
        app.bestiary_cursor = 0;

        app.process_action(Action::MenuDown);
        assert_eq!(app.bestiary_cursor, 1);
        app.process_action(Action::MenuUp);
        assert_eq!(app.bestiary_cursor, 0);
    }

    #[test]
    fn test_quit_action() {
        let mut app = setup_test_app();
        app.state = RunState::AwaitingInput;
        app.process_action(Action::Quit);
        assert!(app.exit);

        let mut app2 = setup_test_app();
        app2.state = RunState::Dead;
        app2.process_action(Action::Quit);
        assert!(app2.exit);
    }

    #[test]
    fn test_wait_action() {
        let mut app = setup_test_app();
        app.state = RunState::AwaitingInput;
        app.process_action(Action::Wait);
        assert_eq!(app.state, RunState::MonsterTurn);
    }
}
