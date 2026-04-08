use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;

impl App {
    pub fn init_respec(&mut self) {
        let player_id = self.get_player_id().unwrap();
        
        let level = self.world.get::<&Experience>(player_id).map(|e| e.level).unwrap_or(1);
        self.respec_points = level - 1;

        let class = self.world.get::<&Class>(player_id).map(|c| c.class).unwrap_or(CharacterClass::Fighter);

        // Base attributes for Fighter (currently only class)
        let base_attrs = match class {
            CharacterClass::Fighter => Attributes {
                strength: 15,
                dexterity: 13,
                constitution: 14,
                intelligence: 8,
                wisdom: 12,
                charisma: 10,
            },
        };

        self.world.insert_one(player_id, base_attrs).unwrap();
        self.recalculate_player_max_hp();
        self.log.push(format!("You have {} attribute points to redistribute.", self.respec_points));
    }

    pub fn handle_respec_input(&mut self, action: Action) {
        match action {
            Action::MenuUp => {
                if self.level_up_cursor > 0 {
                    self.level_up_cursor -= 1;
                }
            }
            Action::MenuDown => {
                if self.level_up_cursor < 5 {
                    self.level_up_cursor += 1;
                }
            }
            Action::MenuSelect => {
                if self.respec_points > 0 {
                    let player_id = self.get_player_id().unwrap();
                    if let Ok(mut attr) = self.world.get::<&mut Attributes>(player_id) {
                        match self.level_up_cursor {
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
                    self.respec_points -= 1;
                    self.recalculate_player_max_hp();
                    
                    if self.respec_points == 0 {
                        self.log.push("You have finished redistributing your attributes.".to_string());
                        self.state = RunState::AwaitingInput;
                    }
                } else {
                    self.state = RunState::AwaitingInput;
                }
            }
            _ => {}
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
    fn test_respec_flow() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 5 },
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            Experience { level: 3, xp: 0, next_level_xp: 0, xp_reward: 0 },
            Class { class: CharacterClass::Fighter },
            Position { x: 0, y: 0 }
        ));

        app.init_respec();
        assert_eq!(app.respec_points, 2);
        {
            let attr = app.world.get::<&Attributes>(player).unwrap();
            assert_eq!(attr.strength, 15); // Base Fighter STR
        }

        // Increase STR
        app.level_up_cursor = 0;
        app.handle_respec_input(Action::MenuSelect);
        assert_eq!(app.respec_points, 1);
        {
            let attr = app.world.get::<&Attributes>(player).unwrap();
            assert_eq!(attr.strength, 16);
        }

        // Increase CON
        app.level_up_cursor = 2;
        app.handle_respec_input(Action::MenuSelect);
        assert_eq!(app.respec_points, 0);
        assert_eq!(app.state, RunState::AwaitingInput);
        {
            let attr = app.world.get::<&Attributes>(player).unwrap();
            assert_eq!(attr.constitution, 15); // 14 + 1
        }
    }
}
