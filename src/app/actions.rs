use crate::actions::Action;
use crate::app::{App, RunState};

impl App {
    pub fn process_action(&mut self, action: Action) {
        match self.state {
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
            RunState::Dead | RunState::Victory => {
                if let Action::Quit | Action::CloseMenu = action {
                    self.exit = true;
                }
            }
            _ => {}
        }
    }

    pub fn apply_class_selection(&mut self) {
        use crate::components::*;
        let player_id = self.get_player_id().unwrap();

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
            self.world.insert_one(player_id, attrs).unwrap();
            let hp = 24 + Attributes::get_modifier(attrs.constitution);
            self.world.insert_one(player_id, CombatStats {
                hp,
                max_hp: hp,
                defense: 0,
                power: 5,
            }).unwrap();
            self.world.insert_one(player_id, Class { class: CharacterClass::Fighter }).unwrap();

            // Give starting equipment: Longsword, Shield, Chainmail
            let starting_items = ["Torch", "Health Potion", "Longsword", "Shield", "Chainmail"];
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
                    if item_name == "Longsword" || item_name == "Shield" || item_name == "Chainmail" || item_name == "Torch"
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
            _ => {}
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
