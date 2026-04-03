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
                            .find(|m| m.name == minion_name)
                            .expect("Minion not found");
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
