use crate::app::{App, MonsterAction, RunState, VisualEffect};
use crate::components::*;
use bracket_pathfinding::prelude::*;
use ratatui::prelude::Color;

impl App {
    pub fn execute_monster_action(
        &mut self,
        id: hecs::Entity,
        action: MonsterAction,
        player_id: hecs::Entity,
        occupied_positions: &mut std::collections::HashSet<(u16, u16)>,
    ) {
        match action {
            MonsterAction::Move(dx, dy) => {
                let (new_x, new_y) = {
                    if let Ok(pos) = self.world.get::<&Position>(id) {
                        (
                            (pos.x as i16 + dx).max(0) as u16,
                            (pos.y as i16 + dy).max(0) as u16,
                        )
                    } else {
                        return;
                    }
                };
                if !occupied_positions.contains(&(new_x, new_y))
                    && !self.map.blocked[(new_y * self.map.width + new_x) as usize]
                {
                    if let Ok(mut pos) = self.world.get::<&mut Position>(id) {
                        occupied_positions.remove(&(pos.x, pos.y));
                        pos.x = new_x;
                        pos.y = new_y;
                        occupied_positions.insert((new_x, new_y));
                    }
                }
            }
            MonsterAction::Attack(target_id) => {
                let (monster_name, monster_power) = {
                    let Ok(stats) = self.world.get::<&CombatStats>(id) else {
                        return;
                    };
                    let Ok(name) = self.world.get::<&Name>(id) else {
                        return;
                    };
                    (name.0.clone(), stats.power)
                };
                let target_name = self
                    .world
                    .get::<&Name>(target_id)
                    .map(|n| n.0.clone())
                    .unwrap_or("Something".to_string());

                let target_defense = if self.world.get::<&Player>(target_id).is_ok() {
                    let (_, def) = self.get_player_stats();
                    def
                } else {
                    self.world
                        .get::<&CombatStats>(target_id)
                        .map(|s| s.defense)
                        .unwrap_or(0)
                };

                let damage = (monster_power - target_defense).max(0);
                let target_hp = {
                    if let Ok(mut target_stats) = self.world.get::<&mut CombatStats>(target_id) {
                        target_stats.hp -= damage;
                        target_stats.hp
                    } else {
                        0
                    }
                };

                if target_id == player_id {
                    self.log
                        .push(format!("{} hits you for {} damage!", monster_name, damage));
                    if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                        self.effects.push(VisualEffect::Flash {
                            x: p_pos.x,
                            y: p_pos.y,
                            glyph: '!',
                            fg: Color::Red,
                            bg: Some(Color::Indexed(232)),
                            duration: 5,
                        });
                    }
                    if target_hp <= 0 {
                        self.log.push("You are dead!".to_string());
                        self.state = RunState::Dead;
                        self.death = true;
                    }
                } else {
                    self.log.push(format!(
                        "{} hits {} for {} damage!",
                        monster_name, target_name, damage
                    ));
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
                    self.world.remove_one::<LastHitByPlayer>(target_id).ok();
                    if target_hp <= 0 {
                        self.log.push(format!("{} dies!", target_name));
                    }
                }
            }
            MonsterAction::RangedAttack(target_id) => {
                let (monster_name, rw) = {
                    let Ok(name) = self.world.get::<&Name>(id) else {
                        return;
                    };
                    let Ok(r) = self.world.get::<&RangedWeapon>(id) else {
                        return;
                    };
                    (name.0.clone(), *r)
                };
                let target_name = self
                    .world
                    .get::<&Name>(target_id)
                    .map(|n| n.0.clone())
                    .unwrap_or("Something".to_string());

                let target_defense = if self.world.get::<&Player>(target_id).is_ok() {
                    let (_, def) = self.get_player_stats();
                    def
                } else {
                    self.world
                        .get::<&CombatStats>(target_id)
                        .map(|s| s.defense)
                        .unwrap_or(0)
                };

                let damage = (rw.damage_bonus - target_defense).max(0);

                let target_hp = {
                    if let Ok(mut target_stats) = self.world.get::<&mut CombatStats>(target_id) {
                        target_stats.hp -= damage;
                        target_stats.hp
                    } else {
                        0
                    }
                };

                if target_id == player_id {
                    self.log.push(format!(
                        "{} fires at you for {} damage!",
                        monster_name, damage
                    ));
                    if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                        self.effects.push(VisualEffect::Flash {
                            x: p_pos.x,
                            y: p_pos.y,
                            glyph: '!',
                            fg: Color::Red,
                            bg: Some(Color::Indexed(232)),
                            duration: 5,
                        });
                    }
                    if target_hp <= 0 {
                        self.log.push("You are dead!".to_string());
                        self.state = RunState::Dead;
                        self.death = true;
                    }
                } else {
                    self.log.push(format!(
                        "{} fires at {} for {} damage!",
                        monster_name, target_name, damage
                    ));
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
                    self.world.remove_one::<LastHitByPlayer>(target_id).ok();
                    if target_hp <= 0 {
                        self.log.push(format!("{} dies!", target_name));
                    }
                }

                // Add projectile animation
                if let (Ok(m_pos), Ok(t_pos)) = (
                    self.world.get::<&Position>(id),
                    self.world.get::<&Position>(target_id),
                ) {
                    let line = line2d(
                        LineAlg::Bresenham,
                        Point::new(m_pos.x, m_pos.y),
                        Point::new(t_pos.x, t_pos.y),
                    );
                    let path: Vec<(u16, u16)> =
                        line.iter().map(|p| (p.x as u16, p.y as u16)).collect();
                    self.effects.push(VisualEffect::Projectile {
                        path,
                        glyph: '*',
                        fg: Color::Cyan,
                        frame: 0,
                        speed: 2,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;
    use std::collections::HashSet;

    fn setup_test_app() -> App {
        let mut app = App::new_random();
        app.world = World::new();
        app.map = crate::map::Map::new(80, 50);
        for t in app.map.tiles.iter_mut() {
            *t = crate::map::TileType::Floor;
        }
        app.update_blocked_and_opaque();
        app
    }

    #[test]
    fn test_monster_move_execution() {
        let mut app = setup_test_app();
        let monster = app.world.spawn((
            Monster,
            Position { x: 10, y: 10 },
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 1 }
        ));
        let player = app.world.spawn((Player, Position { x: 20, y: 20 }));
        let mut occupied = HashSet::new();
        occupied.insert((10, 10));

        app.execute_monster_action(monster, MonsterAction::Move(1, 0), player, &mut occupied);

        let pos = app.world.get::<&Position>(monster).unwrap();
        assert_eq!(pos.x, 11);
        assert_eq!(pos.y, 10);
        assert!(occupied.contains(&(11, 10)));
        assert!(!occupied.contains(&(10, 10)));
    }

    #[test]
    fn test_monster_attack_player_execution() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 10, y: 10 },
            CombatStats { hp: 20, max_hp: 20, defense: 2, power: 5 }
        ));
        let monster = app.world.spawn((
            Monster,
            Name("Orc".to_string()),
            Position { x: 11, y: 10 },
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 6 }
        ));
        let mut occupied = HashSet::new();

        app.execute_monster_action(monster, MonsterAction::Attack(player), player, &mut occupied);

        let stats = app.world.get::<&CombatStats>(player).unwrap();
        assert_eq!(stats.hp, 16); // 20 - (6-2) = 16
        assert!(!app.log.is_empty());
    }

    #[test]
    fn test_monster_ranged_attack_execution() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 10, y: 10 },
            CombatStats { hp: 20, max_hp: 20, defense: 0, power: 5 }
        ));
        let monster = app.world.spawn((
            Monster,
            Name("Archer".to_string()),
            Position { x: 15, y: 10 },
            RangedWeapon { range: 8, damage_bonus: 4 },
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 1 }
        ));
        let mut occupied = HashSet::new();

        app.execute_monster_action(monster, MonsterAction::RangedAttack(player), player, &mut occupied);

        let stats = app.world.get::<&CombatStats>(player).unwrap();
        assert_eq!(stats.hp, 16); // 20 - 4 = 16
        assert!(!app.effects.is_empty()); // Projectile effect
    }
}
