use crate::app::{App, RunState};
use crate::components::*;
use rand::Rng;

impl App {
    pub fn on_turn_tick(&mut self) {
        self.turn_count += 1;
        let player_id = self.get_player_id().expect("Player not found in turn tick");

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
            self.world.despawn(id).expect("Failed to despawn noise");
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
            self.world.despawn(id).expect("Failed to despawn monster");
            self.monsters_killed += 1;
        }
        self.update_blocked_and_opaque();
        if total_xp > 0 {
            self.add_player_xp(total_xp);
        }
    }
}
