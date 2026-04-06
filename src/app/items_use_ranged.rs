use crate::app::{App, RunState, VisualEffect};
use crate::components::*;
use bracket_pathfinding::prelude::*;
use ratatui::prelude::Color;

impl App {
    pub fn fire_targeting_item(&mut self) {
        if let Some(item_id) = self.targeting_item {
            let item_name = self.get_item_name(item_id);

            let (player_pos, player_id) = {
                let Some(id) = self.get_player_id() else {
                    return;
                };
                let Ok(pos) = self.world.get::<&Position>(id) else {
                    return;
                };
                (*pos, id)
            };

            let line = line2d(
                LineAlg::Bresenham,
                Point::new(player_pos.x, player_pos.y),
                Point::new(self.targeting_cursor.0, self.targeting_cursor.1),
            );

            let mut actual_target = self.targeting_cursor;
            for p in line.iter().skip(1) {
                let idx = (p.y as u16 * self.map.width + p.x as u16) as usize;
                if self.map.blocked[idx] {
                    actual_target = (p.x as u16, p.y as u16);
                    self.log.push(format!("The {} is blocked!", item_name));
                    break;
                }
            }

            // Add projectile animation
            let path: Vec<(u16, u16)> = line
                .iter()
                .take_while(|p| (p.x as u16, p.y as u16) != actual_target)
                .map(|p| (p.x as u16, p.y as u16))
                .chain(std::iter::once(actual_target))
                .collect();
            self.effects.push(VisualEffect::Projectile {
                path,
                glyph: '*',
                fg: Color::Yellow,
                frame: 0,
                speed: 1,
            });

            let mut targets = Vec::new();

            // Collect info before mutations
            let aoe_radius = self
                .world
                .get::<&AreaOfEffect>(item_id)
                .ok()
                .map(|a| a.radius);
            let confusion_turns = self.world.get::<&Confusion>(item_id).ok().map(|c| c.turns);
            let poison_effect = self.world.get::<&Poison>(item_id).ok().map(|p| *p);
            let mut power = self
                .world
                .get::<&CombatStats>(item_id)
                .map(|s| s.power)
                .unwrap_or(10);
            let is_ranged_weapon = self.world.get::<&RangedWeapon>(item_id).ok().map(|rw| *rw);

            if let Some(rw) = is_ranged_weapon {
                power = rw.damage_bonus;
                // Consume ammo
                let ammo_id = self
                    .world
                    .query::<(&Ammunition, &InBackpack)>()
                    .iter()
                    .filter(|(_, (_, backpack))| backpack.owner == player_id)
                    .map(|(id, _)| id)
                    .next();
                if let Some(aid) = ammo_id {
                    if let Err(e) = self.world.despawn(aid) {
                        log::error!("Failed to despawn ammunition: {}", e);
                    }
                }
            }

            if let Some(radius) = aoe_radius {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    let dist = ((pos.x as f32 - actual_target.0 as f32).powi(2)
                        + (pos.y as f32 - actual_target.1 as f32).powi(2))
                    .sqrt();
                    if dist <= radius as f32 {
                        targets.push(id);
                    }
                }
                self.log.push(format!("The {} explodes!", item_name));
                self.generate_noise(actual_target.0, actual_target.1, 15.0); // Explosions are very loud
                for target_id in targets {
                    if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target_id) {
                        stats.hp -= power;
                    }
                    if let Ok(t_pos) = self.world.get::<&Position>(target_id) {
                        self.effects.push(VisualEffect::Flash {
                            x: t_pos.x,
                            y: t_pos.y,
                            glyph: '*',
                            fg: Color::Indexed(208),
                            bg: None,
                            duration: 10,
                        });
                    }
                    let _ = self.world.insert_one(target_id, LastHitByPlayer);
                    let _ = self.world.insert_one(target_id, AlertState::Aggressive);
                }
            } else if let Some(turns) = confusion_turns {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 {
                        targets.push(id);
                    }
                }
                self.generate_noise(actual_target.0, actual_target.1, 4.0);
                for target_id in targets {
                    self.log
                        .push(format!("The monster is confused by the {}!", item_name));
                    let _ = self.world.insert_one(target_id, Confusion { turns });
                    let _ = self.world.insert_one(target_id, AlertState::Aggressive);
                }
            } else if let Some(poison) = poison_effect {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 {
                        targets.push(id);
                    }
                }
                self.generate_noise(actual_target.0, actual_target.1, 4.0);
                for target_id in targets {
                    self.log
                        .push(format!("The monster is poisoned by the {}!", item_name));
                    let _ = self.world.insert_one(target_id, poison);
                    let _ = self.world.insert_one(target_id, LastHitByPlayer);
                    let _ = self.world.insert_one(target_id, AlertState::Aggressive);
                }
            } else {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 {
                        targets.push(id);
                    }
                }
                self.generate_noise(actual_target.0, actual_target.1, 6.0);
                for target_id in targets {
                    if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target_id) {
                        stats.hp -= power;
                        self.log
                            .push(format!("The {} hits for {} damage!", item_name, power));
                    }
                    if let Ok(t_pos) = self.world.get::<&Position>(target_id) {
                        self.effects.push(VisualEffect::Flash {
                            x: t_pos.x,
                            y: t_pos.y,
                            glyph: '*',
                            fg: Color::Red,
                            bg: None,
                            duration: 5,
                        });
                    }
                    let _ = self.world.insert_one(target_id, LastHitByPlayer);
                    let _ = self.world.insert_one(target_id, AlertState::Aggressive);
                }
            }

            let mut to_despawn = Vec::new();
            let mut total_xp: i32 = 0;
            for (id, (stats, _)) in self.world.query::<(&CombatStats, &Monster)>().iter() {
                if stats.hp <= 0 {
                    to_despawn.push(id);
                    if let Ok(exp) = self.world.get::<&Experience>(id) {
                        total_xp = total_xp.saturating_add(exp.xp_reward);
                    }
                }
            }
            for id in to_despawn {
                if let Err(e) = self.world.despawn(id) {
                    log::error!("Failed to despawn monster: {}", e);
                }
                self.monsters_killed += 1;
            }
            self.update_blocked_and_opaque();

            if total_xp > 0 {
                self.add_player_xp(total_xp);
            }

            self.identify_item(item_id);

            if is_ranged_weapon.is_none() {
                if let Err(e) = self.world.despawn(item_id) {
                    log::error!("Failed to despawn consumable item after use: {}", e);
                }
            }
            if self.state != RunState::LevelUp {
                self.state = RunState::MonsterTurn;
            }
            self.targeting_item = None;
        }
    }
}
