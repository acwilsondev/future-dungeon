use crate::app::App;
use crate::components::*;
use rand::seq::IteratorRandom;

impl App {
    /// Tick mana regen for all entities with a ManaPool.
    /// - If the entity has ManaDrought, decrement its duration (removing at 0)
    ///   and skip regen this turn.
    /// - Otherwise, grant +1 current mana to a randomly chosen color with the
    ///   greatest deficit (max - current). If no deficit exists, do nothing.
    pub fn tick_mana_regen(&mut self) {
        let mut drought_to_remove = Vec::new();
        let mut drought_ids = Vec::new();
        let mut regen_ids: Vec<hecs::Entity> = Vec::new();

        for (id, _pool) in self.world.query::<&ManaPool>().iter() {
            if self.world.get::<&ManaDrought>(id).is_ok() {
                drought_ids.push(id);
            } else {
                regen_ids.push(id);
            }
        }

        for id in drought_ids {
            if let Ok(mut drought) = self.world.get::<&mut ManaDrought>(id) {
                drought.duration = drought.duration.saturating_sub(1);
                if drought.duration == 0 {
                    drought_to_remove.push(id);
                }
            }
        }
        for id in &drought_to_remove {
            self.world.remove_one::<ManaDrought>(*id).ok();
            if self.world.get::<&Player>(*id).is_ok() {
                self.log.push("Your mana drought lifts.".to_string());
            }
        }

        for id in regen_ids {
            let (orange_deficit, purple_deficit) = {
                let Ok(pool) = self.world.get::<&ManaPool>(id) else {
                    continue;
                };
                (
                    pool.max_orange.saturating_sub(pool.current_orange),
                    pool.max_purple.saturating_sub(pool.current_purple),
                )
            };
            let max_deficit = orange_deficit.max(purple_deficit);
            if max_deficit == 0 {
                continue;
            }
            let mut candidates = Vec::new();
            if orange_deficit == max_deficit {
                candidates.push(ManaColor::Orange);
            }
            if purple_deficit == max_deficit {
                candidates.push(ManaColor::Purple);
            }
            let Some(color) = candidates.into_iter().choose(&mut self.rng) else {
                continue;
            };
            if let Ok(mut pool) = self.world.get::<&mut ManaPool>(id) {
                match color {
                    ManaColor::Orange => pool.current_orange += 1,
                    ManaColor::Purple => pool.current_purple += 1,
                }
            }
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

    #[test]
    fn test_regen_fills_deficit() {
        let mut app = setup();
        let player = app.world.spawn((
            Player,
            ManaPool {
                current_orange: 0,
                max_orange: 2,
                current_purple: 0,
                max_purple: 0,
            },
        ));
        app.tick_mana_regen();
        let pool = app.world.get::<&ManaPool>(player).unwrap();
        assert_eq!(pool.current_orange, 1);
    }

    #[test]
    fn test_regen_chooses_greatest_deficit() {
        let mut app = setup();
        let player = app.world.spawn((
            Player,
            ManaPool {
                current_orange: 1,
                max_orange: 3,
                current_purple: 1,
                max_purple: 2,
            },
        ));
        // Orange deficit=2, purple deficit=1. Should regen orange.
        app.tick_mana_regen();
        let pool = app.world.get::<&ManaPool>(player).unwrap();
        assert_eq!(pool.current_orange, 2);
        assert_eq!(pool.current_purple, 1);
    }

    #[test]
    fn test_regen_skipped_during_drought() {
        let mut app = setup();
        let player = app.world.spawn((
            Player,
            ManaPool {
                current_orange: 0,
                max_orange: 2,
                current_purple: 0,
                max_purple: 0,
            },
            ManaDrought { duration: 3 },
        ));
        app.tick_mana_regen();
        let pool = app.world.get::<&ManaPool>(player).unwrap();
        assert_eq!(pool.current_orange, 0);
        let d = app.world.get::<&ManaDrought>(player).unwrap();
        assert_eq!(d.duration, 2);
    }

    #[test]
    fn test_drought_expires() {
        let mut app = setup();
        let player = app
            .world
            .spawn((Player, ManaPool::default(), ManaDrought { duration: 1 }));
        app.tick_mana_regen();
        assert!(app.world.get::<&ManaDrought>(player).is_err());
    }

    #[test]
    fn test_regen_no_deficit() {
        let mut app = setup();
        let player = app.world.spawn((
            Player,
            ManaPool {
                current_orange: 2,
                max_orange: 2,
                current_purple: 1,
                max_purple: 1,
            },
        ));
        app.tick_mana_regen();
        let pool = app.world.get::<&ManaPool>(player).unwrap();
        assert_eq!(pool.current_orange, 2);
        assert_eq!(pool.current_purple, 1);
    }
}
