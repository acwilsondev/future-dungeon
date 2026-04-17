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
                let passable = self.map.idx(new_x, new_y)
                    .map(|i| !self.map.blocked[i])
                    .unwrap_or(false);
                if !occupied_positions.contains(&(new_x, new_y)) && passable
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
                let res = self.resolve_attack(id, target_id, None, 0, false);
                let (tx, ty) = if let Ok(pos) = self.world.get::<&Position>(target_id) {
                    (pos.x, pos.y)
                } else {
                    (0, 0)
                };

                self.apply_attack_result(target_id, &res, tx, ty);

                if target_id == player_id {
                    let target_hp = self
                        .world
                        .get::<&CombatStats>(target_id)
                        .map(|s| s.hp)
                        .unwrap_or(0);
                    if target_hp <= 0 {
                        self.log.push("You are dead!".to_string());
                        self.state = RunState::Dead;
                        self.death = true;
                    }
                } else {
                    let target_hp = self
                        .world
                        .get::<&CombatStats>(target_id)
                        .map(|s| s.hp)
                        .unwrap_or(0);
                    if target_hp <= 0 {
                        self.log.push(format!("{} dies!", res.target_name));
                        // Monsters killing monsters? Despawn?
                        if let Err(e) = self.world.despawn(target_id) {
                            log::error!("Failed to despawn monster killed by monster: {}", e);
                        }
                        self.update_blocked_and_opaque();
                    }
                }
            }
            MonsterAction::RangedAttack(target_id) => {
                let (tx, ty, disadvantage) = {
                    let Ok(m_pos) = self.world.get::<&Position>(id) else {
                        return;
                    };
                    let Ok(t_pos) = self.world.get::<&Position>(target_id) else {
                        return;
                    };
                    let Ok(rw) = self.world.get::<&RangedWeapon>(id) else {
                        return;
                    };

                    let dist = (((m_pos.x as f32 - t_pos.x as f32).powi(2)
                        + (m_pos.y as f32 - t_pos.y as f32).powi(2))
                    .sqrt()) as i32;
                    let d = if dist > rw.range {
                        ((dist - rw.range) / rw.range_increment) as u32 + 1
                    } else {
                        0
                    };
                    (t_pos.x, t_pos.y, d)
                };

                let res = self.resolve_attack(id, target_id, Some(id), disadvantage, true);
                self.apply_attack_result(target_id, &res, tx, ty);

                if target_id == player_id {
                    let target_hp = self
                        .world
                        .get::<&CombatStats>(target_id)
                        .map(|s| s.hp)
                        .unwrap_or(0);
                    if target_hp <= 0 {
                        self.log.push("You are dead!".to_string());
                        self.state = RunState::Dead;
                        self.death = true;
                    }
                } else {
                    let target_hp = self
                        .world
                        .get::<&CombatStats>(target_id)
                        .map(|s| s.hp)
                        .unwrap_or(0);
                    if target_hp <= 0 {
                        self.log.push(format!("{} dies!", res.target_name));
                        if let Err(e) = self.world.despawn(target_id) {
                            log::error!("Failed to despawn monster killed by monster: {}", e);
                        }
                        self.update_blocked_and_opaque();
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
    fn test_monster_move_at_map_edge_does_not_panic() {
        let mut app = setup_test_app();
        // Monster at the bottom edge; moving down would produce y=50 on a 50-row map,
        // giving idx = 50*80 + x = 4000+ which is out of bounds without a guard.
        let monster = app.world.spawn((
            Monster,
            Position { x: 40, y: 49 },
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 1 },
        ));
        let player = app.world.spawn((Player, Position { x: 40, y: 0 }));
        let mut occupied = HashSet::new();
        occupied.insert((40, 49));

        // Should not panic; monster stays put since the destination is out of bounds.
        app.execute_monster_action(monster, MonsterAction::Move(0, 1), player, &mut occupied);
        let pos = app.world.get::<&Position>(monster).unwrap();
        assert_eq!(pos.y, 49);
    }

    #[test]
    fn test_monster_move_execution() {
        let mut app = setup_test_app();
        let monster = app.world.spawn((
            Monster,
            Position { x: 10, y: 10 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
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
            Attributes {
                strength: 10,
                dexterity: 1,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 20,
                max_hp: 20,
                defense: 2,
                power: 5,
            },
        ));
        let monster = app.world.spawn((
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
                power: 6,
            },
        ));
        let mut occupied = HashSet::new();

        app.execute_monster_action(
            monster,
            MonsterAction::Attack(player),
            player,
            &mut occupied,
        );

        let stats = app.world.get::<&CombatStats>(player).unwrap();
        assert!(stats.hp < 20);
        assert!(!app.log.is_empty());
    }

    #[test]
    fn test_monster_ranged_attack_execution() {
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
                hp: 100,
                max_hp: 100,
                defense: 0,
                power: 5,
            },
        ));
        let monster = app.world.spawn((
            Monster,
            Name("Archer".to_string()),
            Position { x: 15, y: 10 },
            Attributes {
                strength: 10,
                dexterity: 50,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            RangedWeapon {
                range: 8,
                range_increment: 12,
                damage_bonus: 4,
            },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));
        let mut occupied = HashSet::new();

        app.execute_monster_action(
            monster,
            MonsterAction::RangedAttack(player),
            player,
            &mut occupied,
        );

        let stats = app.world.get::<&CombatStats>(player).unwrap();
        assert!(stats.hp < 100);
        assert!(!app.effects.is_empty()); // Projectile effect
    }

    #[test]
    fn test_monster_attacks_monster() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let monster1 = app.world.spawn((
            Monster,
            Name("Orc1".to_string()),
            Attributes {
                strength: 50,
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
                power: 5,
            },
        ));
        let monster2 = app.world.spawn((
            Monster,
            Name("Orc2".to_string()),
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
        ));
        let mut occupied = HashSet::new();

        app.execute_monster_action(
            monster1,
            MonsterAction::Attack(monster2),
            player,
            &mut occupied,
        );

        let stats2 = app.world.get::<&CombatStats>(monster2).unwrap();
        assert!(stats2.hp < 50);
        assert!(app.log.last().unwrap().contains("Orc1 hits Orc2"));
    }

    #[test]
    fn test_monster_kills_player() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Attributes {
                strength: 10,
                dexterity: 1,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 2,
                max_hp: 20,
                defense: 0,
                power: 5,
            },
        ));
        let monster = app.world.spawn((
            Monster,
            Name("Orc".to_string()),
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
                power: 20, // Enough to kill in one hit
            },
        ));
        let mut occupied = HashSet::new();

        app.execute_monster_action(
            monster,
            MonsterAction::Attack(player),
            player,
            &mut occupied,
        );

        assert_eq!(app.state, RunState::Dead);
        assert!(app.death);
    }

    #[test]
    fn test_monster_inflicts_poison_with_save() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Name("Player".to_string()),
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 50,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 20,
                max_hp: 20,
                defense: 0,
                power: 5,
            },
        ));
        let monster = app.world.spawn((
            Monster,
            Name("Spider".to_string()),
            Attributes {
                strength: 50,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            Poison {
                damage: 2,
                turns: 5,
            },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));
        let mut occupied = HashSet::new();

        app.execute_monster_action(
            monster,
            MonsterAction::Attack(player),
            player,
            &mut occupied,
        );

        let has_poison = app.world.get::<&Poison>(player).is_ok();
        assert!(!has_poison);
        assert!(app.log.iter().any(|l| l.contains("resists the poison")));

        app.world
            .insert_one(
                player,
                Attributes {
                    strength: 10,
                    dexterity: 10,
                    constitution: -100,
                    intelligence: 10,
                    wisdom: 10,
                    charisma: 10,
                },
            )
            .unwrap();
        app.execute_monster_action(
            monster,
            MonsterAction::Attack(player),
            player,
            &mut occupied,
        );
        assert!(app.world.get::<&Poison>(player).is_ok());
    }
}
