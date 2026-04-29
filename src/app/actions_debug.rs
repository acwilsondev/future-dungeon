use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;

impl App {
    pub fn handle_debug_console_input(&mut self, action: Action) {
        match action {
            Action::TypeChar(c) => {
                self.debug_console_buffer.push(c);
            }
            Action::Backspace => {
                self.debug_console_buffer.pop();
            }
            Action::SubmitCommand => {
                let command = self.debug_console_buffer.clone();
                self.execute_debug_command(&command);
                self.debug_console_buffer.clear();
                if self.state == RunState::ShowDebugConsole {
                    self.state = RunState::AwaitingInput;
                }
            }
            Action::ToggleDebugConsole | Action::CloseMenu => {
                self.state = RunState::AwaitingInput;
            }
            _ => {}
        }
    }

    fn execute_debug_command(&mut self, command: &str) {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "help" => {
                self.log.push(
                    "Debug Commands: spawn [name], teleport [lvl], heal, reveal, levelup, god, reload_content"
                        .to_string(),
                );
            }
            "reload_content" => {
                match crate::content::Content::load_from_dir(std::path::Path::new("content/")) {
                    Ok(new_content) => {
                        self.content = new_content;
                        self.log
                            .push("Debug: Content reloaded from content/".to_string());
                    }
                    Err(e) => {
                        self.log
                            .push(format!("Debug: Content reload failed: {}", e));
                    }
                }
            }
            "spawn" => {
                if parts.len() > 1 {
                    let item_name = parts[1..].join(" ");
                    if let Some(item_raw) = self
                        .content
                        .items
                        .iter()
                        .find(|i| i.name == item_name)
                        .cloned()
                    {
                        let Some(player_id) = self.get_player_id() else {
                            return;
                        };
                        let Some(pos) = self.world.get::<&Position>(player_id).ok().map(|p| *p)
                        else {
                            return;
                        };
                        crate::spawner::spawn_item(&mut self.world, pos.x, pos.y, &item_raw);
                        self.log.push(format!("Debug: Spawned {}", item_name));
                    } else {
                        self.log
                            .push(format!("Debug: Item '{}' not found", item_name));
                    }
                }
            }
            "teleport" => {
                if parts.len() > 1 {
                    if let Ok(level) = parts[1].parse::<u16>() {
                        let dest = (level, self.current_branch);
                        self.go_to_level(dest);
                        self.log
                            .push(format!("Debug: Teleported to level {}", level));
                    }
                }
            }
            "heal" => {
                let Some(player_id) = self.get_player_id() else {
                    return;
                };
                if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                    stats.hp = stats.max_hp;
                    self.log.push("Debug: Healed player".to_string());
                }
                self.world.remove_one::<Poison>(player_id).ok();
                self.world.remove_one::<Confusion>(player_id).ok();
            }
            "reveal" => {
                for r in self.map.revealed.iter_mut() {
                    *r = true;
                }
                self.log.push("Debug: Revealed map".to_string());
            }
            "levelup" => {
                let Some(player_id) = self.get_player_id() else {
                    return;
                };
                let needed = if let Ok(exp) = self.world.get::<&Experience>(player_id) {
                    Some(exp.next_level_xp - exp.xp)
                } else {
                    None
                };
                if let Some(needed) = needed {
                    self.add_player_xp(needed);
                    self.log.push("Debug: Leveled up player".to_string());
                }
            }
            "god" => {
                self.god_mode = !self.god_mode;
                self.log
                    .push(format!("Debug: God mode set to {}", self.god_mode));
            }
            _ => {
                self.log
                    .push(format!("Debug: Unknown command '{}'", parts[0]));
            }
        }
    }
}
