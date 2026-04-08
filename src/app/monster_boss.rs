use crate::app::App;
use crate::components::*;

impl App {
    pub fn process_boss_phases(&mut self, id: hecs::Entity) {
        let mut boss_actions = Vec::new();
        if let Ok(mut boss) = self.world.get::<&mut Boss>(id) {
            if let Ok(stats) = self.world.get::<&CombatStats>(id) {
                for phase in boss.phases.iter_mut() {
                    if !phase.triggered && stats.hp <= phase.hp_threshold {
                        phase.triggered = true;
                        boss_actions.push(phase.action);
                    }
                }
            }
        }

        for action in boss_actions {
            let boss_name = self
                .world
                .get::<&Name>(id)
                .map(|n| n.0.clone())
                .unwrap_or("Boss".to_string());
            match action {
                BossPhaseAction::SummonMinions => {
                    self.log
                        .push(format!("{} bellows: 'To my side, my children!'", boss_name));
                    let boss_pos = self.world.get::<&Position>(id).ok().map(|p| *p);
                    if let Some(pos) = boss_pos {
                        let minion_name = if boss_name.contains("Broodmother") {
                            "Spider"
                        } else {
                            "Goblin"
                        };
                        let minion_raw = self
                            .content
                            .monsters
                            .iter()
                            .find(|m| m.name == minion_name);
                        if let Some(minion_raw) = minion_raw {
                            for (dx, dy) in &[(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                                let (mx, my) = (
                                    (pos.x as i16 + dx).max(0) as u16,
                                    (pos.y as i16 + dy).max(0) as u16,
                                );
                                if !self.map.blocked[(my * self.map.width + mx) as usize] {
                                    crate::spawner::spawn_monster(
                                        &mut self.world,
                                        mx,
                                        my,
                                        minion_raw,
                                        self.dungeon_level,
                                    );
                                }
                            }
                        }
                    }
                }
                BossPhaseAction::Enrage => {
                    self.log
                        .push(format!("{} enters a bloodthirsty rage!", boss_name));
                    if let Ok(mut stats) = self.world.get::<&mut CombatStats>(id) {
                        stats.power += 4;
                        stats.defense += 2;
                    }
                }
            }
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
    fn test_boss_phase_trigger() {
        let mut app = setup_test_app();
        let boss = app.world.spawn((
            Boss {
                phases: vec![BossPhase {
                    hp_threshold: 10,
                    action: BossPhaseAction::Enrage,
                    triggered: false,
                }],
            },
            CombatStats { hp: 15, max_hp: 20, defense: 0, power: 5 },
            Name("Test Boss".to_string())
        ));

        // Not triggered yet
        app.process_boss_phases(boss);
        {
            let stats = app.world.get::<&CombatStats>(boss).unwrap();
            assert_eq!(stats.power, 5);
        }

        // Drop HP
        if let Ok(mut stats) = app.world.get::<&mut CombatStats>(boss) {
            stats.hp = 5;
        }

        app.process_boss_phases(boss);
        {
            let stats = app.world.get::<&CombatStats>(boss).unwrap();
            assert_eq!(stats.power, 9); // Enraged: +4
        }
    }

    #[test]
    fn test_boss_summon_minions() {
        let mut app = setup_test_app();
        // Load some dummy content for minions
        app.content.monsters.push(crate::content::RawMonster {
            name: "Goblin".to_string(),
            glyph: 'g',
            color: (0, 255, 0),
            hp: 5,
            defense: 0,
            power: 1,
            viewshed: 8,
            spawn_chance: 1.0,
            min_floor: 1,
            max_floor: 10,
            personality: Personality::Brave,
            faction: FactionKind::Orcs,
            xp_reward: 5,
            ranged: None,
            confusion: None,
            poison: None,
            is_boss: None,
            phases: None,
            guaranteed_loot: None,
            branches: None,
            biomes: None,
        });

        let boss = app.world.spawn((
            Boss {
                phases: vec![BossPhase {
                    hp_threshold: 10,
                    action: BossPhaseAction::SummonMinions,
                    triggered: false,
                }],
            },
            CombatStats { hp: 5, max_hp: 20, defense: 0, power: 5 },
            Position { x: 10, y: 10 },
            Name("Goblin King".to_string())
        ));

        app.process_boss_phases(boss);

        // Should have spawned 4 minions around (10,10)
        let monster_count = app.world.query::<&Monster>().iter().count();
        assert!(monster_count >= 4);
    }
}
