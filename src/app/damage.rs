use crate::app::{App, RunState};
use crate::components::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DamageRoute {
    /// Outside the shield. Aegis soaks first, overflow through AV to HP.
    Projectile,
    /// Past the shield, still outside the body. AV mitigates, no Aegis interaction.
    Contact,
    /// Inside the body. HP only, no Aegis, no AV.
    Systemic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DamageOutcome {
    pub aegis_damage: i32,
    pub hp_damage: i32,
    pub aegis_depleted: bool,
}

impl App {
    /// Apply `raw` pre-mitigation damage to `target` through `route`.
    /// Handles Aegis, AV, drought application, and player-death transitions.
    pub fn apply_damage(
        &mut self,
        target: hecs::Entity,
        raw: i32,
        route: DamageRoute,
    ) -> DamageOutcome {
        if raw <= 0 {
            return DamageOutcome::default();
        }

        let is_player = self.world.get::<&Player>(target).is_ok();
        if is_player && self.god_mode {
            return DamageOutcome::default();
        }

        let mut out = DamageOutcome::default();

        match route {
            DamageRoute::Systemic => {
                if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target) {
                    stats.hp -= raw;
                    out.hp_damage = raw;
                }
            }
            DamageRoute::Contact => {
                let av = self.get_target_av(target);
                let hp_damage = (raw - av).max(1);
                if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target) {
                    stats.hp -= hp_damage;
                    out.hp_damage = hp_damage;
                }
            }
            DamageRoute::Projectile => {
                let aegis_before = self
                    .world
                    .get::<&Aegis>(target)
                    .ok()
                    .map(|a| a.current)
                    .unwrap_or(0);
                let aegis_taken = raw.min(aegis_before).max(0);
                if aegis_taken > 0 {
                    if let Ok(mut a) = self.world.get::<&mut Aegis>(target) {
                        a.current -= aegis_taken;
                        out.aegis_damage = aegis_taken;
                        out.aegis_depleted = a.current == 0;
                    }
                }

                let overflow = raw - aegis_taken;
                if overflow > 0 {
                    let av = self.get_target_av(target);
                    let hp_damage = (overflow - av).max(1);
                    if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target) {
                        stats.hp -= hp_damage;
                        out.hp_damage = hp_damage;
                    }
                }

                if aegis_taken > 0 {
                    let drought_turns = if out.aegis_depleted { 10 } else { 5 };
                    self.apply_aegis_drought(target, drought_turns);
                }
            }
        }

        if is_player {
            if let Ok(stats) = self.world.get::<&CombatStats>(target) {
                if stats.hp <= 0 {
                    self.death = true;
                    self.state = RunState::Dead;
                }
            }
        }

        out
    }

    /// Apply or refresh an AegisDrought. Never shortens an existing drought.
    pub fn apply_aegis_drought(&mut self, target: hecs::Entity, new_duration: u32) {
        let should_insert = match self.world.get::<&mut AegisDrought>(target) {
            Ok(mut d) => {
                d.duration = d.duration.max(new_duration);
                false
            }
            Err(_) => true,
        };
        if should_insert {
            let _ = self.world.insert_one(
                target,
                AegisDrought {
                    duration: new_duration,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    fn setup() -> App {
        let mut app = App::new_test(42);
        app.world = World::new();
        app
    }

    fn spawn_target(app: &mut App, hp: i32, defense: i32, aegis: Option<i32>) -> hecs::Entity {
        let entity = app.world.spawn((
            CombatStats {
                hp,
                max_hp: hp,
                defense,
                power: 0,
            },
            Attributes {
                strength: 10,
                dexterity: 10,
                constitution: 10,
                intelligence: 10,
                wisdom: 10,
                charisma: 10,
            },
        ));
        if let Some(a) = aegis {
            app.world
                .insert_one(entity, Aegis { current: a, max: a })
                .unwrap();
        }
        entity
    }

    #[test]
    fn projectile_damages_aegis_before_hp() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, Some(5));
        let out = app.apply_damage(t, 3, DamageRoute::Projectile);
        assert_eq!(out.aegis_damage, 3);
        assert_eq!(out.hp_damage, 0);
        let stats = app.world.get::<&CombatStats>(t).unwrap();
        assert_eq!(stats.hp, 10);
        let a = app.world.get::<&Aegis>(t).unwrap();
        assert_eq!(a.current, 2);
    }

    #[test]
    fn av_does_not_apply_to_aegis() {
        // 8 raw, 5 aegis, AV 3 → Aegis drops 5, overflow 3 through AV (floored at 1) → HP loses 1.
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 3, Some(5));
        let out = app.apply_damage(t, 8, DamageRoute::Projectile);
        assert_eq!(out.aegis_damage, 5);
        assert_eq!(out.hp_damage, 1);
        let a = app.world.get::<&Aegis>(t).unwrap();
        assert_eq!(a.current, 0);
        let stats = app.world.get::<&CombatStats>(t).unwrap();
        assert_eq!(stats.hp, 9);
    }

    #[test]
    fn av_applies_to_overflow_exactly_once() {
        // 10 raw, 3 aegis, AV 2 → Aegis drops 3, overflow 7, HP loses 5.
        let mut app = setup();
        let t = spawn_target(&mut app, 20, 2, Some(3));
        let out = app.apply_damage(t, 10, DamageRoute::Projectile);
        assert_eq!(out.aegis_damage, 3);
        assert_eq!(out.hp_damage, 5);
    }

    #[test]
    fn contact_bypasses_aegis_but_applies_av() {
        // 6 raw melee, 5 aegis, AV 2 → Aegis unchanged, HP loses 4.
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 2, Some(5));
        let out = app.apply_damage(t, 6, DamageRoute::Contact);
        assert_eq!(out.aegis_damage, 0);
        assert_eq!(out.hp_damage, 4);
        let a = app.world.get::<&Aegis>(t).unwrap();
        assert_eq!(a.current, 5);
        let stats = app.world.get::<&CombatStats>(t).unwrap();
        assert_eq!(stats.hp, 6);
    }

    #[test]
    fn systemic_bypasses_both_aegis_and_av() {
        // 2 systemic, 5 aegis, AV 3 → HP loses 2 flat.
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 3, Some(5));
        let out = app.apply_damage(t, 2, DamageRoute::Systemic);
        assert_eq!(out.aegis_damage, 0);
        assert_eq!(out.hp_damage, 2);
        let stats = app.world.get::<&CombatStats>(t).unwrap();
        assert_eq!(stats.hp, 8);
    }

    #[test]
    fn fully_absorbed_projectile_does_not_touch_hp() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, Some(10));
        let out = app.apply_damage(t, 5, DamageRoute::Projectile);
        assert_eq!(out.hp_damage, 0);
        let stats = app.world.get::<&CombatStats>(t).unwrap();
        assert_eq!(stats.hp, 10);
    }

    #[test]
    fn partial_aegis_dent_applies_drought_5() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, Some(5));
        app.apply_damage(t, 2, DamageRoute::Projectile);
        let d = app.world.get::<&AegisDrought>(t).unwrap();
        assert_eq!(d.duration, 5);
    }

    #[test]
    fn depleting_aegis_applies_drought_10() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, Some(5));
        app.apply_damage(t, 10, DamageRoute::Projectile);
        let d = app.world.get::<&AegisDrought>(t).unwrap();
        assert_eq!(d.duration, 10);
    }

    #[test]
    fn drought_never_shortens() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, Some(10));
        // First hit depletes: drought 10.
        app.apply_damage(t, 10, DamageRoute::Projectile);
        // Refill aegis by hand so a subsequent hit re-dents.
        if let Ok(mut a) = app.world.get::<&mut Aegis>(t) {
            a.current = 10;
        }
        app.apply_damage(t, 1, DamageRoute::Projectile);
        let d = app.world.get::<&AegisDrought>(t).unwrap();
        assert_eq!(d.duration, 10, "new 5 should not shorten active 10");
    }

    #[test]
    fn contact_does_not_apply_drought() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, Some(5));
        app.apply_damage(t, 5, DamageRoute::Contact);
        assert!(app.world.get::<&AegisDrought>(t).is_err());
    }

    #[test]
    fn systemic_does_not_apply_drought() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, Some(5));
        app.apply_damage(t, 3, DamageRoute::Systemic);
        assert!(app.world.get::<&AegisDrought>(t).is_err());
    }

    #[test]
    fn projectile_without_aegis_falls_back_to_contact_math() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 2, None);
        let out = app.apply_damage(t, 5, DamageRoute::Projectile);
        assert_eq!(out.aegis_damage, 0);
        assert_eq!(out.hp_damage, 3);
    }
}
