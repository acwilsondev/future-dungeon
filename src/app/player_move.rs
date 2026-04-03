use crate::app::{App, RunState, VisualEffect};
use crate::components::*;
use ratatui::prelude::Color;

impl App {
    pub fn move_player(&mut self, dx: i16, dy: i16) {
        let (new_x, new_y, player_power) = {
            let (power, _) = self.get_player_stats();
            let mut player_query = self.world.query::<(&Position, &Player)>();
            let (_, (pos, _)) = player_query.iter().next().expect("Player not found");
            (
                (pos.x as i16 + dx).max(0) as u16,
                (pos.y as i16 + dy).max(0) as u16,
                power,
            )
        };

        let mut target_interactable = None;
        for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
            if pos.x == new_x && pos.y == new_y {
                target_interactable = Some(id);
                break;
            }
        }
        if target_interactable.is_none() {
            for (id, (pos, _)) in self.world.query::<(&Position, &Merchant)>().iter() {
                if pos.x == new_x && pos.y == new_y {
                    target_interactable = Some(id);
                    break;
                }
            }
        }
        if target_interactable.is_none() {
            for (id, (pos, _)) in self.world.query::<(&Position, &AlchemyStation)>().iter() {
                if pos.x == new_x && pos.y == new_y {
                    target_interactable = Some(id);
                    break;
                }
            }
        }

        if let Some(target_id) = target_interactable {
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
                        x: new_x,
                        y: new_y,
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
                self.generate_noise(new_x, new_y, 8.0); // Combat is loud
            }

            if !monster_died && monster_damaged {
                self.world
                    .insert_one(target_id, LastHitByPlayer)
                    .expect("Failed to insert LastHitByPlayer");
                self.world
                    .insert_one(target_id, AlertState::Aggressive)
                    .expect("Failed to alert monster");
            }
            if monster_died {
                self.log.push(format!("{} dies!", monster_name));
                self.world
                    .despawn(target_id)
                    .expect("Failed to despawn monster");
                self.monsters_killed += 1;
                self.add_player_xp(xp_reward);
                self.update_blocked_and_opaque();
            }
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
            let player_id = {
                let mut player_query = self.world.query::<(&mut Position, &Player)>();
                let (player_id, (pos, _)) = player_query.iter().next().expect("Player not found");
                pos.x = new_x;
                pos.y = new_y;
                player_id
            };
            self.generate_noise(new_x, new_y, 3.0); // Moving is quiet but not silent

            // Gold pickup - ensure we don't pick up the player!
            let mut gold_to_pick = Vec::new();
            for (id, (g_pos, gold)) in self.world.query::<(&Position, &Gold)>().iter() {
                if id != player_id && g_pos.x == new_x && g_pos.y == new_y {
                    gold_to_pick.push((id, gold.amount));
                }
            }

            for (id, amount) in gold_to_pick {
                if let Ok(mut player_gold) = self.world.get::<&mut Gold>(player_id) {
                    player_gold.amount += amount;
                    self.log.push(format!("You pick up {} gold.", amount));
                }
                self.world.despawn(id).expect("Failed to despawn gold");
            }

            let mut total_damage = 0;
            let mut triggered_traps = Vec::new();
            let mut poisons_to_apply = Vec::new();
            for (id, (t_pos, trap)) in self.world.query::<(&Position, &mut Trap)>().iter() {
                if t_pos.x == new_x && t_pos.y == new_y {
                    let mut levitating = false;
                    for (eq_id, (eq, backpack)) in
                        self.world.query::<(&Equipped, &InBackpack)>().iter()
                    {
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
                            self.log
                                .push("You levitate safely over a trap!".to_string());
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
                let mut stats_query = self.world.query::<(&mut CombatStats, &Player)>();
                if let Some((_, (player_stats, _))) = stats_query.iter().next() {
                    player_stats.hp -= total_damage;
                    if player_stats.hp <= 0 {
                        self.death = true;
                        self.state = RunState::Dead;
                    }
                }
                drop(stats_query);
            }
            for trap_id in triggered_traps {
                self.world.despawn(trap_id).expect("Failed to despawn trap");
            }

            for poison in poisons_to_apply {
                self.world.insert_one(player_id, poison).ok();
                self.log
                    .push("You step on a Poison Spore and are poisoned!".to_string());
            }

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
}

