use crate::app::{App, RunState, MonsterAction, VisualEffect};
use crate::components::*;
use bracket_pathfinding::prelude::*;
use ratatui::prelude::Color;

impl App {
    pub fn execute_monster_action(&mut self, id: hecs::Entity, action: MonsterAction, player_id: hecs::Entity, occupied_positions: &mut std::collections::HashSet<(u16, u16)>) {
        match action {
            MonsterAction::Move(dx, dy) => {
                let (new_x, new_y) = { 
                    if let Ok(pos) = self.world.get::<&Position>(id) {
                        ((pos.x as i16 + dx).max(0) as u16, (pos.y as i16 + dy).max(0) as u16)
                    } else { return; }
                };
                if !occupied_positions.contains(&(new_x, new_y)) && !self.map.blocked[(new_y as u16 * self.map.width + new_x as u16) as usize] {
                    if let Ok(mut pos) = self.world.get::<&mut Position>(id) {
                        occupied_positions.remove(&(pos.x, pos.y));
                        pos.x = new_x; pos.y = new_y;
                        occupied_positions.insert((new_x, new_y));
                    }
                }
            }
            MonsterAction::Attack(target_id) => {
                let (monster_name, monster_power) = {
                    let stats = self.world.get::<&CombatStats>(id).expect("Monster has no stats");
                    let name = self.world.get::<&Name>(id).expect("Monster has no name");
                    (name.0.clone(), stats.power)
                };
                let target_name = self.world.get::<&Name>(target_id).map(|n| n.0.clone()).unwrap_or("Something".to_string());

                let target_defense = if self.world.get::<&Player>(target_id).is_ok() {
                    let (_, def) = self.get_player_stats();
                    def
                } else {
                    self.world.get::<&CombatStats>(target_id).map(|s| s.defense).unwrap_or(0)
                };

                let damage = (monster_power - target_defense).max(0);
                let target_hp = {
                    if let Ok(mut target_stats) = self.world.get::<&mut CombatStats>(target_id) {
                        target_stats.hp -= damage;
                        target_stats.hp
                    } else { 0 }
                };
                
                if target_id == player_id {
                    self.log.push(format!("{} hits you for {} damage!", monster_name, damage));
                    if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                        self.effects.push(VisualEffect::Flash { x: p_pos.x, y: p_pos.y, glyph: '!', fg: Color::Red, bg: Some(Color::Indexed(232)), duration: 5 });
                    }
                    if target_hp <= 0 { self.log.push("You are dead!".to_string()); self.state = RunState::Dead; self.death = true; }
                } else {
                    self.log.push(format!("{} hits {} for {} damage!", monster_name, target_name, damage));
                    if let Ok(t_pos) = self.world.get::<&Position>(target_id) {
                        self.effects.push(VisualEffect::Flash { x: t_pos.x, y: t_pos.y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
                    }
                    self.world.remove_one::<LastHitByPlayer>(target_id).ok();
                    if target_hp <= 0 {
                        self.log.push(format!("{} dies!", target_name));
                    }
                }
            }
            MonsterAction::RangedAttack(target_id) => {
                let (monster_name, rw) = {
                    let name = self.world.get::<&Name>(id).expect("Monster has no name");
                    let r = self.world.get::<&RangedWeapon>(id).expect("Monster has no ranged weapon");
                    (name.0.clone(), *r)
                };
                let target_name = self.world.get::<&Name>(target_id).map(|n| n.0.clone()).unwrap_or("Something".to_string());
                
                let target_defense = if self.world.get::<&Player>(target_id).is_ok() {
                    let (_, def) = self.get_player_stats();
                    def
                } else {
                    self.world.get::<&CombatStats>(target_id).map(|s| s.defense).unwrap_or(0)
                };

                let damage = (rw.damage_bonus - target_defense).max(0);

                let target_hp = {
                    if let Ok(mut target_stats) = self.world.get::<&mut CombatStats>(target_id) {
                        target_stats.hp -= damage;
                        target_stats.hp
                    } else { 0 }
                };

                if target_id == player_id {
                    self.log.push(format!("{} fires at you for {} damage!", monster_name, damage));
                    if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                        self.effects.push(VisualEffect::Flash { x: p_pos.x, y: p_pos.y, glyph: '!', fg: Color::Red, bg: Some(Color::Indexed(232)), duration: 5 });
                    }
                    if target_hp <= 0 { self.log.push("You are dead!".to_string()); self.state = RunState::Dead; self.death = true; }
                } else {
                    self.log.push(format!("{} fires at {} for {} damage!", monster_name, target_name, damage));
                    if let Ok(t_pos) = self.world.get::<&Position>(target_id) {
                        self.effects.push(VisualEffect::Flash { x: t_pos.x, y: t_pos.y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
                    }
                    self.world.remove_one::<LastHitByPlayer>(target_id).ok();
                    if target_hp <= 0 {
                        self.log.push(format!("{} dies!", target_name));
                    }
                }
                
                // Add projectile animation
                if let (Ok(m_pos), Ok(t_pos)) = (self.world.get::<&Position>(id), self.world.get::<&Position>(target_id)) {
                    let line = line2d(LineAlg::Bresenham, Point::new(m_pos.x, m_pos.y), Point::new(t_pos.x, t_pos.y));
                    let path: Vec<(u16, u16)> = line.iter().map(|p| (p.x as u16, p.y as u16)).collect();
                    self.effects.push(VisualEffect::Projectile { path, glyph: '*', fg: Color::Cyan, frame: 0, speed: 2 });
                }
            }
        }
    }
}
