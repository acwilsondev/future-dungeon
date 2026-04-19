use crate::app::{App, RunState};
use crate::components::*;
use rand::Rng;

impl App {
    /// Begin the casting flow for a spell the player knows.
    /// Step 1 (Mana Check) + sets up Step 2 (Targeting).
    pub fn begin_cast(&mut self, spell: Spell) {
        let Some(player_id) = self.get_player_id() else {
            return;
        };

        // Step 1: Mana Check
        let has_mana = self
            .world
            .get::<&ManaPool>(player_id)
            .map(|p| p.has_mana_for(&spell.mana_cost))
            .unwrap_or(false);
        if !has_mana {
            self.log
                .push(format!("You do not have the mana for {}.", spell.title));
            self.state = RunState::AwaitingInput;
            return;
        }

        // Step 2: Targeting
        match spell.targeting.selection {
            TargetSelection::SelfCast => {
                let origin = self
                    .world
                    .get::<&Position>(player_id)
                    .map(|p| (p.x, p.y))
                    .unwrap_or((0, 0));
                self.finish_cast(spell, origin);
            }
            TargetSelection::Entity | TargetSelection::Location => {
                let origin = self
                    .world
                    .get::<&Position>(player_id)
                    .map(|p| (p.x, p.y))
                    .unwrap_or((0, 0));
                self.targeting_cursor = origin;
                self.casting_spell = Some(spell.clone());
                self.targeting_item = None;
                self.log
                    .push(format!("Select target for {}...", spell.title));
                self.state = RunState::ShowTargeting;
            }
        }
    }

    /// Called when player confirms a target during an active cast.
    pub fn confirm_cast_target(&mut self) {
        let Some(spell) = self.casting_spell.take() else {
            return;
        };
        let origin = self.targeting_cursor;

        // Validate range if present
        if let Some(range) = spell.targeting.range {
            if let Some(player_id) = self.get_player_id() {
                if let Ok(pos) = self.world.get::<&Position>(player_id) {
                    let dx = origin.0 as i32 - pos.x as i32;
                    let dy = origin.1 as i32 - pos.y as i32;
                    let dist = ((dx * dx + dy * dy) as f32).sqrt() as u32;
                    if dist > range {
                        self.log.push("Target is out of range.".to_string());
                        self.state = RunState::AwaitingInput;
                        return;
                    }
                }
            }
        }

        // For Entity selection, require a creature at origin
        if spell.targeting.selection == TargetSelection::Entity {
            let has_entity = self
                .world
                .query::<(&Position, &CombatStats)>()
                .iter()
                .any(|(_, (p, _))| p.x == origin.0 && p.y == origin.1);
            if !has_entity {
                self.log
                    .push("No valid target at that location.".to_string());
                self.state = RunState::AwaitingInput;
                return;
            }
        }

        self.finish_cast(spell, origin);
    }

    /// Step 3 (Pay Mana) + Step 4 (Apply Effects) + Step 5 (Cleanup).
    pub fn finish_cast(&mut self, spell: Spell, origin: (u16, u16)) {
        let Some(player_id) = self.get_player_id() else {
            return;
        };

        // Step 3: Pay mana
        if let Ok(mut pool) = self.world.get::<&mut ManaPool>(player_id) {
            pool.pay(&spell.mana_cost);
        }
        // Trigger ManaDrought if total current mana reached zero.
        let now_empty = self
            .world
            .get::<&ManaPool>(player_id)
            .map(|p| p.total_current() == 0)
            .unwrap_or(false);
        if now_empty {
            self.world
                .insert_one(player_id, ManaDrought { duration: 5 })
                .ok();
            self.log
                .push("You are overcome by mana drought!".to_string());
        }

        self.log.push(format!("You cast {}!", spell.title));

        // Step 4: Apply effects
        let cha_mod = self
            .world
            .get::<&Attributes>(player_id)
            .map(|a| Attributes::get_modifier(a.charisma))
            .unwrap_or(0);
        let save_dc = 10 + spell.level as i32 + cha_mod;

        for instr in &spell.instructions {
            let affected = self.resolve_affected_entities(instr, origin);
            for target in affected {
                self.apply_effect(player_id, target, instr, save_dc);
            }
        }

        // Cleanup dead monsters
        self.cleanup_after_spell();

        // Step 5: Cleanup / end turn
        self.state = RunState::MonsterTurn;
    }

    fn resolve_affected_entities(
        &self,
        instr: &EffectInstruction,
        origin: (u16, u16),
    ) -> Vec<hecs::Entity> {
        let mut out = Vec::new();
        let radius = instr.radius.unwrap_or(0) as i32;
        for (id, pos) in self.world.query::<&Position>().iter() {
            let dx = pos.x as i32 - origin.0 as i32;
            let dy = pos.y as i32 - origin.1 as i32;
            let hit = match instr.shape {
                EffectShape::Point => dx == 0 && dy == 0,
                EffectShape::Circle => {
                    let dist_sq = dx * dx + dy * dy;
                    dist_sq <= radius * radius
                }
            };
            if hit {
                out.push(id);
            }
        }
        out
    }

