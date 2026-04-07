use crate::app::{App, MonsterAction};
use crate::components::*;
use bracket_pathfinding::prelude::*;
use rand::Rng;

struct MonsterAIContext {
    id: hecs::Entity,
    pos: Position,
    target_id: hecs::Entity,
    target_pos: Position,
    dist: f32,
    personality: Personality,
    stats: CombatStats,
    is_merchant: bool,
}

impl App {
    fn find_ai_target(
        &self,
        id: hecs::Entity,
        player_id: hecs::Entity,
        faction: Faction,
        viewshed: u16,
        pos: Position,
        alert: AlertState,
    ) -> Option<(hecs::Entity, Position, f32)> {
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
                            target = Some((player_id, *p_pos, dist));
                        }
                    }
                }
            }

            // Check other monsters
            for (other_id, (other_pos, other_faction)) in
                self.world.query::<(&Position, &Faction)>().iter()
            {
                if id == other_id 
                    || self.world.get::<&Wisp>(other_id).is_ok() 
                    || self.world.get::<&AlchemyStation>(other_id).is_ok() 
                {
                    continue;
                }
                if faction.0 != other_faction.0 {
                    let dist = ((pos.x as f32 - other_pos.x as f32).powi(2)
                        + (pos.y as f32 - other_pos.y as f32).powi(2))
                    .sqrt();
                    if dist <= viewshed as f32 && dist < min_dist {
                        min_dist = dist;
                        target = Some((other_id, *other_pos, dist));
                    }
                }
            }
        } else if let AlertState::Curious { x, y } = alert {
            let dist =
                ((pos.x as f32 - x as f32).powi(2) + (pos.y as f32 - y as f32).powi(2)).sqrt();
            if dist >= 1.5 {
                target = Some((id, Position { x, y }, dist));
            }
        }
        target
    }

    fn decide_ai_action(
        &self,
        ctx: MonsterAIContext,
    ) -> Option<MonsterAction> {
        // Check for flee
        if (!ctx.is_merchant) && (
            (ctx.personality == Personality::Cowardly && ctx.stats.hp < ctx.stats.max_hp / 2)
            || (ctx.personality == Personality::Tactical && ctx.dist < 4.0)
        ) {
            let mut dx = 0;
            let mut dy = 0;
            if ctx.pos.x < ctx.target_pos.x { dx = -1; } else if ctx.pos.x > ctx.target_pos.x { dx = 1; }
            if ctx.pos.y < ctx.target_pos.y { dy = -1; } else if ctx.pos.y > ctx.target_pos.y { dy = 1; }
            return Some(MonsterAction::Move(dx, dy));
        }

        // Check for ranged attack
        if ctx.personality == Personality::Tactical
            && ctx.dist > 1.5
            && ctx.dist < 8.0
            && self.world.get::<&RangedWeapon>(ctx.id).is_ok()
        {
            let line = line2d(
                LineAlg::Bresenham,
                Point::new(ctx.pos.x, ctx.pos.y),
                Point::new(ctx.target_pos.x, ctx.target_pos.y),
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
                return Some(MonsterAction::RangedAttack(ctx.target_id));
            }
        }

        // Check for melee attack
        if ctx.dist < 1.5 {
            return Some(MonsterAction::Attack(ctx.target_id));
        }

        // Move towards target
        if !ctx.is_merchant {
            let mut dx = 0;
            let mut dy = 0;
            if ctx.pos.x < ctx.target_pos.x { dx = 1; } else if ctx.pos.x > ctx.target_pos.x { dx = -1; }
            if ctx.pos.y < ctx.target_pos.y { dy = 1; } else if ctx.pos.y > ctx.target_pos.y { dy = -1; }
            return Some(MonsterAction::Move(dx, dy));
        }

        None
    }

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
            return Some(MonsterAction::Move(
                self.rng.gen_range(-1..=1),
                self.rng.gen_range(-1..=1),
            ));
        }

        if alert == AlertState::Sleeping {
            return None;
        }

        let is_merchant = self.world.get::<&Merchant>(id).is_ok();
        
        let target = self.find_ai_target(id, player_id, faction, viewshed.try_into().unwrap_or(0), pos, alert);

        if let Some((target_id, target_pos, dist)) = target {
            let ctx = MonsterAIContext {
                id,
                pos,
                target_id,
                target_pos,
                dist,
                personality: personality.0,
                stats,
                is_merchant,
            };
            return self.decide_ai_action(ctx);
        } else if let AlertState::Curious { .. } = alert {
            // If we had a curious state but no target found by find_ai_target, it means we reached the spot
            let _ = self.world.insert_one(id, AlertState::Sleeping);
        }

        None
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
        app.map.populate_blocked_and_opaque();
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

    #[test]
    fn test_monster_confusion() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Position { x: 10, y: 10 },
            Player,
            Faction(FactionKind::Player),
        ));
        let monster = app.world.spawn((
            Position { x: 15, y: 15 },
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
            Confusion { turns: 5 },
        ));

        let action = app.calculate_monster_action(monster, player);
        if let Some(MonsterAction::Move(dx, dy)) = action {
            assert!(dx >= -1 && dx <= 1);
            assert!(dy >= -1 && dy <= 1);
        } else {
            panic!("Confused monster should move randomly");
        }
    }

    #[test]
    fn test_monster_sleeping() {
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
            AlertState::Sleeping,
        ));

        let action = app.calculate_monster_action(monster, player);
        assert_eq!(action, None);
    }

    #[test]
    fn test_monster_curious() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Position { x: 10, y: 10 },
            Player,
            Faction(FactionKind::Player),
        ));
        let monster = app.world.spawn((
            Position { x: 15, y: 15 },
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
            AlertState::Curious { x: 10, y: 10 },
        ));

        let action = app.calculate_monster_action(monster, player);
        // Should move towards 10,10 from 15,15 -> dx=-1, dy=-1
        assert_eq!(action, Some(MonsterAction::Move(-1, -1)));
    }

    #[test]
    fn test_monster_cowardly_fleeing() {
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
            AIPersonality(Personality::Cowardly),
            CombatStats {
                hp: 2,
                max_hp: 10,
                defense: 0,
                power: 1,
            }, // Low HP
            Viewshed { visible_tiles: 8 },
            AlertState::Aggressive,
        ));

        let action = app.calculate_monster_action(monster, player);
        // Should flee from 10,10 -> at 11,10 it should move dx=1
        assert_eq!(action, Some(MonsterAction::Move(1, 0)));
    }

    #[test]
    fn test_monster_ranged_attack() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Position { x: 10, y: 10 },
            Player,
            Faction(FactionKind::Player),
        ));
        let monster = app.world.spawn((
            Position { x: 15, y: 10 },
            Monster,
            Faction(FactionKind::Orcs),
            AIPersonality(Personality::Tactical),
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
            Viewshed { visible_tiles: 8 },
            AlertState::Aggressive,
            RangedWeapon {
                range: 8,
                range_increment: 12,
                damage_bonus: 2,
            },
        ));

        let action = app.calculate_monster_action(monster, player);
        assert_eq!(action, Some(MonsterAction::RangedAttack(player)));
    }
}
