use crate::app::{App, Branch, RunState};
use crate::components::*;
use rand::Rng;

impl App {
    pub fn monster_turn(&mut self) {
        self.on_turn_tick();
        if self.state == RunState::Dead {
            return;
        }

        let Some(player_id) = self.get_player_id() else {
            log::error!("Player not found in monster_turn");
            return;
        };

        // Handle Speed status
        if self.world.get::<&Speed>(player_id).is_ok() && self.speed_toggle {
            self.speed_toggle = false;
            self.state = RunState::AwaitingInput;
            self.log
                .push("You move with supernatural speed!".to_string());
            return;
        }
        self.speed_toggle = true;

        let mut actions = Vec::new();
        let mut actors: Vec<hecs::Entity> = self
            .world
            .query::<&Monster>()
            .iter()
            .map(|(id, _)| id)
            .collect();
        for (id, _) in self.world.query::<&Merchant>().iter() {
            actors.push(id);
        }

        // 1. Plan actions
        for id in actors {
            self.process_boss_phases(id);
            self.update_monster_perception(id, player_id);
            if let Some(action) = self.calculate_monster_action(id, player_id) {
                actions.push((id, action));
            }
        }

        // 2. Execute actions
        let mut occupied_positions: std::collections::HashSet<(u16, u16)> = self
            .world
            .query::<(&Position, &Monster)>()
            .iter()
            .map(|(_, (p, _))| (p.x, p.y))
            .collect();
        for (_, (p, _)) in self.world.query::<(&Position, &Merchant)>().iter() {
            occupied_positions.insert((p.x, p.y));
        }
        if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
            occupied_positions.insert((p_pos.x, p_pos.y));
        }

        for (id, action) in actions {
            self.execute_monster_action(id, action, player_id, &mut occupied_positions);
        }

        // 3. Cleanup
        self.cleanup_dead_entities();

        // 4. Special Wisp movement (non-combat, random)
        let mut wisp_moves = Vec::new();
        for (id, _) in self.world.query::<&Wisp>().iter() {
            wisp_moves.push((
                id,
                self.rng.random_range(-1..=1),
                self.rng.random_range(-1..=1),
            ));
        }

        for (id, dx, dy) in wisp_moves {
            let (new_x, new_y) = {
                if let Ok(pos) = self.world.get::<&Position>(id) {
                    (
                        (pos.x as i16 + dx).clamp(0, self.map.width as i16 - 1) as u16,
                        (pos.y as i16 + dy).clamp(0, self.map.height as i16 - 1) as u16,
                    )
                } else {
                    continue;
                }
            };
            if !self.map.blocked[new_y as usize * self.map.width as usize + new_x as usize] {
                if let Ok(mut pos) = self.world.get::<&mut Position>(id) {
                    pos.x = new_x;
                    pos.y = new_y;
                }
            }
        }

        self.update_blocked_and_opaque();

        if self.state != RunState::Dead && self.state != RunState::LevelUp {
            if self.current_branch == Branch::Vaults && self.turn_count.is_multiple_of(2) {
                // In Vaults, player is slower, so monsters get a double turn occasionally
                self.monster_turn();
            } else {
                self.state = RunState::AwaitingInput;
            }
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
        app.map = crate::map::Map::new(80, 50);
        for t in app.map.tiles.iter_mut() {
            *t = crate::map::TileType::Floor;
        }
        app.update_blocked_and_opaque();
        app
    }

    #[test]
    fn test_monster_turn_basic() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 10, y: 10 },
            Attributes {
                strength: 10,
                dexterity: 1,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 50,
                max_hp: 50,
                defense: 0,
                power: 5,
            },
            Gold { amount: 0 },
            Faction(FactionKind::Player),
        ));
        let _monster = app.world.spawn((
            Monster,
            Name("Orc".to_string()),
            Position { x: 11, y: 10 },
            Attributes {
                strength: 30,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 3,
            },
            Viewshed { visible_tiles: 8 },
            AlertState::Aggressive,
            Faction(FactionKind::Orcs),
            AIPersonality(Personality::Brave),
        ));

        app.monster_turn();

        let player_stats = app.world.get::<&CombatStats>(player).unwrap();
        // Attacker power 3 + mod 10 = 13.
        // Target AV 0 + mod -5 = -5 (clamped? no, mod added).
        // Actually target_av logic: 0 + DEX mod of 1 (-5) = -5.
        // damage = (roll 1-4) + 10 + 3 - (-5) = roll + 18.
        // wait, roll 1d4 + 10 (STR) + 3 (Base) - (-5) (Target AV) = roll + 18.
        // Player should definitely take damage.
        assert!(player_stats.hp < 50);
        assert_eq!(app.state, RunState::AwaitingInput);
    }

    #[test]
    fn test_monster_turn_player_speed() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 10, y: 10 },
            CombatStats {
                hp: 20,
                max_hp: 20,
                defense: 0,
                power: 5,
            },
            Speed { turns: 5 },
            Gold { amount: 0 },
            Faction(FactionKind::Player),
        ));
        app.speed_toggle = true; // Initial state

        app.monster_turn();

        // Should return early due to speed
        let player_stats = app.world.get::<&CombatStats>(player).unwrap();
        assert_eq!(player_stats.hp, 20); // No damage
        assert_eq!(app.speed_toggle, false);
        assert_eq!(app.state, RunState::AwaitingInput);
    }

    #[test]
    fn test_monster_turn_vaults_double_turn() {
        let mut app = setup_test_app();
        app.current_branch = Branch::Vaults;
        app.turn_count = 1; // Will become 2 during on_turn_tick, triggering double turn

        let player = app.world.spawn((
            Player,
            Position { x: 10, y: 10 },
            Attributes {
                strength: 10,
                dexterity: 1,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 50,
                max_hp: 50,
                defense: 0,
                power: 5,
            },
            Gold { amount: 0 },
            Faction(FactionKind::Player),
        ));
        let _monster = app.world.spawn((
            Monster,
            Name("Orc".to_string()),
            Position { x: 11, y: 10 },
            Attributes {
                strength: 30,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 3,
            },
            Viewshed { visible_tiles: 8 },
            AlertState::Aggressive,
            Faction(FactionKind::Orcs),
            AIPersonality(Personality::Brave),
        ));

        app.monster_turn();

        let player_stats = app.world.get::<&CombatStats>(player).unwrap();
        // Two hits guaranteed by high STR and low DEX
        assert!(player_stats.hp < 50);
        assert_eq!(app.turn_count, 3); // 1 + 1 (first turn) + 1 (second turn)
    }

    #[test]
    fn test_wisp_movement() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let wisp = app.world.spawn((Wisp, Position { x: 10, y: 10 }));

        app.monster_turn();

        let pos = app.world.get::<&Position>(wisp).unwrap();
        // Wisp should have moved randomly (or stayed if RNG 0,0)
        assert!(pos.x >= 9 && pos.x <= 11);
        assert!(pos.y >= 9 && pos.y <= 11);
    }

    #[test]
    fn test_merchant_actor() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));
        let _merchant = app.world.spawn((
            Merchant,
            Position { x: 10, y: 10 },
            Name("Merchant".to_string()),
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
            Viewshed { visible_tiles: 8 },
            AlertState::Sleeping,
            Faction(FactionKind::Player),
            AIPersonality(Personality::Brave),
        ));

        app.monster_turn();
        // Merchant should be processed (but sleeping so no action)
        assert_eq!(app.state, RunState::AwaitingInput);
    }
}
