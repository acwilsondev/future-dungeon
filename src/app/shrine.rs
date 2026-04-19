use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;
use rand::Rng;

impl App {
    /// Begin a shrine interaction (player walked onto a shrine tile and confirms).
    /// Sets up `shrine_entity` and transitions to `RunState::ShowShrine` for confirmation.
    #[allow(dead_code)]
    pub fn begin_shrine_interaction(&mut self, shrine_id: hecs::Entity) {
        let Some(player_id) = self.get_player_id() else {
            return;
        };

        // Already tried?
        if let Ok(shrine) = self.world.get::<&Shrine>(shrine_id) {
            if shrine.tried {
                self.log
                    .push("The shrine is silent. Peace be with you.".to_string());
                return;
            }
        } else {
            return;
        }

        // Already at max mana cap?
        let at_cap = self
            .world
            .get::<&ManaPool>(player_id)
            .map(|p| p.total_max() >= ManaPool::CAP)
            .unwrap_or(false);
        if at_cap {
            self.log
                .push("You already command the full measure of mana.".to_string());
            return;
        }

        self.shrine_entity = Some(shrine_id);
        self.state = RunState::ShowShrine;
    }

    pub fn handle_shrine_input(&mut self, action: Action) {
        match action {
            Action::Confirm => self.attempt_shrine(),
            Action::Decline => {
                self.shrine_entity = None;
                self.state = RunState::AwaitingInput;
            }
            _ => {}
        }
    }

    fn attempt_shrine(&mut self) {
        let Some(shrine_id) = self.shrine_entity.take() else {
            self.state = RunState::AwaitingInput;
            return;
        };
        let Some(player_id) = self.get_player_id() else {
            self.state = RunState::AwaitingInput;
            return;
        };

        let shrine_color = match self.world.get::<&Shrine>(shrine_id) {
            Ok(s) => s.color,
            Err(_) => {
                self.state = RunState::AwaitingInput;
                return;
            }
        };

        let total_max = self
            .world
            .get::<&ManaPool>(player_id)
            .map(|p| p.total_max())
            .unwrap_or(0);
        let dc = 10 + total_max as i32;
        let cha_mod = self
            .world
            .get::<&Attributes>(player_id)
            .map(|a| Attributes::get_modifier(a.charisma))
            .unwrap_or(0);

        let roll = self.rng.random_range(1..=20);
        let success = roll + cha_mod >= dc;

        if let Ok(mut shrine) = self.world.get::<&mut Shrine>(shrine_id) {
            shrine.tried = true;
        }

        if success {
            let mut pool = self
                .world
                .get::<&mut ManaPool>(player_id)
                .ok()
                .map(|p| *p)
                .unwrap_or_default();
            let had_pool = self.world.get::<&ManaPool>(player_id).is_ok();
            let increased = pool.increase_max(shrine_color);
            if had_pool {
                if let Ok(mut p) = self.world.get::<&mut ManaPool>(player_id) {
                    *p = pool;
                }
            } else {
                let _ = self.world.insert_one(player_id, pool);
            }
            if increased {
                self.log.push(format!(
                    "The {} shrine grants you its blessing! Max {} mana increased.",
                    shrine_color.order_name(),
                    shrine_color.order_name()
                ));
            } else {
                self.log.push(format!(
                    "The {} shrine acknowledges you, but you can hold no more mana.",
                    shrine_color.order_name()
                ));
            }
        } else {
            self.log.push(format!(
                "The {} shrine finds you unworthy. (Roll: {}+{} vs DC:{})",
                shrine_color.order_name(),
                roll,
                cha_mod,
                dc
            ));
        }

        self.state = RunState::AwaitingInput;
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

    fn attrs_cha(cha: i32) -> Attributes {
        Attributes {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: cha,
        }
    }

    #[test]
    fn test_begin_shrine_blocks_when_tried() {
        let mut app = setup();
        app.state = RunState::AwaitingInput;
        app.world
            .spawn((Player, attrs_cha(10), ManaPool::default()));
        let shrine = app.world.spawn((Shrine {
            color: ManaColor::Orange,
            tried: true,
        },));
        app.begin_shrine_interaction(shrine);
        assert_eq!(app.state, RunState::AwaitingInput);
        assert!(app.shrine_entity.is_none());
    }

    #[test]
    fn test_begin_shrine_blocks_at_max_cap() {
        let mut app = setup();
        app.state = RunState::AwaitingInput;
        app.world.spawn((
            Player,
            attrs_cha(10),
            ManaPool {
                current_orange: 5,
                max_orange: 5,
                current_purple: 0,
                max_purple: 0,
            },
        ));
        let shrine = app.world.spawn((Shrine {
            color: ManaColor::Orange,
            tried: false,
        },));
        app.begin_shrine_interaction(shrine);
        assert_eq!(app.state, RunState::AwaitingInput);
        assert!(app.shrine_entity.is_none());
    }

    #[test]
    fn test_decline_cancels() {
        let mut app = setup();
        app.state = RunState::AwaitingInput;
        app.world
            .spawn((Player, attrs_cha(10), ManaPool::default()));
        let shrine = app.world.spawn((Shrine {
            color: ManaColor::Orange,
            tried: false,
        },));
        app.begin_shrine_interaction(shrine);
        assert_eq!(app.state, RunState::ShowShrine);
        app.handle_shrine_input(Action::Decline);
        assert_eq!(app.state, RunState::AwaitingInput);
    }

    #[test]
    fn test_confirm_sets_tried_regardless() {
        let mut app = setup();
        app.state = RunState::AwaitingInput;
        app.world
            .spawn((Player, attrs_cha(10), ManaPool::default()));
        let shrine = app.world.spawn((Shrine {
            color: ManaColor::Orange,
            tried: false,
        },));
        app.begin_shrine_interaction(shrine);
        app.handle_shrine_input(Action::Confirm);
        let s = app.world.get::<&Shrine>(shrine).unwrap();
        assert!(s.tried);
        assert_eq!(app.state, RunState::AwaitingInput);
    }
}
