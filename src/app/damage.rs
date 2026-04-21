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

    /// Apply a Tachyonic-modifier projectile hit. Aegis absorbs at 2× rate
    /// (each HP of aegis soaks twice the raw damage); any overflow passes
    /// through AV to HP as normal. Falls through to the regular projectile
    /// path when the target has no aegis.
    pub fn apply_projectile_tachyonic(&mut self, target: hecs::Entity, raw: i32) -> DamageOutcome {
        if raw <= 0 {
            return DamageOutcome::default();
        }

        let is_player = self.world.get::<&Player>(target).is_ok();
        if is_player && self.god_mode {
            return DamageOutcome::default();
        }

        let aegis_before = self
            .world
            .get::<&Aegis>(target)
            .ok()
            .map(|a| a.current)
            .unwrap_or(0);

        if aegis_before == 0 {
            return self.apply_damage(target, raw, DamageRoute::Projectile);
        }

        let mut out = DamageOutcome::default();
        let aegis_consumed = (raw * 2).min(aegis_before);
        // Raw cost for the absorbed portion (ceil div by 2).
        let raw_absorbed = (aegis_consumed + 1) / 2;
        if let Ok(mut a) = self.world.get::<&mut Aegis>(target) {
            a.current -= aegis_consumed;
            out.aegis_damage = aegis_consumed;
            out.aegis_depleted = a.current == 0;
        }

        let overflow = raw - raw_absorbed;
        if overflow > 0 {
            let av = self.get_target_av(target);
            let hp_damage = (overflow - av).max(1);
            if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target) {
                stats.hp -= hp_damage;
                out.hp_damage = hp_damage;
            }
        }

        let drought_turns = if out.aegis_depleted { 10 } else { 5 };
        self.apply_aegis_drought(target, drought_turns);

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

    /// Add `new_stacks` Shredded stacks to `target` (soft-capped at
    /// `SHREDDED_CAP`). Always resets the decay timer to
    /// `SHREDDED_DECAY_INTERVAL`, even when stacks are already at the cap.
    pub fn apply_shredded(&mut self, target: hecs::Entity, new_stacks: u32) {
        if let Ok(mut s) = self.world.get::<&mut Shredded>(target) {
            s.stacks = (s.stacks + new_stacks).min(SHREDDED_CAP);
            s.decay_timer = SHREDDED_DECAY_INTERVAL;
            return;
        }
        let _ = self.world.insert_one(
            target,
            Shredded {
                stacks: new_stacks.min(SHREDDED_CAP),
                decay_timer: SHREDDED_DECAY_INTERVAL,
            },
        );
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

    #[test]
    fn tachyonic_doubles_aegis_soak() {
        // 3 raw vs 10 aegis → aegis drops 6, overflow 0.
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, Some(10));
        let out = app.apply_projectile_tachyonic(t, 3);
        assert_eq!(out.aegis_damage, 6);
        assert_eq!(out.hp_damage, 0);
        assert_eq!(app.world.get::<&Aegis>(t).unwrap().current, 4);
    }

    #[test]
    fn tachyonic_partial_absorb_overflows_to_hp() {
        // 5 raw vs 6 aegis: aegis drops 6 (clamped), raw_absorbed = 3,
        // overflow 2 → AV 0 → HP loses 2.
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, Some(6));
        let out = app.apply_projectile_tachyonic(t, 5);
        assert_eq!(out.aegis_damage, 6);
        assert_eq!(out.hp_damage, 2);
        let stats = app.world.get::<&CombatStats>(t).unwrap();
        assert_eq!(stats.hp, 8);
        assert!(out.aegis_depleted);
    }

    #[test]
    fn tachyonic_no_aegis_behaves_as_normal_projectile() {
        // 4 raw, AV 1, no aegis → HP loses 3.
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 1, None);
        let out = app.apply_projectile_tachyonic(t, 4);
        assert_eq!(out.aegis_damage, 0);
        assert_eq!(out.hp_damage, 3);
    }

    #[test]
    fn apply_shredded_inserts_with_timer() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, None);
        app.apply_shredded(t, 1);
        let s = app.world.get::<&Shredded>(t).unwrap();
        assert_eq!(s.stacks, 1);
        assert_eq!(s.decay_timer, SHREDDED_DECAY_INTERVAL);
    }

    #[test]
    fn apply_shredded_stacks_and_resets_timer() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, None);
        app.apply_shredded(t, 1);
        // Manually age the timer down to 1.
        app.world.get::<&mut Shredded>(t).unwrap().decay_timer = 1;
        app.apply_shredded(t, 1);
        let s = app.world.get::<&Shredded>(t).unwrap();
        assert_eq!(s.stacks, 2);
        assert_eq!(
            s.decay_timer, SHREDDED_DECAY_INTERVAL,
            "timer resets on re-apply"
        );
    }

    #[test]
    fn apply_shredded_soft_caps_at_ten() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, None);
        for _ in 0..15 {
            app.apply_shredded(t, 1);
        }
        let s = app.world.get::<&Shredded>(t).unwrap();
        assert_eq!(s.stacks, SHREDDED_CAP);
    }

    #[test]
    fn shredded_reset_at_cap_refreshes_timer() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 0, None);
        for _ in 0..SHREDDED_CAP {
            app.apply_shredded(t, 1);
        }
        app.world.get::<&mut Shredded>(t).unwrap().decay_timer = 1;
        app.apply_shredded(t, 1);
        let s = app.world.get::<&Shredded>(t).unwrap();
        assert_eq!(s.stacks, SHREDDED_CAP);
        assert_eq!(s.decay_timer, SHREDDED_DECAY_INTERVAL);
    }

    #[test]
    fn shredded_reduces_effective_av() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 4, None);
        app.apply_shredded(t, 2);
        assert_eq!(app.get_target_av(t), 2);
    }

    #[test]
    fn shredded_av_floors_at_zero() {
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 1, None);
        app.apply_shredded(t, 5);
        assert_eq!(app.get_target_av(t), 0, "AV floors at 0, never negative");
    }

    #[test]
    fn shredded_does_not_reduce_aegis_soak() {
        // With aegis up, shredding has no effect on the soaked portion.
        let mut app = setup();
        let t = spawn_target(&mut app, 10, 2, Some(5));
        app.apply_shredded(t, 3);
        // 3 raw projectile, all absorbed by aegis — HP untouched.
        let out = app.apply_damage(t, 3, DamageRoute::Projectile);
        assert_eq!(out.aegis_damage, 3);
        assert_eq!(out.hp_damage, 0);
    }
}
