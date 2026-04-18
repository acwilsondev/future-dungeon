use crate::actions::Action;
use crate::app::{App, RunState};

impl App {
    pub fn handle_level_up_input(&mut self, action: Action) {
        match action {
            Action::MenuUp if self.level_up_cursor > 0 => {
                self.level_up_cursor -= 1;
            }
            Action::MenuDown if self.level_up_cursor < 5 => {
                self.level_up_cursor += 1;
            }
            Action::MenuSelect => {
                if let Some(player_id) = self.get_player_id() {
                    let cursor = self.level_up_cursor;
                    self.increment_attribute(player_id, cursor);
                    self.recalculate_player_max_hp();
                }
                self.state = RunState::MonsterTurn;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use hecs::World;

    fn setup_test_app() -> App {
        let mut app = App::new_test(42);
        app.world = World::new();
        app
    }

    #[test]
    fn test_level_up_navigation() {
        let mut app = setup_test_app();
        app.level_up_cursor = 0;
        app.handle_level_up_input(Action::MenuDown);
        assert_eq!(app.level_up_cursor, 1);
        app.handle_level_up_input(Action::MenuUp);
        assert_eq!(app.level_up_cursor, 0);
    }

    #[test]
    fn test_level_up_attributes() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 5,
            },
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            Experience {
                level: 1,
                xp: 0,
                next_level_xp: 100,
                xp_reward: 0,
            },
            Position { x: 0, y: 0 },
        ));

        // Test choice 0: Strength
        app.level_up_cursor = 0;
        app.handle_level_up_input(Action::MenuSelect);
        {
            let attr = app.world.get::<&Attributes>(player).unwrap();
            assert_eq!(attr.strength, 11);
        }

        // Test choice 2: Constitution (affects HP)
        app.level_up_cursor = 2;
        app.handle_level_up_input(Action::MenuSelect);
        {
            let attr = app.world.get::<&Attributes>(player).unwrap();
            assert_eq!(attr.constitution, 11);
            // CON mod is still 0 at 11.
            // recalculate_max_hp: 16 + (1 * 8) + (1 * 0) = 24
            let stats = app.world.get::<&CombatStats>(player).unwrap();
            assert_eq!(stats.max_hp, 24);
        }

        // Increase CON to 12 (mod 1)
        app.level_up_cursor = 2;
        app.handle_level_up_input(Action::MenuSelect);
        {
            let attr = app.world.get::<&Attributes>(player).unwrap();
            assert_eq!(attr.constitution, 12);
            // CON mod is now 1.
            // recalculate_max_hp: 16 + (1 * 8) + (1 * 1) = 25
            let stats = app.world.get::<&CombatStats>(player).unwrap();
            assert_eq!(stats.max_hp, 25);
        }
    }
}
