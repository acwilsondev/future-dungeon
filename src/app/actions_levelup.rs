use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;

impl App {
    pub fn handle_level_up_input(&mut self, action: Action) {
        match action {
            Action::MenuUp => {
                if self.level_up_cursor > 0 {
                    self.level_up_cursor -= 1;
                }
            }
            Action::MenuDown => {
                if self.level_up_cursor < 3 {
                    self.level_up_cursor += 1;
                }
            }
            Action::MenuSelect => {
                if let Some(player_id) = self.get_player_id() {
                    match self.level_up_cursor {
                    0 => {
                        if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                            stats.max_hp += 10;
                            stats.hp += 10;
                        }
                        if let Ok(mut perks) = self.world.get::<&mut Perks>(player_id) {
                            perks.traits.push(Perk::Toughness);
                        }
                        self.log
                            .push("You chose Toughness! Max HP increased.".to_string());
                    }
                    1 => {
                        if let Ok(mut viewshed) = self.world.get::<&mut Viewshed>(player_id) {
                            viewshed.visible_tiles += 2;
                        }
                        if let Ok(mut perks) = self.world.get::<&mut Perks>(player_id) {
                            perks.traits.push(Perk::EagleEye);
                        }
                        self.log
                            .push("You chose Eagle Eye! FOV increased.".to_string());
                    }
                    2 => {
                        if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                            stats.power += 2;
                        }
                        if let Ok(mut perks) = self.world.get::<&mut Perks>(player_id) {
                            perks.traits.push(Perk::Strong);
                        }
                        self.log
                            .push("You chose Strong! Power increased.".to_string());
                    }
                    3 => {
                        if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                            stats.defense += 1;
                        }
                        if let Ok(mut perks) = self.world.get::<&mut Perks>(player_id) {
                            perks.traits.push(Perk::ThickSkin);
                        }
                        self.log
                            .push("You chose Thick Skin! Defense increased.".to_string());
                    }
                    _ => {}
                    }
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
    use hecs::World;

    fn setup_test_app() -> App {
        let mut app = App::new_random();
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
    fn test_level_up_perks() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 5 },
            Viewshed { visible_tiles: 8 },
            Perks { traits: Vec::new() },
            Position { x: 0, y: 0 }
        ));

        // Test perk 0: Toughness
        app.level_up_cursor = 0;
        app.handle_level_up_input(Action::MenuSelect);
        {
            let stats = app.world.get::<&CombatStats>(player).unwrap();
            assert_eq!(stats.max_hp, 20);
            let perks = app.world.get::<&Perks>(player).unwrap();
            assert!(perks.traits.contains(&Perk::Toughness));
        }

        // Test perk 1: Eagle Eye
        app.level_up_cursor = 1;
        app.handle_level_up_input(Action::MenuSelect);
        {
            let viewshed = app.world.get::<&Viewshed>(player).unwrap();
            assert_eq!(viewshed.visible_tiles, 10);
            let perks = app.world.get::<&Perks>(player).unwrap();
            assert!(perks.traits.contains(&Perk::EagleEye));
        }

        // Test perk 2: Strong
        app.level_up_cursor = 2;
        app.handle_level_up_input(Action::MenuSelect);
        {
            let stats = app.world.get::<&CombatStats>(player).unwrap();
            assert_eq!(stats.power, 7);
            let perks = app.world.get::<&Perks>(player).unwrap();
            assert!(perks.traits.contains(&Perk::Strong));
        }

        // Test perk 3: Thick Skin
        app.level_up_cursor = 3;
        app.handle_level_up_input(Action::MenuSelect);
        {
            let stats = app.world.get::<&CombatStats>(player).unwrap();
            assert_eq!(stats.defense, 1);
            let perks = app.world.get::<&Perks>(player).unwrap();
            assert!(perks.traits.contains(&Perk::ThickSkin));
        }
    }
}
