use crate::app::{App, RunState, VisualEffect};
use crate::components::*;
use ratatui::prelude::Color;

impl App {
    fn get_interactable_at(&self, x: u16, y: u16) -> Option<hecs::Entity> {
        for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
            if pos.x == x && pos.y == y {
                return Some(id);
            }
        }
        for (id, (pos, _)) in self.world.query::<(&Position, &Merchant)>().iter() {
            if pos.x == x && pos.y == y {
                return Some(id);
            }
        }
        for (id, (pos, _)) in self.world.query::<(&Position, &AlchemyStation)>().iter() {
            if pos.x == x && pos.y == y {
                return Some(id);
            }
        }
        None
    }

    fn handle_combat(&mut self, target_id: hecs::Entity, player_power: i32, x: u16, y: u16) {
        let mut monster_damaged = false;
        let mut monster_died = false;
        let monster_name = self
            .world
            .get::<&Name>(target_id)
            .map(|n| n.0.clone())
            .unwrap_or("Something".to_string());
        let mut xp_reward = 0;

        {
            if let Ok(mut monster_stats) = self.world.get::<&mut CombatStats>(target_id) {
                let mut damage = (player_power - monster_stats.defense).max(0);

                // Sneak Attack?
                if let Ok(alert) = self.world.get::<&AlertState>(target_id) {
                    if *alert != AlertState::Aggressive {
                        damage *= 2;
                        self.log.push(format!("Sneak Attack on {}!", monster_name));
                    }
                }

                monster_stats.hp -= damage;
                self.log
                    .push(format!("You hit {} for {} damage!", monster_name, damage));
                self.effects.push(VisualEffect::Flash {
                    x,
                    y,
                    glyph: '*',
                    fg: Color::Red,
                    bg: None,
                    duration: 5,
                });
                monster_damaged = true;

                if monster_stats.hp <= 0 {
                    monster_died = true;
                    if let Ok(exp) = self.world.get::<&Experience>(target_id) {
                        xp_reward = exp.xp_reward;
                    }
                }
            }
        }

        if monster_damaged {
            self.generate_noise(x, y, 8.0); // Combat is loud
        }

        if !monster_died && monster_damaged {
            let _ = self.world.insert_one(target_id, LastHitByPlayer);
            let _ = self.world.insert_one(target_id, AlertState::Aggressive);
        }

        if monster_died {
            self.log.push(format!("{} dies!", monster_name));
            if let Err(e) = self.world.despawn(target_id) {
                log::error!("Failed to despawn monster {:?}: {}", target_id, e);
            }
            self.monsters_killed += 1;
            self.add_player_xp(xp_reward);
            self.update_blocked_and_opaque();
        }
    }

    fn handle_gold_pickup(&mut self, player_id: hecs::Entity, x: u16, y: u16) {
        let mut gold_to_pick = Vec::new();
        for (id, (g_pos, gold)) in self.world.query::<(&Position, &Gold)>().iter() {
            if id != player_id && g_pos.x == x && g_pos.y == y {
                gold_to_pick.push((id, gold.amount));
            }
        }

        for (id, amount) in gold_to_pick {
            if let Ok(mut player_gold) = self.world.get::<&mut Gold>(player_id) {
                player_gold.amount += amount;
                self.log.push(format!("You pick up {} gold.", amount));
            }
            if let Err(e) = self.world.despawn(id) {
                log::error!("Failed to despawn gold entity {:?}: {}", id, e);
            }
        }
    }

    fn handle_traps(&mut self, player_id: hecs::Entity, x: u16, y: u16) {
        let mut total_damage = 0;
        let mut triggered_traps = Vec::new();
        let mut poisons_to_apply = Vec::new();

        for (id, (t_pos, trap)) in self.world.query::<(&Position, &mut Trap)>().iter() {
            if t_pos.x == x && t_pos.y == y {
                let mut levitating = false;
                for (eq_id, (eq, backpack)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
                    if backpack.owner == player_id && eq.slot == EquipmentSlot::Feet {
                        let name = self
                            .world
                            .get::<&Name>(eq_id)
                            .map(|n| n.0.clone())
                            .unwrap_or_default();
                        if name == "Boots of Levitation" {
                            levitating = true;
                            break;
                        }
                    }
                }

                if levitating {
                    if !trap.revealed {
                        trap.revealed = true;
                        self.log.push("You levitate safely over a trap!".to_string());
                    }
                } else {
                    triggered_traps.push(id);
                    total_damage += trap.damage;
                    trap.revealed = true;
                    if let Ok(poison) = self.world.get::<&Poison>(id) {
                        if self.world.get::<&Poison>(player_id).is_err() {
                            poisons_to_apply.push(*poison);
                        }
                    }
                }
            }
        }

        if total_damage > 0 {
            self.log
                .push(format!("A trap deals {} damage to you!", total_damage));
            if let Ok(mut player_stats) = self.world.get::<&mut CombatStats>(player_id) {
                player_stats.hp -= total_damage;
                if player_stats.hp <= 0 {
                    self.death = true;
                    self.state = RunState::Dead;
                }
            }
        }

        for trap_id in triggered_traps {
            if let Err(e) = self.world.despawn(trap_id) {
                log::error!("Failed to despawn trap entity {:?}: {}", trap_id, e);
            }
        }

        for poison in poisons_to_apply {
            self.world.insert_one(player_id, poison).ok();
            self.log
                .push("You step on a Poison Spore and are poisoned!".to_string());
        }
    }

    pub fn move_player(&mut self, dx: i16, dy: i16) {
        let (new_x, new_y, player_power) = {
            let (power, _) = self.get_player_stats();
            let Some(player_id) = self.get_player_id() else {
                return;
            };
            let Ok(pos) = self.world.get::<&Position>(player_id) else {
                return;
            };
            (
                (pos.x as i16 + dx).max(0) as u16,
                (pos.y as i16 + dy).max(0) as u16,
                power,
            )
        };

        if let Some(target_id) = self.get_interactable_at(new_x, new_y) {
            // Check if it's a Merchant
            if self.world.get::<&Merchant>(target_id).is_ok() {
                self.active_merchant = Some(target_id);
                self.state = RunState::ShowShop;
                self.shop_cursor = 0;
                self.log.push("You talk to the Merchant.".to_string());
                return;
            }

            // Check if it's an Alchemy Station
            if self.world.get::<&AlchemyStation>(target_id).is_ok() {
                self.state = RunState::ShowAlchemy;
                self.inventory_cursor = 0;
                self.alchemy_selection.clear();
                self.log
                    .push("You approach the Alchemy Station.".to_string());
                return;
            }

            self.handle_combat(target_id, player_power, new_x, new_y);
            if self.state != RunState::LevelUp {
                self.state = RunState::MonsterTurn;
            }
            return;
        }

        let mut target_door = None;
        for (id, (d_pos, door)) in self.world.query::<(&Position, &Door)>().iter() {
            if d_pos.x == new_x && d_pos.y == new_y && !door.open {
                target_door = Some(id);
                break;
            }
        }

        if let Some(door_id) = target_door {
            if let Ok(mut door) = self.world.get::<&mut Door>(door_id) {
                door.open = true;
            }
            if let Ok(mut render) = self.world.get::<&mut Renderable>(door_id) {
                render.glyph = '/';
            }
            self.log.push("You open the door.".to_string());
            self.generate_noise(new_x, new_y, 10.0); // Opening doors is very loud
            self.update_blocked_and_opaque();
            self.update_fov();
            self.state = RunState::MonsterTurn;
            return;
        }

        if !self.map.blocked[(new_y * self.map.width + new_x) as usize] {
            let Some(player_id) = self.get_player_id() else {
                return;
            };
            if let Ok(mut pos) = self.world.get::<&mut Position>(player_id) {
                pos.x = new_x;
                pos.y = new_y;
            }
            self.generate_noise(new_x, new_y, 3.0); // Moving is quiet but not silent

            self.handle_gold_pickup(player_id, new_x, new_y);
            self.handle_traps(player_id, new_x, new_y);

            self.update_fov();
            self.state = RunState::MonsterTurn;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{CombatStats, Player, Position};
    use crate::map::TileType;
    use hecs::World;

    fn setup_test_app() -> App {
        let mut app = App::new_random();
        app.world = World::new(); // Clear random entities
        app.map = crate::map::Map::new(80, 50);
        for t in app.map.tiles.iter_mut() {
            *t = TileType::Floor;
        }
        app.update_blocked_and_opaque();
        app
    }

    #[test]
    fn test_player_movement() {
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
        ));

        app.move_player(1, 0);

        let pos = app.world.get::<&Position>(player).unwrap();
        assert_eq!(pos.x, 11);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_player_collision_with_wall() {
        let mut app = setup_test_app();
        app.map.tiles[(10 * 80 + 11) as usize] = TileType::Wall;
        app.update_blocked_and_opaque();

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
        ));

        app.move_player(1, 0);

        let pos = app.world.get::<&Position>(player).unwrap();
        assert_eq!(pos.x, 10); // Should NOT have moved
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_player_attack_monster() {
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
        ));

        let monster = app.world.spawn((
            Position { x: 11, y: 10 },
            Monster,
            Name("Test Monster".to_string()),
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 2,
            },
        ));

        app.move_player(1, 0);

        let monster_stats = app.world.get::<&CombatStats>(monster).unwrap();
        assert_eq!(monster_stats.hp, 5); // 10hp - 5power = 5hp

        let player_pos = app.world.get::<&Position>(player).unwrap();
        assert_eq!(player_pos.x, 10); // Player should NOT move when attacking
    }

    #[test]
    fn test_player_opens_door() {
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
        ));

        let door = app.world.spawn((
            Position { x: 11, y: 10 },
            Door { open: false },
            Renderable {
                glyph: '+',
                fg: Color::White,
            },
        ));

        app.move_player(1, 0);

        let door_comp = app.world.get::<&Door>(door).unwrap();
        assert!(door_comp.open);
        let render = app.world.get::<&Renderable>(door).unwrap();
        assert_eq!(render.glyph, '/');

        let player_pos = app.world.get::<&Position>(player).unwrap();
        assert_eq!(player_pos.x, 10); // Player stays put when opening door
    }

    #[test]
    fn test_player_picks_up_gold() {
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
        ));

        app.world.spawn((Position { x: 11, y: 10 }, Gold { amount: 50 }));

        app.move_player(1, 0);

        let player_gold = app.world.get::<&Gold>(player).unwrap();
        assert_eq!(player_gold.amount, 50);

        // Gold entity should be despawned. Player exists, and movement generated noise.
        assert_eq!(app.world.len(), 2);
    }

    #[test]
    fn test_player_triggers_trap() {
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
        ));

        app.world.spawn((
            Position { x: 11, y: 10 },
            Trap {
                damage: 5,
                revealed: false,
            },
        ));

        app.move_player(1, 0);

        let player_stats = app.world.get::<&CombatStats>(player).unwrap();
        assert_eq!(player_stats.hp, 5);
    }

    #[test]
    fn test_player_interacts_with_merchant() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((
            Position { x: 10, y: 10 },
            Player,
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 5,
            },
            Gold { amount: 100 },
        ));

        let merchant = app.world.spawn((
            Position { x: 11, y: 10 },
            Merchant,
            Name("Merchant".to_string()),
        ));

        app.move_player(1, 0);

        assert_eq!(app.state, RunState::ShowShop);
        assert_eq!(app.active_merchant, Some(merchant));
    }

    #[test]
    fn test_player_interacts_with_alchemy_station() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((
            Position { x: 10, y: 10 },
            Player,
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 5,
            },
            Gold { amount: 0 },
        ));

        app.world.spawn((Position { x: 11, y: 10 }, AlchemyStation));

        app.move_player(1, 0);

        assert_eq!(app.state, RunState::ShowAlchemy);
    }

    #[test]
    fn test_sneak_attack() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((
            Position { x: 10, y: 10 },
            Player,
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 5,
            },
            Gold { amount: 0 },
        ));
        let monster = app.world.spawn((
            Position { x: 11, y: 10 },
            Monster,
            Name("Orc".to_string()),
            CombatStats {
                hp: 20,
                max_hp: 20,
                defense: 0,
                power: 1,
            },
            AlertState::Sleeping,
        ));

        app.move_player(1, 0);

        let stats = app.world.get::<&CombatStats>(monster).unwrap();
        assert_eq!(stats.hp, 10); // 20 - (5*2) = 10
        assert!(app.log.iter().any(|l| l.contains("Sneak Attack")));
    }
}