    fn apply_effect(
        &mut self,
        caster: hecs::Entity,
        target: hecs::Entity,
        instr: &EffectInstruction,
        save_dc: i32,
    ) {
        // Application save
        if let Some(attr) = instr.application_save {
            if self.make_saving_throw(target, save_dc, attr.to_saving_throw_kind()) {
                let name = self.get_entity_name(target);
                self.log.push(format!("{} resists the effect!", name));
                return;
            }
        }

        match instr.opcode {
            EffectOpCode::DealDamage => {
                let dmg = instr
                    .magnitude
                    .as_ref()
                    .map(|d| d.roll(&mut self.rng))
                    .unwrap_or(0);
                self.apply_spell_damage(caster, target, dmg);
            }
            EffectOpCode::Heal => {
                let amt = instr
                    .magnitude
                    .as_ref()
                    .map(|d| d.roll(&mut self.rng))
                    .unwrap_or(0);
                if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target) {
                    stats.hp = (stats.hp + amt).min(stats.max_hp);
                }
            }
            EffectOpCode::GrantStatus => {
                if let EffectMetadata::Status(ref st) = instr.metadata {
                    self.grant_status(target, st);
                }
            }
            EffectOpCode::RemoveStatus => {
                if let EffectMetadata::RemoveStatus(ref name) = instr.metadata {
                    self.remove_status(target, name);
                }
            }
            EffectOpCode::Push | EffectOpCode::Teleport | EffectOpCode::CreateEntity => {
                // Scaffolded opcodes — not yet fully implemented.
                self.log.push(format!(
                    "[{:?}] effect is not yet implemented.",
                    instr.opcode
                ));
            }
        }
    }

    fn apply_spell_damage(&mut self, caster: hecs::Entity, target: hecs::Entity, damage: i32) {
        let is_player = self.world.get::<&Player>(target).is_ok();
        if is_player && self.god_mode {
            return;
        }
        if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target) {
            stats.hp -= damage;
            let name = if is_player {
                "You".to_string()
            } else {
                self.get_entity_name(target)
            };
            self.log.push(format!("{} takes {} damage.", name, damage));
            if stats.hp <= 0 && is_player {
                self.death = true;
                self.state = RunState::Dead;
            }
        }
        // Mark hostile if caster is player
        if self.world.get::<&Player>(caster).is_ok() && self.world.get::<&Monster>(target).is_ok() {
            let _ = self.world.insert_one(target, LastHitByPlayer);
            let _ = self.world.insert_one(target, AlertState::Aggressive);
        }
    }

    fn grant_status(&mut self, target: hecs::Entity, st: &BakedStatusEffect) {
        let duration = st.duration.unwrap_or(1) as i32;
        let magnitude = st
            .magnitude
            .as_ref()
            .map(|d| d.roll(&mut self.rng))
            .unwrap_or(0);
        let name = self.get_entity_name(target);
        match st.status_type.as_str() {
            "Confusion" => {
                let _ = self.world.insert_one(target, Confusion { turns: duration });
                self.log.push(format!("{} is confused.", name));
            }
            "Poisoned" | "DamagePoison" => {
                let _ = self.world.insert_one(
                    target,
                    Poison {
                        damage: magnitude.max(1),
                        turns: duration,
                    },
                );
                self.log.push(format!("{} is poisoned.", name));
            }
            "Mired" => {
                let _ = self.world.insert_one(
                    target,
                    Mired {
                        magnitude: magnitude.max(1),
                        duration: duration as u32,
                        recovery_save: st.recovery_save,
                    },
                );
                self.log.push(format!("{} is mired.", name));
            }
            "Armored" => {
                let amount = magnitude.max(0);
                if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target) {
                    stats.defense += amount;
                }
                let _ = self.world.insert_one(
                    target,
                    Armored {
                        magnitude: amount,
                        duration: duration as u32,
                        recovery_save: st.recovery_save,
                    },
                );
                self.log.push(format!("{} is armored.", name));
            }
            "ManaDrought" => {
                let _ = self.world.insert_one(
                    target,
                    ManaDrought {
                        duration: duration as u32,
                    },
                );
                self.log.push(format!("{} is in mana drought.", name));
            }
            other => {
                self.log
                    .push(format!("[{} status] not yet implemented.", other));
            }
        }
    }

    fn remove_status(&mut self, target: hecs::Entity, name: &str) {
        match name {
            "Confusion" => {
                self.world.remove_one::<Confusion>(target).ok();
            }
            "Poisoned" => {
                self.world.remove_one::<Poison>(target).ok();
            }
            "Mired" => {
                self.world.remove_one::<Mired>(target).ok();
            }
            "Armored" => {
                self.world.remove_one::<Armored>(target).ok();
            }
            "ManaDrought" => {
                self.world.remove_one::<ManaDrought>(target).ok();
            }
            _ => {}
        }
    }

    fn cleanup_after_spell(&mut self) {
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
                .unwrap_or_else(|_| "Monster".to_string());
            self.log.push(format!("{} dies!", name));
            if let Err(e) = self.world.despawn(id) {
                log::error!("Failed to despawn monster: {}", e);
            }
            self.monsters_killed += 1;
        }
        self.update_blocked_and_opaque();
        if total_xp > 0 {
            self.add_player_xp(total_xp);
        }
    }
}

