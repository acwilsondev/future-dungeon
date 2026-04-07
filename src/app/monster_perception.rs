use crate::app::App;
use crate::components::*;
use bracket_pathfinding::prelude::*;

impl App {
    pub fn update_monster_perception(&mut self, id: hecs::Entity, player_id: hecs::Entity) {
        let (pos, viewshed, mut current_alert) = {
            if let (Ok(p), Ok(v), Ok(a)) = (
                self.world.get::<&Position>(id),
                self.world.get::<&Viewshed>(id),
                self.world.get::<&AlertState>(id),
            ) {
                (*p, v.visible_tiles, *a)
            } else {
                return;
            }
        };

        if current_alert != AlertState::Aggressive {
            // Check for player visibility
            let mut can_see_player = false;
            if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                let dist = ((pos.x as f32 - p_pos.x as f32).powi(2)
                    + (pos.y as f32 - p_pos.y as f32).powi(2))
                .sqrt();
                if dist <= viewshed as f32 {
                    // Monster can only see player if player is lit or very close
                    let p_idx = (p_pos.y * self.map.width + p_pos.x) as usize;
                    if self.map.light[p_idx] > 0.2 || dist < 1.5 {
                        let line = line2d(
                            LineAlg::Bresenham,
                            Point::new(pos.x, pos.y),
                            Point::new(p_pos.x, p_pos.y),
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
                            can_see_player = true;
                        }
                    }
                }
            }

            if can_see_player {
                let _ = self.world.insert_one(id, AlertState::Aggressive);
            } else {
                // Check for noise
                let idx = (pos.y * self.map.width + pos.x) as usize;
                let sound_level = self.map.sound[idx];
                if sound_level > 1.0 {
                    let p_pos_data = self
                        .world
                        .get::<&Position>(player_id)
                        .ok()
                        .map(|p| (p.x, p.y));
                    if let Some((px, py)) = p_pos_data {
                        current_alert = AlertState::Curious { x: px, y: py };
                        let _ = self.world.insert_one(id, current_alert);
                    }
                }
            }
        }
    }

    pub fn cleanup_dead_entities(&mut self) {
        let mut to_despawn = Vec::new();
        let mut total_xp: i32 = 0;
        let mut drops = Vec::new();
        for (id, (stats, _)) in self.world.query::<(&CombatStats, &Monster)>().iter() {
            if stats.hp <= 0 {
                to_despawn.push(id);
                if self.world.get::<&LastHitByPlayer>(id).is_ok() {
                    if let Ok(exp) = self.world.get::<&Experience>(id) {
                        total_xp = total_xp.saturating_add(exp.xp_reward);
                    }
                }

                // Collect drop info
                if let Ok(name) = self.world.get::<&Name>(id) {
                    if let Ok(pos) = self.world.get::<&Position>(id) {
                        let boss_raw = self.content.monsters.iter().find(|m| m.name == name.0);
                        if let Some(raw) = boss_raw {
                            if let Some(loot_name) = &raw.guaranteed_loot {
                                if let Some(item_raw) =
                                    self.content.items.iter().find(|i| &i.name == loot_name)
                                {
                                    drops.push((*pos, item_raw.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
        for id in to_despawn {
            if let Err(e) = self.world.despawn(id) {
                log::error!("Failed to despawn monster {:?}: {}", id, e);
            }
            self.monsters_killed += 1;
        }
        self.update_blocked_and_opaque();

        for (pos, raw) in drops {
            crate::spawner::spawn_item(&mut self.world, pos.x, pos.y, &raw);
            self.log.push(format!("The boss dropped {}!", raw.name));
        }

        if total_xp > 0 {
            self.add_player_xp(total_xp);
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
    fn test_monster_sees_player() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 10, y: 10 }));
        let monster = app.world.spawn((
            Monster,
            Position { x: 12, y: 10 },
            Viewshed { visible_tiles: 8 },
            AlertState::Sleeping,
        ));

        // Light the player's position
        let idx = (10 * app.map.width + 10) as usize;
        app.map.light[idx] = 1.0;

        app.update_monster_perception(monster, player);

        let alert = app.world.get::<&AlertState>(monster).unwrap();
        assert_eq!(*alert, AlertState::Aggressive);
    }

    #[test]
    fn test_monster_hears_noise() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 10, y: 10 }));
        let monster = app.world.spawn((
            Monster,
            Position { x: 20, y: 20 },
            Viewshed { visible_tiles: 8 },
            AlertState::Sleeping,
        ));

        // Create noise at monster position
        let idx = (20 * app.map.width + 20) as usize;
        app.map.sound[idx] = 2.0;

        app.update_monster_perception(monster, player);

        let alert = app.world.get::<&AlertState>(monster).unwrap();
        assert_eq!(*alert, AlertState::Curious { x: 10, y: 10 });
    }

    #[test]
    fn test_boss_loot_drop() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Position { x: 0, y: 0 }));

        // Mock content
        let monster_raw = crate::content::RawMonster {
            name: "Boss".to_string(),
            glyph: 'B',
            color: (255, 0, 0),
            hp: 100,
            defense: 10,
            power: 20,
            viewshed: 10,
            spawn_chance: 0.0,
            min_floor: 1,
            max_floor: 10,
            personality: Personality::Brave,
            faction: FactionKind::Orcs,
            xp_reward: 500,
            ranged: None,
            is_boss: Some(true),
            phases: None,
            guaranteed_loot: Some("Amulet".to_string()),
            branches: None,
        };
        let item_raw = crate::content::RawItem {
            name: "Amulet".to_string(),
            glyph: '"',
            color: (255, 215, 0),
            spawn_chance: 0.0,
            min_floor: 1,
            max_floor: 10,
            price: 1000,
            ..Default::default()
        };
        app.content.monsters = vec![monster_raw];
        app.content.items = vec![item_raw];

        let _boss = app.world.spawn((
            Monster,
            Name("Boss".to_string()),
            Position { x: 10, y: 10 },
            CombatStats {
                hp: 0,
                max_hp: 100,
                defense: 10,
                power: 20,
            },
        ));

        app.cleanup_dead_entities();

        // Check if loot was dropped
        let mut loot_query = app.world.query::<(&Name, &Item)>();
        let loot = loot_query.iter().find(|(_, (name, _))| name.0 == "Amulet");
        assert!(loot.is_some());
    }
}
