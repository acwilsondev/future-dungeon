use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;
use rand::Rng;

impl App {
    /// Begin the Study flow for an identified Tome in the player's backpack.
    /// Transitions to `RunState::ShowStudyTome` for confirmation.
    #[allow(dead_code)]
    pub fn begin_study_tome(&mut self, tome_id: hecs::Entity) {
        if self.world.get::<&Tome>(tome_id).is_err() {
            return;
        }
        self.study_tome_entity = Some(tome_id);
        self.yes_no_cursor = 0;
        self.state = RunState::ShowStudyTome;
    }

    pub fn handle_study_tome_input(&mut self, action: Action) {
        match action {
            Action::Confirm => self.attempt_study_tome(),
            Action::Decline => {
                self.study_tome_entity = None;
                self.state = RunState::AwaitingInput;
            }
            Action::MenuUp => self.yes_no_cursor = 0,
            Action::MenuDown => self.yes_no_cursor = 1,
            Action::MenuSelect => {
                if self.yes_no_cursor == 0 {
                    self.attempt_study_tome();
                } else {
                    self.study_tome_entity = None;
                    self.state = RunState::AwaitingInput;
                }
            }
            _ => {}
        }
    }

    fn attempt_study_tome(&mut self) {
        let Some(tome_id) = self.study_tome_entity.take() else {
            self.state = RunState::AwaitingInput;
            return;
        };
        let Some(player_id) = self.get_player_id() else {
            self.state = RunState::AwaitingInput;
            return;
        };

        let (spell_name, level) = match self.world.get::<&Tome>(tome_id) {
            Ok(t) => (t.spell_name.clone(), t.level),
            Err(_) => {
                self.state = RunState::AwaitingInput;
                return;
            }
        };

        let cha_mod = self
            .world
            .get::<&Attributes>(player_id)
            .map(|a| Attributes::get_modifier(a.charisma))
            .unwrap_or(0);
        let dc = 5 + 2 * level as i32;
        let roll = self.rng.random_range(1..=20);
        let success = roll + cha_mod >= dc;

        if success {
            match self.content.find_spell(&spell_name) {
                Ok(spell) => {
                    let title = spell.title.clone();
                    let has_book = self.world.get::<&Spellbook>(player_id).is_ok();
                    if !has_book {
                        let _ = self.world.insert_one(player_id, Spellbook::default());
                    }
                    if let Ok(mut book) = self.world.get::<&mut Spellbook>(player_id) {
                        if !book.spells.iter().any(|s| s.title == title) {
                            book.spells.push(spell);
                            self.log.push(format!("You learned {}.", title));
                        } else {
                            self.log.push(format!("You already know {}.", title));
                        }
                    }
                    // Destroy the Tome on success as well (it has been consumed).
                    if let Err(e) = self.world.despawn(tome_id) {
                        log::error!("Failed to despawn studied tome: {}", e);
                    }
                }
                Err(e) => {
                    log::error!("Tome references unknown spell '{}': {}", spell_name, e);
                    self.log
                        .push("The Tome's script is nonsensical.".to_string());
                }
            }
        } else {
            self.log
                .push("You failed to understand the Tome, and it crumbles.".to_string());
            if let Err(e) = self.world.despawn(tome_id) {
                log::error!("Failed to despawn failed tome: {}", e);
            }
        }

        self.state = RunState::AwaitingInput;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    fn setup() -> App {
        let mut app = App::new_test(42);
        app.world = World::new();
        app
    }

    fn attrs_cha(cha: i32) -> Attributes {
        Attributes {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: cha,
        }
    }

    #[test]
    fn test_decline_study_cancels() {
        let mut app = setup();
        app.world.spawn((Player, attrs_cha(10)));
        let tome = app.world.spawn((
            Item,
            Tome {
                spell_name: "Firebolt".into(),
                color: ManaColor::Orange,
                level: 1,
            },
        ));
        app.begin_study_tome(tome);
        assert_eq!(app.state, RunState::ShowStudyTome);
        app.handle_study_tome_input(Action::Decline);
        assert_eq!(app.state, RunState::AwaitingInput);
        assert!(app.world.get::<&Tome>(tome).is_ok());
    }

    #[test]
    fn test_failed_study_destroys_tome() {
        let mut app = setup();
        // CHA -5 (score 1) plus dc 5+2*10=25 makes success near-impossible.
        app.world.spawn((Player, attrs_cha(1)));
        let tome = app.world.spawn((
            Item,
            Tome {
                spell_name: "SomeSpell".into(),
                color: ManaColor::Orange,
                level: 10,
            },
        ));
        app.begin_study_tome(tome);
        app.handle_study_tome_input(Action::Confirm);
        assert!(app.world.get::<&Tome>(tome).is_err());
        assert_eq!(app.state, RunState::AwaitingInput);
    }
}
