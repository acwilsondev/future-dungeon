use crate::app::{App, Branch, RunState};
use crate::components::*;
use rand::Rng;

impl App {
    pub fn monster_turn(&mut self) {
        self.on_turn_tick();
        if self.state == RunState::Dead {
            return;
        }

        let player_id = self
            .get_player_id()
            .expect("Player not found in monster_turn");

        // Handle Speed status
        if self.world.get::<&Speed>(player_id).is_ok() && self.speed_toggle {
            self.speed_toggle = false;
            self.state = RunState::AwaitingInput;
            self.log
                .push("You move with supernatural speed!".to_string());
            return;
        }
        self.speed_toggle = true;

        let mut actions = Vec::new();
        let mut actors: Vec<hecs::Entity> = self
            .world
            .query::<&Monster>()
            .iter()
            .map(|(id, _)| id)
            .collect();
        for (id, _) in self.world.query::<&Merchant>().iter() {
            actors.push(id);
        }

        // 1. Plan actions
        for id in actors {
            self.process_boss_phases(id);
            self.update_monster_perception(id, player_id);
            if let Some(action) = self.calculate_monster_action(id, player_id) {
                actions.push((id, action));
            }
        }

        // 2. Execute actions
        let mut occupied_positions: std::collections::HashSet<(u16, u16)> = self
            .world
            .query::<(&Position, &Monster)>()
            .iter()
            .map(|(_, (p, _))| (p.x, p.y))
            .collect();
        for (_, (p, _)) in self.world.query::<(&Position, &Merchant)>().iter() {
            occupied_positions.insert((p.x, p.y));
        }
        if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
            occupied_positions.insert((p_pos.x, p_pos.y));
        }

        for (id, action) in actions {
            self.execute_monster_action(id, action, player_id, &mut occupied_positions);
        }

        // 3. Cleanup
        self.cleanup_dead_entities();

        // 4. Special Wisp movement (non-combat, random)
        let mut wisp_moves = Vec::new();
        for (id, _) in self.world.query::<&Wisp>().iter() {
            let mut rng = rand::thread_rng();
            wisp_moves.push((id, rng.gen_range(-1..=1), rng.gen_range(-1..=1)));
        }

        for (id, dx, dy) in wisp_moves {
            let (new_x, new_y) = {
                if let Ok(pos) = self.world.get::<&Position>(id) {
                    (
                        (pos.x as i16 + dx).clamp(0, self.map.width as i16 - 1) as u16,
                        (pos.y as i16 + dy).clamp(0, self.map.height as i16 - 1) as u16,
                    )
                } else {
                    continue;
                }
            };
            if !self.map.blocked[new_y as usize * self.map.width as usize + new_x as usize] {
                if let Ok(mut pos) = self.world.get::<&mut Position>(id) {
                    pos.x = new_x;
                    pos.y = new_y;
                }
            }
        }

        self.update_blocked_and_opaque();

        if self.state != RunState::Dead && self.state != RunState::LevelUp {
            if self.current_branch == Branch::Vaults && self.turn_count.is_multiple_of(2) {
                // In Vaults, player is slower, so monsters get a double turn occasionally
                self.monster_turn();
            } else {
                self.state = RunState::AwaitingInput;
            }
        }
    }
}
