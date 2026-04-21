use crate::app::{App, DamageRoute, RunState, VisualEffect};
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

    /// Consume power for a fired ranged weapon. Returns `true` if the weapon
    /// was able to fire. For `Ammo` and `HeavyAmmo`, this despawns one unit
    /// of the appropriate fungible consumable; `Heat` weapons use the
    /// `HeatMeter` component and consume no inventory.
    fn consume_power(&mut self, player_id: hecs::Entity, item_id: hecs::Entity) -> bool {
        let Ok(rw) = self.world.get::<&RangedWeapon>(item_id).map(|rw| *rw) else {
            return true;
        };
        match rw.power_source {
            WeaponPowerSource::Ammo => {
                let ammo_id = self
                    .world
                    .query::<(&Ammunition, &InBackpack)>()
                    .iter()
                    .find(|(_, (_, backpack))| backpack.owner == player_id)
                    .map(|(id, _)| id);
                if let Some(aid) = ammo_id {
                    if let Err(e) = self.world.despawn(aid) {
                        log::error!("Failed to despawn ammunition: {}", e);
                    }
                }
                true
            }
            WeaponPowerSource::HeavyAmmo => {
                // Fungible Heavy Ammo items are not yet in content; for now treat
                // absence as "out of ammo" and refuse to fire.
                self.log.push("Out of heavy ammo.".to_string());
                false
            }
            WeaponPowerSource::Heat => true,
        }
    }

    /// After a Heat weapon fires, accumulate heat and trigger a vent at
    /// capacity. `shots` > 1 for Burst (slice 4). Returns true if the
    /// weapon just entered a vent cycle.
    fn apply_heat_after_fire(&mut self, item_id: hecs::Entity, shots: u32) -> bool {
        let rw = match self.world.get::<&RangedWeapon>(item_id).map(|rw| *rw) {
            Ok(rw) if rw.power_source == WeaponPowerSource::Heat => rw,
            _ => return false,
        };
        let mut meter = match self.world.get::<&mut HeatMeter>(item_id) {
            Ok(m) => m,
            Err(_) => return false,
        };
        meter.current = meter.current.saturating_add(rw.heat_per_shot * shots);
        if meter.current >= meter.capacity {
            meter.current = 0;
            meter.venting = if rw.efficient_cooldown { 1 } else { 3 };
            true
        } else {
            false
        }
    }

    /// Returns true if the weapon is currently venting and cannot fire.
    fn is_venting(&self, item_id: hecs::Entity) -> bool {
        self.world
            .get::<&HeatMeter>(item_id)
            .map(|m| m.venting > 0)
            .unwrap_or(false)
    }

    fn handle_aoe_effect(
        &mut self,
        radius: i32,
        actual_target: (u16, u16),
        power: i32,
        item_name: &str,
    ) {
        let mut targets = Vec::new();
        for (id, (pos, _stats)) in self.world.query::<(&Position, &CombatStats)>().iter() {
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
            let mut damage = power;
            if self.make_saving_throw(target_id, 14, SavingThrowKind::Dexterity) {
                damage /= 2;
                self.log.push(format!(
                    "{} dodges some of the blast!",
                    self.get_entity_name(target_id)
                ));
            }

            let flash_pos = self.world.get::<&Position>(target_id).ok().map(|p| *p);
            let is_player = self.world.get::<&Player>(target_id).is_ok();
            if is_player && self.god_mode {
                self.log
                    .push("Debug: Player is in God Mode! No AOE damage taken.".to_string());
            }
            self.apply_damage(target_id, damage, DamageRoute::Projectile);

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
            if self.world.get::<&Monster>(target_id).is_ok() {
                let _ = self.world.insert_one(target_id, LastHitByPlayer);
                let _ = self.world.insert_one(target_id, AlertState::Aggressive);
            }
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
        for (id, (pos, _stats)) in self.world.query::<(&Position, &CombatStats)>().iter() {
            if pos.x == actual_target.0 && pos.y == actual_target.1 {
                targets.push(id);
            }
        }
        self.generate_noise(actual_target.0, actual_target.1, 4.0);
        for target_id in targets {
            let is_player = self.world.get::<&Player>(target_id).is_ok();
            if let Some(turns) = confusion {
                if is_player && self.god_mode {
                    // Skip
                } else if !self.make_saving_throw(target_id, 14, SavingThrowKind::Intelligence) {
                    self.log.push(format!(
                        "The {} is confused by the {}!",
                        self.get_entity_name(target_id),
                        item_name
                    ));
                    let _ = self.world.insert_one(target_id, Confusion { turns });
                } else {
                    self.log.push(format!(
                        "{} resists the confusion!",
                        self.get_entity_name(target_id)
                    ));
                }
            }
            if let Some(p) = poison {
                if is_player && self.god_mode {
                    // Skip
                } else if !self.make_saving_throw(target_id, 14, SavingThrowKind::Constitution) {
                    self.log.push(format!(
                        "The {} is poisoned by the {}!",
                        self.get_entity_name(target_id),
                        item_name
                    ));
                    let _ = self.world.insert_one(target_id, p);
                    if self.world.get::<&Monster>(target_id).is_ok() {
                        let _ = self.world.insert_one(target_id, LastHitByPlayer);
                    }
                } else {
                    self.log.push(format!(
                        "{} resists the poison!",
                        self.get_entity_name(target_id)
                    ));
                }
            }
            if self.world.get::<&Monster>(target_id).is_ok() {
                let _ = self.world.insert_one(target_id, AlertState::Aggressive);
            }
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
        let res = self.resolve_attack(
            attacker,
            target_id,
            specific_weapon,
            disadvantage_count,
            true,
        );
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
                let name = self
                    .world
                    .get::<&Name>(id)
                    .map(|n| n.0.clone())
                    .unwrap_or("Monster".to_string());
                self.log.push(format!("{} dies!", name));
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

            // Heat-weapon vent lockout: attempting to fire still costs a turn.
            if self.is_venting(item_id) {
                self.log.push(format!("The {} is venting heat.", item_name));
                if self.state != RunState::LevelUp {
                    self.state = RunState::MonsterTurn;
                }
                return;
            }

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
                if !self.consume_power(player_id, item_id) {
                    if self.state != RunState::LevelUp {
                        self.state = RunState::MonsterTurn;
                    }
                    return;
                }
            }

            if let Some(radius) = aoe_radius {
                self.handle_aoe_effect(radius, actual_target, power, &item_name);
            } else if confusion_turns.is_some() || poison_effect.is_some() {
                self.handle_status_effect(
                    actual_target,
                    &item_name,
                    confusion_turns,
                    poison_effect,
                );
            } else {
                let mut targets = Vec::new();
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 {
                        targets.push(id);
                    }
                }
                let burst_count = ranged_weapon_info
                    .map(|rw| rw.burst_count.max(1))
                    .unwrap_or(1);
                for target_id in targets {
                    for shot in 0..burst_count {
                        self.handle_direct_damage(
                            player_id,
                            target_id,
                            actual_target,
                            Some(item_id),
                            disadvantage + shot,
                        );
                    }

                    // Off-hand ranged proc (once per target, not per burst shot)
                    if let Some(off_hand_id) = self.get_off_hand_weapon(player_id) {
                        if self.world.get::<&RangedWeapon>(off_hand_id).is_ok() {
                            let dex_mod = self.get_dex_modifier(player_id);
                            let chance = 10 + (dex_mod * 10);
                            if self.rng.random_range(1..=100) <= chance {
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

            let shots_fired = ranged_weapon_info
                .map(|rw| rw.burst_count.max(1))
                .unwrap_or(1);
            if self.apply_heat_after_fire(item_id, shots_fired) {
                self.log
                    .push(format!("The {} vents superheated gas!", item_name));
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
        let Some(player_id) = self.get_player_id() else {
            return;
        };

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
            if ranged_item.is_some() {
                break;
            }
        }

        if let Some(item_id) = ranged_item {
            let has_ammo = self
                .world
                .query::<(&Ammunition, &InBackpack)>()
                .iter()
                .any(|(_, (_, backpack))| backpack.owner == player_id);

            if !has_ammo {
                self.log
                    .push("You have no ammunition for this weapon!".to_string());
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
            self.log
                .push("You have no ranged weapon equipped!".to_string());
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
            Attributes {
                strength: 10,
                dexterity: -100,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));
        let monster2 = app.world.spawn((
            Monster,
            Position { x: 13, y: 10 },
            Attributes {
                strength: 10,
                dexterity: -100,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));
        let scroll = app.world.spawn((
            Item,
            Name("Fire Scroll".to_string()),
            AreaOfEffect { radius: 3 },
            CombatStats {
                hp: 0,
                max_hp: 0,
                defense: 0,
                power: 8,
            },
            Consumable,
            InBackpack { owner: player },
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
            Attributes {
                strength: 10,
                dexterity: 50,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
        ));
        let monster = app.world.spawn((
            Monster,
            Position { x: 15, y: 10 },
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 100,
                max_hp: 100,
                defense: 0,
                power: 1,
            },
        ));
        let bow = app.world.spawn((
            Item,
            Name("Shortbow".to_string()),
            RangedWeapon {
                range: 8,
                range_increment: 12,
                damage_bonus: 4,
                ..Default::default()
            },
            InBackpack { owner: player },
        ));
        let _arrows = app
            .world
            .spawn((Item, Ammunition, InBackpack { owner: player }));

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
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: -100,
                wisdom: 10,
                charisma: 10,
            },
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
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: -100,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
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

    #[test]
    fn test_apply_heat_after_fire_increments_current() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let weapon = app.world.spawn((
            RangedWeapon {
                range: 6,
                range_increment: 4,
                damage_bonus: 1,
                power_source: WeaponPowerSource::Heat,
                heat_per_shot: 2,
                efficient_cooldown: false,
                ..Default::default()
            },
            HeatMeter {
                current: 0,
                capacity: 6,
                venting: 0,
            },
        ));
        let vented = app.apply_heat_after_fire(weapon, 1);
        assert!(!vented);
        let m = app.world.get::<&HeatMeter>(weapon).unwrap();
        assert_eq!(m.current, 2);
        assert_eq!(m.venting, 0);
    }

    #[test]
    fn test_apply_heat_after_fire_triggers_vent_at_capacity() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let weapon = app.world.spawn((
            RangedWeapon {
                range: 6,
                range_increment: 4,
                damage_bonus: 1,
                power_source: WeaponPowerSource::Heat,
                heat_per_shot: 3,
                efficient_cooldown: false,
                ..Default::default()
            },
            HeatMeter {
                current: 4,
                capacity: 6,
                venting: 0,
            },
        ));
        let vented = app.apply_heat_after_fire(weapon, 1);
        assert!(vented);
        let m = app.world.get::<&HeatMeter>(weapon).unwrap();
        assert_eq!(m.current, 0);
        assert_eq!(m.venting, 3);
    }

    #[test]
    fn test_efficient_cooldown_shortens_vent() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let weapon = app.world.spawn((
            RangedWeapon {
                range: 6,
                range_increment: 4,
                damage_bonus: 1,
                power_source: WeaponPowerSource::Heat,
                heat_per_shot: 6,
                efficient_cooldown: true,
                ..Default::default()
            },
            HeatMeter {
                current: 0,
                capacity: 6,
                venting: 0,
            },
        ));
        let vented = app.apply_heat_after_fire(weapon, 1);
        assert!(vented);
        assert_eq!(app.world.get::<&HeatMeter>(weapon).unwrap().venting, 1);
    }

    #[test]
    fn test_is_venting_detection() {
        let mut app = setup_test_app();
        let cooled = app.world.spawn((HeatMeter {
            current: 2,
            capacity: 6,
            venting: 0,
        },));
        let hot = app.world.spawn((HeatMeter {
            current: 0,
            capacity: 6,
            venting: 2,
        },));
        assert!(!app.is_venting(cooled));
        assert!(app.is_venting(hot));
    }

    #[test]
    fn test_burst_weapon_accumulates_heat_per_shot_times_count() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 10, y: 10 },
            Attributes {
                strength: 10,
                dexterity: 50,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
        ));
        let _monster = app.world.spawn((
            Monster,
            Position { x: 15, y: 10 },
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 100,
                max_hp: 100,
                defense: 0,
                power: 1,
            },
        ));
        let carbine = app.world.spawn((
            Item,
            Name("Carbine".to_string()),
            RangedWeapon {
                range: 8,
                range_increment: 8,
                damage_bonus: 1,
                power_source: WeaponPowerSource::Heat,
                heat_per_shot: 1,
                burst_count: 3,
                ..Default::default()
            },
            HeatMeter {
                current: 0,
                capacity: 9,
                venting: 0,
            },
            InBackpack { owner: player },
        ));

        app.targeting_item = Some(carbine);
        app.targeting_cursor = (15, 10);
        app.fire_targeting_item();

        let meter = app.world.get::<&HeatMeter>(carbine).unwrap();
        assert_eq!(meter.current, 3, "3-burst heat accumulation");
        assert_eq!(meter.venting, 0, "not venting yet");
    }

    #[test]
    fn test_burst_weapon_hits_capacity_triggers_vent() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 10, y: 10 },
            Attributes {
                strength: 10,
                dexterity: 50,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
        ));
        let _monster = app.world.spawn((
            Monster,
            Position { x: 15, y: 10 },
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
            CombatStats {
                hp: 100,
                max_hp: 100,
                defense: 0,
                power: 1,
            },
        ));
        let carbine = app.world.spawn((
            Item,
            Name("Carbine".to_string()),
            RangedWeapon {
                range: 8,
                range_increment: 8,
                damage_bonus: 1,
                power_source: WeaponPowerSource::Heat,
                heat_per_shot: 1,
                burst_count: 3,
                ..Default::default()
            },
            HeatMeter {
                current: 0,
                capacity: 3,
                venting: 0,
            },
            InBackpack { owner: player },
        ));

        app.targeting_item = Some(carbine);
        app.targeting_cursor = (15, 10);
        app.fire_targeting_item();

        let meter = app.world.get::<&HeatMeter>(carbine).unwrap();
        assert_eq!(meter.current, 0, "reset on vent");
        assert_eq!(meter.venting, 3, "vent triggered");
    }
}
