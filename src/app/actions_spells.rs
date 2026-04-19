use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;

impl App {
    /// Number of spells in the player's spellbook.
    pub fn player_spell_count(&self) -> usize {
        let Some(player_id) = self.get_player_id() else {
            return 0;
        };
        self.world
            .get::<&Spellbook>(player_id)
            .map(|b| b.spells.len())
            .unwrap_or(0)
    }

    fn cloned_player_spell(&self, index: usize) -> Option<Spell> {
        let player_id = self.get_player_id()?;
        let book = self.world.get::<&Spellbook>(player_id).ok()?;
        book.spells.get(index).cloned()
    }

    pub fn handle_spells_input(&mut self, action: Action) {
        match action {
            Action::CloseMenu | Action::OpenSpells => self.state = RunState::AwaitingInput,
            Action::MenuUp if self.spell_cursor > 0 => {
                self.spell_cursor -= 1;
            }
            Action::MenuDown => {
                let count = self.player_spell_count();
                if count > 0 && self.spell_cursor < count - 1 {
                    self.spell_cursor += 1;
                }
            }
            Action::MenuSelect => {
                let Some(spell) = self.cloned_player_spell(self.spell_cursor) else {
                    return;
                };
                self.begin_cast(spell);
            }
            _ => {}
        }
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

    #[test]
    fn test_spells_menu_navigation() {
        let mut app = setup();
        let mut book = Spellbook::default();
        book.spells.push(Spell {
            title: "A".into(),
            description: "".into(),
            mana_cost: ManaCost {
                orange: 1,
                purple: 0,
            },
            level: 1,
            targeting: TargetSpec {
                range: None,
                selection: TargetSelection::SelfCast,
            },
            instructions: vec![],
        });
        book.spells.push(Spell {
            title: "B".into(),
            description: "".into(),
            mana_cost: ManaCost {
                orange: 1,
                purple: 0,
            },
            level: 1,
            targeting: TargetSpec {
                range: None,
                selection: TargetSelection::SelfCast,
            },
            instructions: vec![],
        });
        app.world.spawn((Player, Position { x: 0, y: 0 }, book));

        app.handle_spells_input(Action::MenuDown);
        assert_eq!(app.spell_cursor, 1);
        app.handle_spells_input(Action::MenuUp);
        assert_eq!(app.spell_cursor, 0);
        app.handle_spells_input(Action::CloseMenu);
        assert_eq!(app.state, RunState::AwaitingInput);
    }
}
