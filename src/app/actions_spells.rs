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

    fn setup_map() -> App {
        let mut app = setup();
        app.map = crate::map::Map::new(80, 50);
        for t in app.map.tiles.iter_mut() {
            *t = crate::map::TileType::Floor;
        }
        app.update_blocked_and_opaque();
        app
    }

    fn default_attrs() -> Attributes {
        Attributes {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }

    #[test]
    fn test_full_cast_cycle_end_to_end() {
        let mut app = setup_map();
        let mut book = Spellbook::default();
        book.spells.push(Spell {
            title: "Firebolt".into(),
            description: "".into(),
            mana_cost: ManaCost {
                orange: 1,
                purple: 0,
            },
            level: 1,
            targeting: TargetSpec {
                range: Some(10),
                selection: TargetSelection::Entity,
            },
            instructions: vec![EffectInstruction {
                opcode: EffectOpCode::DealDamage,
                shape: EffectShape::Point,
                radius: None,
                application_save: None,
                magnitude: Some(Dice::flat(7)),
                metadata: EffectMetadata::Damage(DamageType::Fire),
            }],
        });
        let player = app.world.spawn((
            Player,
            Position { x: 5, y: 5 },
            default_attrs(),
            book,
            ManaPool {
                current_orange: 2,
                max_orange: 2,
                current_purple: 0,
                max_purple: 0,
            },
        ));
        let monster = app.world.spawn((
            Monster,
            Position { x: 7, y: 5 },
            Name("Rat".into()),
            default_attrs(),
            CombatStats {
                hp: 20,
                max_hp: 20,
                defense: 0,
                power: 1,
            },
            Experience {
                level: 1,
                xp: 0,
                next_level_xp: 0,
                xp_reward: 0,
            },
        ));

        app.state = RunState::ShowSpells;
        app.spell_cursor = 0;
        app.handle_spells_input(Action::MenuSelect);
        assert_eq!(app.state, RunState::ShowTargeting);
        assert!(app.casting_spell.is_some());

        app.targeting_cursor = (7, 5);
        app.handle_targeting_input(Action::MenuSelect);

        assert_eq!(app.state, RunState::MonsterTurn);
        let stats = app.world.get::<&CombatStats>(monster).unwrap();
        assert_eq!(stats.hp, 13);
        let pool = app.world.get::<&ManaPool>(player).unwrap();
        assert_eq!(pool.current_orange, 1);
        assert!(app.casting_spell.is_none());
    }

    #[test]
    fn test_mana_drought_decays_across_turns() {
        let mut app = setup_map();
        let player = app.world.spawn((
            Player,
            Position { x: 5, y: 5 },
            default_attrs(),
            ManaPool {
                current_orange: 0,
                max_orange: 2,
                current_purple: 0,
                max_purple: 0,
            },
            ManaDrought { duration: 2 },
        ));

        app.tick_mana_regen();
        assert_eq!(app.world.get::<&ManaDrought>(player).unwrap().duration, 1);
        let pool = app.world.get::<&ManaPool>(player).unwrap();
        assert_eq!(pool.current_orange, 0);
        drop(pool);

        app.tick_mana_regen();
        assert!(app.world.get::<&ManaDrought>(player).is_err());
    }

    #[test]
    fn test_aoe_circle_hits_multiple_entities() {
        let mut app = setup_map();
        let _player = app.world.spawn((
            Player,
            Position { x: 5, y: 5 },
            default_attrs(),
            ManaPool {
                current_orange: 0,
                max_orange: 0,
                current_purple: 2,
                max_purple: 2,
            },
        ));
        let m1 = app.world.spawn((
            Monster,
            Position { x: 10, y: 10 },
            Name("A".into()),
            default_attrs(),
            CombatStats {
                hp: 20,
                max_hp: 20,
                defense: 0,
                power: 1,
            },
            Experience {
                level: 1,
                xp: 0,
                next_level_xp: 0,
                xp_reward: 0,
            },
        ));
        let m2 = app.world.spawn((
            Monster,
            Position { x: 11, y: 10 },
            Name("B".into()),
            default_attrs(),
            CombatStats {
                hp: 20,
                max_hp: 20,
                defense: 0,
                power: 1,
            },
            Experience {
                level: 1,
                xp: 0,
                next_level_xp: 0,
                xp_reward: 0,
            },
        ));

        let blast = Spell {
            title: "Blast".into(),
            description: "".into(),
            mana_cost: ManaCost {
                orange: 0,
                purple: 1,
            },
            level: 1,
            targeting: TargetSpec {
                range: Some(20),
                selection: TargetSelection::Location,
            },
            instructions: vec![EffectInstruction {
                opcode: EffectOpCode::DealDamage,
                shape: EffectShape::Circle,
                radius: Some(2),
                application_save: None,
                magnitude: Some(Dice::flat(5)),
                metadata: EffectMetadata::Damage(DamageType::Fire),
            }],
        };
        app.finish_cast(blast, (10, 10));
        assert_eq!(app.world.get::<&CombatStats>(m1).unwrap().hp, 15);
        assert_eq!(app.world.get::<&CombatStats>(m2).unwrap().hp, 15);
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
