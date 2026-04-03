use crate::app::{App, RunState};
use crate::actions::Action;
use crate::components::*;

impl App {
    pub fn handle_inventory_input(&mut self, action: Action) {
        match action {
            Action::CloseMenu | Action::OpenInventory => self.state = RunState::AwaitingInput,
            Action::MenuUp => if self.inventory_cursor > 0 { self.inventory_cursor -= 1; },
            Action::MenuDown => {
                let player_id = self.get_player_id().expect("Player not found during inventory browsing");
                let count = self.world.query::<(&InBackpack,)>().iter()
                    .filter(|(_, (backpack,))| backpack.owner == player_id).count();
                if count > 0 && self.inventory_cursor < count - 1 { self.inventory_cursor += 1; }
            },
            Action::MenuSelect => {
                let player_id = self.get_player_id().expect("Player not found during item selection");
                let item_to_use = self.world.query::<(&Item, &InBackpack)>()
                    .iter()
                    .filter(|(_, (_, backpack))| backpack.owner == player_id)
                    .nth(self.inventory_cursor)
                    .map(|(id, _)| id);
                
                if let Some(id) = item_to_use {
                    self.use_item(id);
                    self.inventory_cursor = 0;
                }
            }
            _ => {}
        }
    }

    pub fn handle_identify_input(&mut self, action: Action) {
        match action {
            Action::CloseMenu => self.state = RunState::AwaitingInput,
            Action::MenuUp => if self.inventory_cursor > 0 { self.inventory_cursor -= 1; },
            Action::MenuDown => {
                let player_id = self.get_player_id().expect("Player not found during identify browsing");
                let count = self.world.query::<(&InBackpack,)>().iter()
                    .filter(|(_, (backpack,))| backpack.owner == player_id).count();
                if count > 0 && self.inventory_cursor < count - 1 { self.inventory_cursor += 1; }
            },
            Action::MenuSelect => {
                let player_id = self.get_player_id().expect("Player not found during identify selection");
                let item_to_identify = self.world.query::<(&Item, &InBackpack)>()
                    .iter()
                    .filter(|(_, (_, backpack))| backpack.owner == player_id)
                    .nth(self.inventory_cursor)
                    .map(|(id, _)| id);
                
                if let Some(id) = item_to_identify {
                    let real_name = self.world.get::<&Name>(id).map(|n| n.0.clone()).unwrap_or("Item".to_string());
                    if !self.identified_items.contains(&real_name) {
                        self.identified_items.insert(real_name.clone());
                        self.log.push(format!("You identify the {}!", real_name));
                        
                        if let Some(scroll_id) = self.targeting_item {
                            self.world.despawn(scroll_id).expect("Failed to despawn identify scroll");
                        }
                        self.state = RunState::AwaitingInput;
                        self.targeting_item = None;
                        self.inventory_cursor = 0;
                    } else {
                        self.log.push("That item is already identified.".to_string());
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_targeting_input(&mut self, action: Action) {
        match action {
            Action::CloseMenu => self.state = RunState::AwaitingInput,
            Action::MovePlayer(dx, dy) => {
                let new_x = (self.targeting_cursor.0 as i16 + dx).clamp(0, self.map.width as i16 - 1) as u16;
                let new_y = (self.targeting_cursor.1 as i16 + dy).clamp(0, self.map.height as i16 - 1) as u16;
                self.targeting_cursor = (new_x, new_y);
            }
            Action::MenuSelect => self.fire_targeting_item(),
            _ => {}
        }
    }
}
