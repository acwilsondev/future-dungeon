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
                self.world
                    .insert_one(id, AlertState::Aggressive)
                    .expect("Failed to update AlertState");
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
                        self.world
                            .insert_one(id, current_alert)
                            .expect("Failed to update AlertState");
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
            self.world.despawn(id).expect("Failed to despawn monster");
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
