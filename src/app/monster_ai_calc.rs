use crate::app::{App, MonsterAction};
use crate::components::*;
use bracket_pathfinding::prelude::*;
use rand::Rng;

impl App {
    pub fn calculate_monster_action(
        &mut self,
        id: hecs::Entity,
        player_id: hecs::Entity,
    ) -> Option<MonsterAction> {
        let (pos, faction, personality, stats, viewshed, alert) = {
            if let (Ok(p), Ok(f), Ok(pers), Ok(s), Ok(v), Ok(a)) = (
                self.world.get::<&Position>(id),
                self.world.get::<&Faction>(id),
                self.world.get::<&AIPersonality>(id),
                self.world.get::<&CombatStats>(id),
                self.world.get::<&Viewshed>(id),
                self.world.get::<&AlertState>(id),
            ) {
                (*p, *f, *pers, *s, v.visible_tiles, *a)
            } else {
                return None;
            }
        };

        if self.world.get::<&Confusion>(id).is_ok() {
            let mut rng = rand::thread_rng();
            return Some(MonsterAction::Move(
                rng.gen_range(-1..=1),
                rng.gen_range(-1..=1),
            ));
        }

        if alert == AlertState::Sleeping {
            return None;
        }

        let is_merchant = self.world.get::<&Merchant>(id).is_ok();
        let mut target = None;
        let mut min_dist = viewshed as f32 + 1.0;

        if alert == AlertState::Aggressive {
            // Check player
            if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                if let Ok(p_faction) = self.world.get::<&Faction>(player_id) {
                    if faction.0 != p_faction.0 {
                        let dist = ((pos.x as f32 - p_pos.x as f32).powi(2)
                            + (pos.y as f32 - p_pos.y as f32).powi(2))
                        .sqrt();
                        if dist <= viewshed as f32 {
                            min_dist = dist;
                            target = Some((player_id, *p_pos));
                        }
                    }
                }
            }

            // Check other monsters
            for (other_id, (other_pos, other_faction)) in
                self.world.query::<(&Position, &Faction)>().iter()
            {
                if id == other_id {
                    continue;
                }
                if self.world.get::<&Wisp>(other_id).is_ok() {
                    continue;
                }
                if faction.0 != other_faction.0 {
                    let dist = ((pos.x as f32 - other_pos.x as f32).powi(2)
                        + (pos.y as f32 - other_pos.y as f32).powi(2))
                    .sqrt();
                    if dist <= viewshed as f32 && dist < min_dist {
                        min_dist = dist;
                        target = Some((other_id, *other_pos));
                    }
                }
            }
        } else if let AlertState::Curious { x, y } = alert {
            let dist =
                ((pos.x as f32 - x as f32).powi(2) + (pos.y as f32 - y as f32).powi(2)).sqrt();
            if dist < 1.5 {
                self.world
                    .insert_one(id, AlertState::Sleeping)
                    .expect("Failed to update AlertState");
                return None;
            } else {
                target = Some((id, Position { x, y }));
            }
        }

        if let Some((target_id, target_pos)) = target {
            let mut move_vec = None;
            let mut attack = false;

            if (personality.0 == Personality::Cowardly
                && stats.hp < stats.max_hp / 2
                && !is_merchant)
                || (personality.0 == Personality::Tactical && min_dist < 4.0 && !is_merchant)
            {
                let mut dx = 0;
                let mut dy = 0;
                if pos.x < target_pos.x {
                    dx = -1;
                } else if pos.x > target_pos.x {
                    dx = 1;
                }
                if pos.y < target_pos.y {
                    dy = -1;
                } else if pos.y > target_pos.y {
                    dy = 1;
                }
                move_vec = Some((dx, dy));
            } else if min_dist < 1.5 {
                attack = true;
            } else if !is_merchant {
                let mut dx = 0;
                let mut dy = 0;
                if pos.x < target_pos.x {
                    dx = 1;
                } else if pos.x > target_pos.x {
                    dx = -1;
                }
                if pos.y < target_pos.y {
                    dy = 1;
                } else if pos.y > target_pos.y {
                    dy = -1;
                }
                move_vec = Some((dx, dy));
            }

            if attack {
                return Some(MonsterAction::Attack(target_id));
            } else if let Some((dx, dy)) = move_vec {
                return Some(MonsterAction::Move(dx, dy));
            }

            // Ranged attack check
            if personality.0 == Personality::Tactical
                && min_dist > 1.5
                && min_dist < 8.0
                && self.world.get::<&RangedWeapon>(id).is_ok()
            {
                let line = line2d(
                    LineAlg::Bresenham,
                    Point::new(pos.x, pos.y),
                    Point::new(target_pos.x, target_pos.y),
                );
                let mut blocked = false;
                for p in line.iter().skip(1).take(line.len() - 2) {
                    let idx = (p.y as u16 * self.map.width + p.x as u16) as usize;
                    if self.map.blocked[idx] {
                        blocked = true;
                        break;
                    }
                }
                if !blocked {
                    return Some(MonsterAction::RangedAttack(target_id));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    fn setup_test_app() -> App {
        let mut app = App::new_random();
        app.world = World::new();
        app.map = crate::map::Map::new(80, 50);
        app
    }

    #[test]
    fn test_monster_attacks_when_adjacent() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Position { x: 10, y: 10 },
            Player,
            Faction(FactionKind::Player),
        ));
        let monster = app.world.spawn((
            Position { x: 11, y: 10 },
            Monster,
            Faction(FactionKind::Orcs),
            AIPersonality(Personality::Brave),
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
            Viewshed { visible_tiles: 8 },
            AlertState::Aggressive,
        ));

        let action = app.calculate_monster_action(monster, player);
        assert_eq!(action, Some(MonsterAction::Attack(player)));
    }

    #[test]
    fn test_monster_moves_towards_player() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Position { x: 10, y: 10 },
            Player,
            Faction(FactionKind::Player),
        ));
        let monster = app.world.spawn((
            Position { x: 12, y: 10 },
            Monster,
            Faction(FactionKind::Orcs),
            AIPersonality(Personality::Brave),
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
            Viewshed { visible_tiles: 8 },
            AlertState::Aggressive,
        ));

        let action = app.calculate_monster_action(monster, player);
        // Should move from 12,10 towards 10,10 -> dx = -1
        assert_eq!(action, Some(MonsterAction::Move(-1, 0)));
    }
}

