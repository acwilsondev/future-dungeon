use crate::app::{App, DamageRoute};
use crate::components::*;
use rand::Rng;

const TORCH_FADE_TURNS: i32 = 1000;

impl App {
    pub fn on_turn_tick(&mut self) {
        self.turn_count += 1;
        let Some(player_id) = self.get_player_id() else {
            log::error!("Player not found in turn tick");
            return;
        };

        self.update_light_sources(player_id);
        self.apply_passive_equipment_effects(player_id);
        self.cleanup_noise();
        self.apply_status_effects(player_id);
        self.tick_aegis(player_id);
        self.tick_heat();
        self.tick_shredded();
        self.tick_mana_regen();
        self.handle_dead_monsters_from_poison();
        self.trim_log();
    }

    fn tick_heat(&mut self) {
        for (_id, meter) in self.world.query::<&mut HeatMeter>().iter() {
            if meter.venting > 0 {
                meter.venting -= 1;
            } else {
                meter.current = meter.current.saturating_sub(1);
            }
        }
    }

    fn tick_shredded(&mut self) {
        let mut to_remove = Vec::new();
        for (id, s) in self.world.query::<&mut Shredded>().iter() {
            if s.decay_timer > 0 {
                s.decay_timer -= 1;
            }
            if s.decay_timer == 0 {
                s.stacks = s.stacks.saturating_sub(1);
                if s.stacks == 0 {
                    to_remove.push(id);
                } else {
                    s.decay_timer = SHREDDED_DECAY_INTERVAL;
                }
            }
        }
        for id in to_remove {
            self.world.remove_one::<Shredded>(id).ok();
        }
    }

    fn tick_aegis(&mut self, player_id: hecs::Entity) {
        let mut had_drought = Vec::new();
        for (id, _) in self.world.query::<&AegisDrought>().iter() {
            had_drought.push(id);
        }

        let mut drought_to_remove = Vec::new();
        for (id, drought) in self.world.query::<&mut AegisDrought>().iter() {
            drought.duration = drought.duration.saturating_sub(1);
            if drought.duration == 0 {
                drought_to_remove.push(id);
            }
        }
        for id in &drought_to_remove {
            self.world.remove_one::<AegisDrought>(*id).ok();
            if *id == player_id {
                self.log.push("Your aegis is recharging again.".to_string());
            }
        }

        let mut boost_expired = Vec::new();
        for (id, boost) in self.world.query::<&mut AegisBoost>().iter() {
            boost.duration = boost.duration.saturating_sub(1);
            if boost.duration == 0 {
                boost_expired.push((id, boost.magnitude));
            }
        }
        for (id, magnitude) in boost_expired {
            if let Ok(mut aegis) = self.world.get::<&mut Aegis>(id) {
                aegis.max = (aegis.max - magnitude).max(0);
                aegis.current = aegis.current.min(aegis.max);
            }
            self.world.remove_one::<AegisBoost>(id).ok();
            if id == player_id {
                self.log
                    .push("Your temporary aegis boost fades.".to_string());
            }
        }

        let mut regen_targets = Vec::new();
        for (id, _) in self.world.query::<&Aegis>().iter() {
            if had_drought.contains(&id) {
                continue;
            }
            regen_targets.push(id);
        }
        for id in regen_targets {
            if let Ok(mut aegis) = self.world.get::<&mut Aegis>(id) {
                if aegis.current < aegis.max {
                    aegis.current += 1;
                }
            }
        }
    }

    fn trim_log(&mut self) {
        const MAX_LOG: usize = 500;
        if self.log.len() > MAX_LOG {
            self.log.drain(0..self.log.len() - MAX_LOG);
        }
    }

