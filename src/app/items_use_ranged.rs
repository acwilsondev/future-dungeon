use crate::app::{App, RunState, VisualEffect};
use crate::components::*;
use bracket_pathfinding::prelude::*;
use rand::Rng;
use ratatui::prelude::Color;

impl App {
    fn add_projectile_animation(&mut self, line: &[(u16, u16)], actual_target: (u16, u16)) {
        let path: Vec<(u16, u16)> = line
            .iter()
            .take_while(|&&p| p != actual_target)
            .copied()
            .chain(std::iter::once(actual_target))
            .collect();
        self.effects.push(VisualEffect::Projectile {
            path,
            glyph: '*',
            fg: Color::Yellow,
            frame: 0,
            speed: 1,
        });
    }

    fn consume_ammo(&mut self, player_id: hecs::Entity) {
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

    fn handle_aoe_effect(
        &mut self,
        radius: i32,
        actual_target: (u16, u16),
        power: i32,
        item_name: &str,
    ) {
        let mut targets = Vec::new();
        for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
            let dist = ((pos.x as f32 - actual_target.0 as f32).powi(2)
                + (pos.y as f32 - actual_target.1 as f32).powi(2))
            .sqrt();
            if dist <= radius as f32 {
                targets.push(id);
            }
        }
        self.log.push(format!("The {} explodes!", item_name));
        self.generate_noise(actual_target.0, actual_target.1, 15.0);
        for target_id in targets {
            let mut flash_pos = None;
            if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target_id) {
                stats.hp -= power;
                if let Ok(t_pos) = self.world.get::<&Position>(target_id) {
                    flash_pos = Some(*t_pos);
                }
            }
            if let Some(pos) = flash_pos {
                self.effects.push(VisualEffect::Flash {
                    x: pos.x,
                    y: pos.y,
                    glyph: '*',
                    fg: Color::Indexed(208),
                    bg: None,
                    duration: 10,
                });
            }
            let _ = self.world.insert_one(target_id, LastHitByPlayer);
            let _ = self.world.insert_one(target_id, AlertState::Aggressive);
        }
    }

    fn handle_status_effect(
        &mut self,
        actual_target: (u16, u16),
        item_name: &str,
        confusion: Option<i32>,
        poison: Option<Poison>,
    ) {
        let mut targets = Vec::new();
        for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
            if pos.x == actual_target.0 && pos.y == actual_target.1 {
                targets.push(id);
            }
        }
        self.generate_noise(actual_target.0, actual_target.1, 4.0);
        for target_id in targets {
            if let Some(turns) = confusion {
                self.log
                    .push(format!("The monster is confused by the {}!", item_name));
                let _ = self.world.insert_one(target_id, Confusion { turns });
            }
            if let Some(p) = poison {
                self.log
                    .push(format!("The monster is poisoned by the {}!", item_name));
                let _ = self.world.insert_one(target_id, p);
                let _ = self.world.insert_one(target_id, LastHitByPlayer);
            }
            let _ = self.world.insert_one(target_id, AlertState::Aggressive);
        }
    }

    fn handle_direct_damage(
        &mut self,
        attacker: hecs::Entity,
        target_id: hecs::Entity,
        actual_target: (u16, u16),
        specific_weapon: Option<hecs::Entity>,
        disadvantage_count: u32,
    ) {
        let res = self.resolve_attack(attacker, target_id, specific_weapon, disadvantage_count, true);
        self.apply_attack_result(target_id, &res, actual_target.0, actual_target.1);
        
        if res.hit {
            let _ = self.world.insert_one(target_id, LastHitByPlayer);
            let _ = self.world.insert_one(target_id, AlertState::Aggressive);
        }
    }

    fn cleanup_dead_monsters(&mut self) -> i32 {
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
        total_xp
    }

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

            let points = line2d(
                LineAlg::Bresenham,
                Point::new(player_pos.x, player_pos.y),
                Point::new(self.targeting_cursor.0, self.targeting_cursor.1),
            );
            let line: Vec<(u16, u16)> = points.iter().map(|p| (p.x as u16, p.y as u16)).collect();

            let mut actual_target = self.targeting_cursor;
            for p in &line[1..] {
                let idx = (p.1 * self.map.width + p.0) as usize;
                if self.map.blocked[idx] {
                    actual_target = *p;
                    self.log.push(format!("The {} is blocked!", item_name));
                    break;
                }
            }

            self.add_projectile_animation(&line, actual_target);

            // Collect info before mutations
            let aoe_radius = self.world.get::<&AreaOfEffect>(item_id).ok().map(|a| a.radius);
            let confusion_turns = self.world.get::<&Confusion>(item_id).ok().map(|c| c.turns);
            let poison_effect = self.world.get::<&Poison>(item_id).ok().map(|p| *p);
            let mut power = self.world.get::<&CombatStats>(item_id).map(|s| s.power).unwrap_or(10);
            let mut disadvantage = 0;
            let ranged_weapon_info = self.world.get::<&RangedWeapon>(item_id).ok().map(|rw| *rw);
            if let Some(rw) = ranged_weapon_info {
                let dist = (((player_pos.x as f32 - actual_target.0 as f32).powi(2)
                    + (player_pos.y as f32 - actual_target.1 as f32).powi(2))
                .sqrt()) as i32;
                if dist > rw.range {
                    disadvantage = ((dist - rw.range) / rw.range_increment) as u32 + 1;
                }
                power = rw.damage_bonus;
                self.consume_ammo(player_id);
            }

            if let Some(radius) = aoe_radius {
                self.handle_aoe_effect(radius, actual_target, power, &item_name);
            } else if confusion_turns.is_some() || poison_effect.is_some() {
                self.handle_status_effect(actual_target, &item_name, confusion_turns, poison_effect);
            } else {
                let mut targets = Vec::new();
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 {
                        targets.push(id);
                    }
                }
                for target_id in targets {
                    self.handle_direct_damage(
                        player_id,
                        target_id,
                        actual_target,
                        Some(item_id),
                        disadvantage,
                    );

                    // Off-hand ranged proc?
                    if let Some(off_hand_id) = self.get_off_hand_weapon(player_id) {
                        if self.world.get::<&RangedWeapon>(off_hand_id).is_ok() {
                            let dex_mod = self.get_attribute_modifier(player_id, |a| a.dexterity);
                            let chance = 10 + (dex_mod * 10);
                            if self.rng.gen_range(1..=100) <= chance {
                                self.handle_direct_damage(
                                    player_id,
                                    target_id,
                                    actual_target,
                                    Some(off_hand_id),
                                    disadvantage,
                                );
                            }
                        }
                    }
                }
            }

            let total_xp = self.cleanup_dead_monsters();
            if total_xp > 0 {
                self.add_player_xp(total_xp);
            }

            self.update_blocked_and_opaque();
            self.identify_item(item_id);

            if self.world.get::<&RangedWeapon>(item_id).is_err() {
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

    pub fn trigger_ranged_targeting(&mut self) {
        let Some(player_id) = self.get_player_id() else { return; };
        
        let mut ranged_item = None;
        for slot in [EquipmentSlot::MainHand, EquipmentSlot::OffHand] {
            for (id, (eq, _rw)) in self.world.query::<(&Equipped, &RangedWeapon)>().iter() {
                if let Ok(backpack) = self.world.get::<&InBackpack>(id) {
                    if backpack.owner == player_id && eq.slot == slot {
                        ranged_item = Some(id);
                        break;
                    }
                }
            }
            if ranged_item.is_some() { break; }
        }

        if let Some(item_id) = ranged_item {
            let has_ammo = self
                .world
                .query::<(&Ammunition, &InBackpack)>()
                .iter()
                .any(|(_, (_, backpack))| backpack.owner == player_id);
            
            if !has_ammo {
                self.log.push("You have no ammunition for this weapon!".to_string());
                return;
            }

            if let Ok(player_pos) = self.world.get::<&Position>(player_id) {
                let item_name = self.get_item_name(item_id);
                self.targeting_cursor = (player_pos.x, player_pos.y);
                self.targeting_item = Some(item_id);
                self.state = RunState::ShowTargeting;
                self.log.push(format!("Select target for {}...", item_name));
            }
        } else {
            self.log.push("You have no ranged weapon equipped!".to_string());
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
    fn test_fire_scroll_aoe() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 10, y: 10 }));
        let monster1 = app.world.spawn((
            Monster,
            Position { x: 12, y: 10 },
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 1 }
        ));
        let monster2 = app.world.spawn((
            Monster,
            Position { x: 13, y: 10 },
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 1 }
        ));
        let scroll = app.world.spawn((
            Item,
            Name("Fire Scroll".to_string()),
            AreaOfEffect { radius: 3 },
            CombatStats { hp: 0, max_hp: 0, defense: 0, power: 8 },
            Consumable,
            InBackpack { owner: player }
        ));

        app.targeting_item = Some(scroll);
        app.targeting_cursor = (12, 10);
        app.fire_targeting_item();

        let stats1 = app.world.get::<&CombatStats>(monster1).unwrap();
        assert_eq!(stats1.hp, 2);
        let stats2 = app.world.get::<&CombatStats>(monster2).unwrap();
        assert_eq!(stats2.hp, 2);
        assert!(app.world.get::<&Item>(scroll).is_err());
    }

    #[test]
    fn test_bow_ranged_weapon() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 10, y: 10 },
            Attributes { strength: 10, dexterity: 50, constitution: 10, intelligence: 10, wisdom: 10, charisma: 10 },
        ));
        let monster = app.world.spawn((
            Monster,
            Position { x: 15, y: 10 },
            Attributes { strength: 10, dexterity: 10, constitution: 10, intelligence: 10, wisdom: 10, charisma: 10 },
            CombatStats { hp: 100, max_hp: 100, defense: 0, power: 1 }
            ));
            let bow = app.world.spawn((
            Item,
            Name("Shortbow".to_string()),
            RangedWeapon { range: 8, range_increment: 12, damage_bonus: 4 },
            InBackpack { owner: player }
            ));
            let _arrows = app.world.spawn((
            Item,
            Ammunition,
            InBackpack { owner: player }
            ));

            app.targeting_item = Some(bow);
            app.targeting_cursor = (15, 10);
        app.fire_targeting_item();

        let stats = app.world.get::<&CombatStats>(monster).unwrap();
        assert!(stats.hp < 100);
    }

    #[test]
    fn test_ranged_blocked() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Position { x: 10, y: 10 }));
        app.map.tiles[(10 * 80 + 11) as usize] = crate::map::TileType::Wall;
        app.map.populate_blocked_and_opaque();

        let wand = app.world.spawn((
            Item,
            Name("Wand".to_string()),
            CombatStats {
                hp: 0,
                max_hp: 0,
                defense: 0,
                power: 5,
            },
        ));

        app.targeting_item = Some(wand);
        app.targeting_cursor = (15, 10);
        app.fire_targeting_item();

        assert!(app.log.last().unwrap().contains("is blocked"));
    }

    #[test]
    fn test_ranged_confusion_poison() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Position { x: 10, y: 10 }));
        let monster1 = app.world.spawn((
            Monster,
            Position { x: 12, y: 10 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));
        let monster2 = app.world.spawn((
            Monster,
            Position { x: 10, y: 12 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));

        let conf_scroll = app.world.spawn((
            Item,
            Name("Confusion Scroll".to_string()),
            Confusion { turns: 5 },
            Consumable,
        ));

        app.targeting_item = Some(conf_scroll);
        app.targeting_cursor = (12, 10);
        app.fire_targeting_item();
        assert!(app.world.get::<&Confusion>(monster1).is_ok());

        let poison_scroll = app.world.spawn((
            Item,
            Name("Poison Scroll".to_string()),
            Poison {
                damage: 2,
                turns: 5,
            },
            Consumable,
        ));
        app.targeting_item = Some(poison_scroll);
        app.targeting_cursor = (10, 12);
        app.fire_targeting_item();
        assert!(app.world.get::<&Poison>(monster2).is_ok());
    }
}