/// Silence unused_imports — Rng is used transitively via roll().
#[allow(dead_code)]
fn _rng_probe<R: Rng>(_r: &mut R) {}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    fn setup() -> App {
        let mut app = App::new_test(42);
        app.world = World::new();
        app.map = crate::map::Map::new(80, 50);
        for t in app.map.tiles.iter_mut() {
            *t = crate::map::TileType::Floor;
        }
        app.update_blocked_and_opaque();
        app
    }

    fn default_attrs() -> Attributes {
        Attributes {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }

    fn firebolt_spell() -> Spell {
        Spell {
            title: "Firebolt".into(),
            description: "".into(),
            mana_cost: ManaCost {
                orange: 1,
                purple: 0,
            },
            level: 1,
            targeting: TargetSpec {
                range: Some(6),
                selection: TargetSelection::Entity,
            },
            instructions: vec![EffectInstruction {
                opcode: EffectOpCode::DealDamage,
                shape: EffectShape::Point,
                radius: None,
                application_save: None,
                magnitude: Some(Dice::flat(10)),
                metadata: EffectMetadata::Damage(DamageType::Fire),
            }],
        }
    }

    #[test]
    fn test_finish_cast_deals_damage_and_pays_mana() {
        let mut app = setup();
        let player = app.world.spawn((
            Player,
            Position { x: 5, y: 5 },
            default_attrs(),
            ManaPool {
                current_orange: 2,
                max_orange: 2,
                current_purple: 0,
                max_purple: 0,
            },
        ));
        let monster = app.world.spawn((
            Monster,
            Position { x: 7, y: 5 },
            Name("Goblin".into()),
            default_attrs(),
            CombatStats {
                hp: 20,
                max_hp: 20,
                defense: 0,
                power: 1,
            },
            Experience {
                level: 1,
                xp: 0,
                next_level_xp: 0,
                xp_reward: 5,
            },
        ));

        app.finish_cast(firebolt_spell(), (7, 5));
        let stats = app.world.get::<&CombatStats>(monster).unwrap();
        assert_eq!(stats.hp, 10);
        let pool = app.world.get::<&ManaPool>(player).unwrap();
        assert_eq!(pool.current_orange, 1);
    }

    #[test]
    fn test_cast_triggers_mana_drought_when_last_mana_spent() {
        let mut app = setup();
        let player = app.world.spawn((
            Player,
            Position { x: 5, y: 5 },
            default_attrs(),
            ManaPool {
                current_orange: 1,
                max_orange: 2,
                current_purple: 0,
                max_purple: 0,
            },
        ));
        let _monster = app.world.spawn((
            Monster,
            Position { x: 5, y: 5 },
            Name("Dummy".into()),
            default_attrs(),
            CombatStats {
                hp: 100,
                max_hp: 100,
                defense: 0,
                power: 1,
            },
            Experience {
                level: 1,
                xp: 0,
                next_level_xp: 0,
                xp_reward: 0,
            },
        ));

        app.finish_cast(firebolt_spell(), (5, 5));
        let drought = app.world.get::<&ManaDrought>(player).unwrap();
        assert_eq!(drought.duration, 5);
    }

    #[test]
    fn test_begin_cast_aborts_without_mana() {
        let mut app = setup();
        app.world.spawn((
            Player,
            Position { x: 5, y: 5 },
            default_attrs(),
            ManaPool::default(),
        ));
        app.state = RunState::ShowSpells;
        app.begin_cast(firebolt_spell());
        assert_eq!(app.state, RunState::AwaitingInput);
    }

    #[test]
    fn test_self_cast_skips_targeting() {
        let mut app = setup();
        let player = app.world.spawn((
            Player,
            Position { x: 5, y: 5 },
            default_attrs(),
            CombatStats {
                hp: 5,
                max_hp: 20,
                defense: 0,
                power: 1,
            },
            ManaPool {
                current_orange: 2,
                max_orange: 2,
                current_purple: 0,
                max_purple: 0,
            },
        ));
        let heal = Spell {
            title: "Heal".into(),
            description: "".into(),
            mana_cost: ManaCost {
                orange: 1,
                purple: 0,
            },
            level: 1,
            targeting: TargetSpec {
                range: None,
                selection: TargetSelection::SelfCast,
            },
            instructions: vec![EffectInstruction {
                opcode: EffectOpCode::Heal,
                shape: EffectShape::Point,
                radius: None,
                application_save: None,
                magnitude: Some(Dice::flat(5)),
                metadata: EffectMetadata::None,
            }],
        };
        app.begin_cast(heal);
        let stats = app.world.get::<&CombatStats>(player).unwrap();
        assert_eq!(stats.hp, 10);
        assert_eq!(app.state, RunState::MonsterTurn);
    }
}