    fn update_light_sources(&mut self, player_id: hecs::Entity) {
        let mut to_remove_light = Vec::new();
        let mut any_light_changed = false;
        {
            for (id, light) in self.world.query::<&mut LightSource>().iter() {
                if let Some(turns) = light.remaining_turns {
                    if turns > 0 {
                        light.remaining_turns = Some(turns - 1);
                        if turns == TORCH_FADE_TURNS + 1 {
                            light.base_range /= 2;
                            light.range = light.range.min(light.base_range);
                            any_light_changed = true;
                            if id == player_id {
                                self.log.push("Your torch begins to dim...".to_string());
                            }
                        }
                    } else {
                        to_remove_light.push(id);
                        any_light_changed = true;
                    }
                }
                if light.flicker {
                    let flicker_amount = self.rng.random_range(-1..=1);
                    let new_range = (light.base_range + flicker_amount).max(1);
                    if new_range != light.range {
                        light.range = new_range;
                        any_light_changed = true;
                    }
                }
            }
        }
        for id in to_remove_light {
            self.world.remove_one::<LightSource>(id).ok();
            if id == player_id {
                self.log
                    .push("Your torch flickers and goes out!".to_string());
                self.world
                    .insert_one(
                        id,
                        LightSource {
                            range: 2,
                            base_range: 2,
                            color: (150, 150, 100),
                            remaining_turns: None,
                            flicker: false,
                        },
                    )
                    .ok();
            }
        }
        if any_light_changed {
            self.update_fov();
        }
    }

