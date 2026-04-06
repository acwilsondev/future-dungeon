use crate::app::{App, RunState};
use crate::components::*;
use rand::Rng;

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
        self.handle_dead_monsters_from_poison();
    }

    fn update_light_sources(&mut self, player_id: hecs::Entity) {
        let mut to_remove_light = Vec::new();
        let mut any_light_changed = false;
        {
            let mut rng = rand::thread_rng();
            for (id, light) in self.world.query::<&mut LightSource>().iter() {
                if let Some(turns) = light.remaining_turns {
                    if turns > 0 {
                        light.remaining_turns = Some(turns - 1);
                        if turns == 1001 {
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
                    let flicker_amount = rng.gen_range(-1..=1);
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
            let mut regen = false;
            for (id, (_eq, backpack)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
                if backpack.owner == player_id {
                    let name = self
                        .world
                        .get::<&Name>(id)
                        .map(|n| n.0.clone())
                        .unwrap_or_default();
                    if name == "Ring of Regeneration" {
                        regen = true;
                        break;
                    }
                }
            }
            if regen {
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
        let mut poison_damage = Vec::new();
        let mut strength_expiration = Vec::new();

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
        }

        for (id, damage) in poison_damage {
            if let Ok(mut stats) = self.world.get::<&mut CombatStats>(id) {
                stats.hp -= damage;
                if id == player_id {
                    self.log
                        .push(format!("You suffer {} damage from poison!", damage));
                    if stats.hp <= 0 {
                        self.death = true;
                        self.state = RunState::Dead;
                    }
                }
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
        let mut app = App::new_random();
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
        app.world.insert_one(player, Poison { damage: 2, turns: 3 }).unwrap();

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
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 5 },
            Position { x: 0, y: 0 },
        ));
        app.world.insert_one(player, Confusion { turns: 2 }).unwrap();

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
            CombatStats { hp: 10, max_hp: 10, defense: 0, power: 5 },
            Position { x: 0, y: 0 },
        ));
        app.world.insert_one(player, Strength { amount: 3, turns: 2 }).unwrap();
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
            }
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
            CombatStats { hp: 1, max_hp: 10, defense: 0, power: 1 },
            Position { x: 1, y: 1 },
            Name("Test Monster".to_string()),
            Experience { level: 1, xp: 0, next_level_xp: 0, xp_reward: 10 }
        ));
        app.world.insert_one(monster, Poison { damage: 2, turns: 5 }).unwrap();
        app.world.insert_one(monster, LastHitByPlayer).unwrap();

        app.on_turn_tick();
        
        assert_eq!(app.monsters_killed, 1);
        assert!(app.world.get::<&Monster>(monster).is_err());
    }
}