    fn apply_passive_equipment_effects(&mut self, player_id: hecs::Entity) {
        if self.turn_count.is_multiple_of(5) {
            let has_regen = self
                .world
                .query::<(&Equipped, &InBackpack, &Regeneration)>()
                .iter()
                .any(|(_, (_, backpack, _))| backpack.owner == player_id);
            if has_regen {
                if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                    if stats.hp < stats.max_hp {
                        stats.hp += 1;
                        self.log
                            .push("The Ring of Regeneration heals you.".to_string());
                    }
                }
            }
        }
    }

    fn cleanup_noise(&mut self) {
        let mut to_despawn_noise = Vec::new();
        for (id, _) in self.world.query::<&Noise>().iter() {
            to_despawn_noise.push(id);
        }
        for id in to_despawn_noise {
            if let Err(e) = self.world.despawn(id) {
                log::error!("Failed to despawn noise entity {:?}: {}", id, e);
            }
        }
    }

    fn apply_status_effects(&mut self, player_id: hecs::Entity) {
        let mut to_remove_confusion = Vec::new();
        let mut to_remove_poison = Vec::new();
        let mut to_remove_strength = Vec::new();
        let mut to_remove_speed = Vec::new();
        let mut to_remove_mired = Vec::new();
        let mut armored_expiration = Vec::new();
        let mut to_remove_armored = Vec::new();
        let mut poison_damage = Vec::new();
        let mut strength_expiration = Vec::new();
        let mut mired_recovery = Vec::new();
        let mut armored_recovery = Vec::new();

        for (id, (_stats,)) in self.world.query::<(&CombatStats,)>().iter() {
            if let Ok(mut poison) = self.world.get::<&mut Poison>(id) {
                poison_damage.push((id, poison.damage));
                poison.turns -= 1;
                if poison.turns <= 0 {
                    to_remove_poison.push(id);
                }
            }
            if let Ok(mut confusion) = self.world.get::<&mut Confusion>(id) {
                confusion.turns -= 1;
                if confusion.turns <= 0 {
                    to_remove_confusion.push(id);
                }
            }
            if let Ok(mut strength) = self.world.get::<&mut Strength>(id) {
                strength.turns -= 1;
                if strength.turns <= 0 {
                    strength_expiration.push((id, strength.amount));
                    to_remove_strength.push(id);
                }
            }
            if let Ok(mut speed) = self.world.get::<&mut Speed>(id) {
                speed.turns -= 1;
                if speed.turns <= 0 {
                    to_remove_speed.push(id);
                }
            }
            if let Ok(mut mired) = self.world.get::<&mut Mired>(id) {
                if let Some(save) = mired.recovery_save {
                    mired_recovery.push((id, save));
                }
                mired.duration = mired.duration.saturating_sub(1);
                if mired.duration == 0 {
                    to_remove_mired.push(id);
                }
            }
            if let Ok(mut armored) = self.world.get::<&mut Armored>(id) {
                if let Some(save) = armored.recovery_save {
                    armored_recovery.push((id, save, armored.magnitude));
                }
                armored.duration = armored.duration.saturating_sub(1);
                if armored.duration == 0 {
                    armored_expiration.push((id, armored.magnitude));
                    to_remove_armored.push(id);
                }
            }
        }

        const RECOVERY_DC: i32 = 10;
        for (id, attr) in mired_recovery {
            if self.make_saving_throw(id, RECOVERY_DC, attr.to_saving_throw_kind()) {
                to_remove_mired.push(id);
            }
        }
        for (id, attr, magnitude) in armored_recovery {
            if self.make_saving_throw(id, RECOVERY_DC, attr.to_saving_throw_kind()) {
                armored_expiration.push((id, magnitude));
                to_remove_armored.push(id);
            }
        }

        for (id, damage) in poison_damage {
            self.apply_damage(id, damage, DamageRoute::Systemic);
            if id == player_id {
                self.log
                    .push(format!("You suffer {} damage from poison!", damage));
            }
        }

        for (id, amount) in strength_expiration {
            if let Ok(mut stats) = self.world.get::<&mut CombatStats>(id) {
                stats.power -= amount;
                if id == player_id {
                    self.log
                        .push("You feel your extra strength wear off.".to_string());
                }
            }
        }

        for id in to_remove_poison {
            self.world.remove_one::<Poison>(id).ok();
        }
        for id in to_remove_confusion {
            self.world.remove_one::<Confusion>(id).ok();
            if id == player_id {
                self.log.push("You are no longer confused.".to_string());
            } else {
                self.log
                    .push("A monster snaps out of confusion.".to_string());
            }
        }
        for id in to_remove_strength {
            self.world.remove_one::<Strength>(id).ok();
        }
        for id in to_remove_speed {
            self.world.remove_one::<Speed>(id).ok();
        }
        for id in to_remove_mired {
            self.world.remove_one::<Mired>(id).ok();
            if id == player_id {
                self.log.push("You break free of the mire.".to_string());
            }
        }
        for (id, magnitude) in armored_expiration {
            if let Ok(mut stats) = self.world.get::<&mut CombatStats>(id) {
                stats.defense -= magnitude;
            }
            if id == player_id {
                self.log.push("Your magical armor fades.".to_string());
            }
        }
        for id in to_remove_armored {
            self.world.remove_one::<Armored>(id).ok();
        }
    }

    fn handle_dead_monsters_from_poison(&mut self) {
        let mut to_despawn = Vec::new();
        let mut total_xp: i32 = 0;
        for (id, (stats, _)) in self.world.query::<(&CombatStats, &Monster)>().iter() {
            if stats.hp <= 0 {
                to_despawn.push(id);
                if self.world.get::<&LastHitByPlayer>(id).is_ok() {
                    if let Ok(exp) = self.world.get::<&Experience>(id) {
                        total_xp = total_xp.saturating_add(exp.xp_reward);
                    }
                }
            }
        }
        for id in to_despawn {
            let name = self
                .world
                .get::<&Name>(id)
                .map(|n| n.0.clone())
                .unwrap_or("Monster".to_string());
            self.log.push(format!("{} dies from poison!", name));
            if let Err(e) = self.world.despawn(id) {
                log::error!("Failed to despawn monster {:?}: {}", id, e);
            }
            self.monsters_killed += 1;
        }
        self.update_blocked_and_opaque();
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
        app
    }

    #[test]
    fn test_poison_status_effect() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 5,
            },
            Position { x: 0, y: 0 },
        ));
        app.world
            .insert_one(
                player,
                Poison {
                    damage: 2,
                    turns: 3,
                },
            )
            .unwrap();

        app.on_turn_tick();
        {
            let stats = app.world.get::<&CombatStats>(player).unwrap();
            assert_eq!(stats.hp, 8);
            let poison = app.world.get::<&Poison>(player).unwrap();
            assert_eq!(poison.turns, 2);
        }

        app.on_turn_tick();
        app.on_turn_tick();
        {
            let stats = app.world.get::<&CombatStats>(player).unwrap();
            assert_eq!(stats.hp, 4);
            assert!(app.world.get::<&Poison>(player).is_err());
        }
    }

    #[test]
    fn test_confusion_status_effect() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 5,
            },
            Position { x: 0, y: 0 },
        ));
        app.world
            .insert_one(player, Confusion { turns: 2 })
            .unwrap();

        app.on_turn_tick();
        assert!(app.world.get::<&Confusion>(player).is_ok());

        app.on_turn_tick();
        assert!(app.world.get::<&Confusion>(player).is_err());
    }

    #[test]
    fn test_strength_status_effect() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 5,
            },
            Position { x: 0, y: 0 },
        ));
        app.world
            .insert_one(
                player,
                Strength {
                    amount: 3,
                    turns: 2,
                },
            )
            .unwrap();
        // Manually add the power bonus as would happen when using the item
        if let Ok(mut stats) = app.world.get::<&mut CombatStats>(player) {
            stats.power += 3;
        }

        app.on_turn_tick();
        assert_eq!(app.world.get::<&CombatStats>(player).unwrap().power, 8);

        app.on_turn_tick();
        assert_eq!(app.world.get::<&CombatStats>(player).unwrap().power, 5);
        assert!(app.world.get::<&Strength>(player).is_err());
    }

    #[test]
    fn test_light_source_depletion() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            LightSource {
                range: 10,
                base_range: 10,
                color: (255, 255, 255),
                remaining_turns: Some(1002),
                flicker: false,
            },
        ));

        app.on_turn_tick(); // turns -> 1001
        {
            let light = app.world.get::<&LightSource>(player).unwrap();
            assert_eq!(light.remaining_turns, Some(1001));
            assert_eq!(light.base_range, 10);
        }

        app.on_turn_tick(); // turns -> 1000, trigger dimming
        {
            let light = app.world.get::<&LightSource>(player).unwrap();
            assert_eq!(light.base_range, 5);
            assert_eq!(light.range, 5);
        }

        let mut light = app.world.get::<&mut LightSource>(player).unwrap();
        light.remaining_turns = Some(0);
        drop(light);

        app.on_turn_tick(); // turns was 0 -> exhaustion
        {
            let light = app.world.get::<&LightSource>(player).unwrap();
            assert_eq!(light.remaining_turns, None); // Should have reset to default torch
            assert_eq!(light.range, 2);
        }
    }

    #[test]
    fn test_monster_dies_from_poison() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let monster = app.world.spawn((
            Monster,
            CombatStats {
                hp: 1,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
            Position { x: 1, y: 1 },
            Name("Test Monster".to_string()),
            Experience {
                level: 1,
                xp: 0,
                next_level_xp: 0,
                xp_reward: 10,
            },
        ));
        app.world
            .insert_one(
                monster,
                Poison {
                    damage: 2,
                    turns: 5,
                },
            )
            .unwrap();
        app.world.insert_one(monster, LastHitByPlayer).unwrap();

        app.on_turn_tick();

        assert_eq!(app.monsters_killed, 1);
        assert!(app.world.get::<&Monster>(monster).is_err());
    }

    #[test]
    fn test_ring_of_regeneration() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 5,
                max_hp: 10,
                defense: 0,
                power: 5,
            },
        ));
        app.world.spawn((
            Item,
            Regeneration,
            Equipped {
                slot: EquipmentSlot::LeftFinger,
            },
            InBackpack { owner: player },
        ));

        app.turn_count = 4; // next tick will be 5, which is a multiple of 5
        app.on_turn_tick();

        let stats = app.world.get::<&CombatStats>(player).unwrap();
        assert_eq!(stats.hp, 6);
    }

    #[test]
    fn test_noise_cleanup() {
        let mut app = setup_test_app();
        app.world.spawn((Player, Position { x: 0, y: 0 }));
        app.world
            .spawn((Position { x: 1, y: 1 }, Noise { amount: 10.0 }));

        app.on_turn_tick();
        // Player remains, noise is gone (Noise entity + possible noise generated by movement if we moved, but here we just tick)
        assert_eq!(app.world.query::<&Noise>().iter().count(), 0);
    }

    #[test]
    fn test_armored_expires_and_strips_defense() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 5, // +3 base + 2 from Armored
                power: 1,
            },
            Armored {
                magnitude: 2,
                duration: 1,
                recovery_save: None,
            },
        ));
        app.on_turn_tick();
        assert!(app.world.get::<&Armored>(player).is_err());
        let stats = app.world.get::<&CombatStats>(player).unwrap();
        assert_eq!(stats.defense, 3);
    }

    #[test]
    fn test_mired_expires() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
            Mired {
                magnitude: 1,
                duration: 1,
                recovery_save: None,
            },
        ));
        app.on_turn_tick();
        assert!(app.world.get::<&Mired>(player).is_err());
    }

    #[test]
    fn test_aegis_regens_one_per_turn() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
            Aegis { current: 2, max: 5 },
        ));
        app.on_turn_tick();
        assert_eq!(app.world.get::<&Aegis>(player).unwrap().current, 3);
        app.on_turn_tick();
        app.on_turn_tick();
        assert_eq!(app.world.get::<&Aegis>(player).unwrap().current, 5);
        app.on_turn_tick();
        assert_eq!(
            app.world.get::<&Aegis>(player).unwrap().current,
            5,
            "clamped at max"
        );
    }

    #[test]
    fn test_aegis_drought_blocks_regen_and_expires() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
            Aegis { current: 0, max: 5 },
            AegisDrought { duration: 2 },
        ));
        app.on_turn_tick();
        assert_eq!(app.world.get::<&Aegis>(player).unwrap().current, 0);
        assert_eq!(app.world.get::<&AegisDrought>(player).unwrap().duration, 1);
        app.on_turn_tick();
        assert!(app.world.get::<&AegisDrought>(player).is_err());
        assert_eq!(
            app.world.get::<&Aegis>(player).unwrap().current,
            0,
            "no regen on expiry tick"
        );
        app.on_turn_tick();
        assert_eq!(
            app.world.get::<&Aegis>(player).unwrap().current,
            1,
            "regen resumes next tick"
        );
    }

    #[test]
    fn test_aegis_boost_expires_and_trims_current_and_max() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
            Aegis { current: 7, max: 7 },
            AegisBoost {
                magnitude: 3,
                duration: 1,
            },
        ));
        app.on_turn_tick();
        assert!(app.world.get::<&AegisBoost>(player).is_err());
        let a = app.world.get::<&Aegis>(player).unwrap();
        assert_eq!(a.max, 4);
        assert_eq!(a.current, 4);
    }

    #[test]
    fn test_heat_passive_cooldown_decrements_when_not_venting() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));
        let weapon = app.world.spawn((HeatMeter {
            current: 3,
            capacity: 6,
            venting: 0,
        },));
        app.on_turn_tick();
        let m = app.world.get::<&HeatMeter>(weapon).unwrap();
        assert_eq!(m.current, 2);
        assert_eq!(m.venting, 0);
    }

    #[test]
    fn test_heat_venting_decrements_and_skips_cooldown() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));
        let weapon = app.world.spawn((HeatMeter {
            current: 6,
            capacity: 6,
            venting: 3,
        },));
        app.on_turn_tick();
        let m = app.world.get::<&HeatMeter>(weapon).unwrap();
        assert_eq!(m.venting, 2);
        assert_eq!(m.current, 6); // Remains at 6 while venting
        drop(m);
        app.on_turn_tick();
        app.on_turn_tick();
        {
            let m = app.world.get::<&HeatMeter>(weapon).unwrap();
            assert_eq!(m.venting, 0);
            assert_eq!(m.current, 6); // Still at 6 immediately after vent expires
        }
        app.on_turn_tick();
        assert_eq!(app.world.get::<&HeatMeter>(weapon).unwrap().current, 5); // Now it starts cooling
    }

    #[test]
    fn test_heat_current_floors_at_zero() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));
        let weapon = app.world.spawn((HeatMeter {
            current: 0,
            capacity: 6,
            venting: 0,
        },));
        app.on_turn_tick();
        assert_eq!(app.world.get::<&HeatMeter>(weapon).unwrap().current, 0);
    }

    fn spawn_pl(app: &mut App) -> hecs::Entity {
        app.world.spawn((
            Player,
            Position { x: 0, y: 0 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ))
    }

    #[test]
    fn test_shredded_decays_one_stack_per_interval() {
        let mut app = setup_test_app();
        let _player = spawn_pl(&mut app);
        let t = app.world.spawn((
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 0,
            },
            Shredded {
                stacks: 5,
                decay_timer: SHREDDED_DECAY_INTERVAL,
            },
        ));
        for _ in 0..SHREDDED_DECAY_INTERVAL {
            app.on_turn_tick();
        }
        let s = app.world.get::<&Shredded>(t).unwrap();
        assert_eq!(s.stacks, 4);
        assert_eq!(s.decay_timer, SHREDDED_DECAY_INTERVAL);
    }

    #[test]
    fn test_shredded_fully_removed_after_total_decay() {
        let mut app = setup_test_app();
        let _player = spawn_pl(&mut app);
        let t = app.world.spawn((
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 0,
            },
            Shredded {
                stacks: 5,
                decay_timer: SHREDDED_DECAY_INTERVAL,
            },
        ));
        // 5 stacks × 5 turns = 25 turns total decay.
        for _ in 0..(SHREDDED_DECAY_INTERVAL * 5) {
            app.on_turn_tick();
        }
        assert!(app.world.get::<&Shredded>(t).is_err());
    }
}
