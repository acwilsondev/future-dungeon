use crate::app::{App, RunState};
use crate::actions::Action;
use crate::components::*;

impl App {
    pub fn handle_alchemy_input(&mut self, action: Action) {
        match action {
            Action::CloseMenu => self.state = RunState::AwaitingInput,
            Action::MenuUp => if self.inventory_cursor > 0 { self.inventory_cursor -= 1; },
            Action::MenuDown => {
                let player_id = self.get_player_id().expect("Player not found during alchemy browsing");
                let count = self.world.query::<(&InBackpack,)>().iter()
                    .filter(|(_, (backpack,))| backpack.owner == player_id).count();
                if count > 0 && self.inventory_cursor < count - 1 { self.inventory_cursor += 1; }
            },
            Action::MenuSelect => {
                let player_id = self.get_player_id().expect("Player not found during alchemy selection");
                let item_to_select = self.world.query::<(&Item, &InBackpack)>()
                    .iter()
                    .filter(|(_, (_, backpack))| backpack.owner == player_id)
                    .nth(self.inventory_cursor)
                    .map(|(id, _)| id);
                
                if let Some(id) = item_to_select {
                    if self.alchemy_selection.contains(&id) {
                        self.alchemy_selection.retain(|&x| x != id);
                        self.log.push("Item deselected.".to_string());
                    } else {
                        self.alchemy_selection.push(id);
                        if self.alchemy_selection.len() == 1 {
                            self.log.push("First item selected. Choose a second to combine.".to_string());
                        } else if self.alchemy_selection.len() == 2 {
                            let item1 = self.alchemy_selection[0];
                            let item2 = self.alchemy_selection[1];
                            
                            let p1_heal = self.world.get::<&Potion>(item1).ok().map(|p| p.heal_amount);
                            let p2_heal = self.world.get::<&Potion>(item2).ok().map(|p| p.heal_amount);
                            
                            let n1 = self.world.get::<&Name>(item1).map(|n| n.0.clone()).unwrap_or_default();
                            let n2 = self.world.get::<&Name>(item2).map(|n| n.0.clone()).unwrap_or_default();

                            if (n1 == "Potion of Strength" && n2 == "Potion of Speed") || (n1 == "Potion of Speed" && n2 == "Potion of Strength") {
                                self.world.despawn(item1).expect("Failed to despawn alchemy item 1");
                                self.world.despawn(item2).expect("Failed to despawn alchemy item 2");
                                
                                let _new_potion = self.world.spawn((
                                    Renderable { glyph: '!', fg: ratatui::prelude::Color::Rgb(255, 215, 0) },
                                    RenderOrder::Item,
                                    Item,
                                    Name("Potion of Heroism".to_string()),
                                    Strength { amount: 5, turns: 20 },
                                    Speed { turns: 20 },
                                    Consumable,
                                    InBackpack { owner: player_id }
                                ));
                                self.identified_items.insert("Potion of Heroism".to_string());
                                self.log.push("You created a Potion of Heroism! It grants both Strength and Speed.".to_string());
                                self.state = RunState::AwaitingInput;
                            } else if let (Some(heal1), Some(heal2)) = (p1_heal, p2_heal) {
                                let new_heal = ((heal1 + heal2) as f32 * 1.5) as i32;
                                
                                self.world.despawn(item1).expect("Failed to despawn alchemy item 1");
                                self.world.despawn(item2).expect("Failed to despawn alchemy item 2");
                                
                                let _new_potion = self.world.spawn((
                                    Renderable { glyph: '!', fg: ratatui::prelude::Color::Rgb(255, 255, 255) },
                                    RenderOrder::Item,
                                    Item,
                                    Name("Greater Potion".to_string()),
                                    Potion { heal_amount: new_heal },
                                    Consumable,
                                    InBackpack { owner: player_id }
                                ));
                                self.identify_item(_new_potion);
                                
                                self.log.push(format!("You combined the potions into a Greater Potion (Heals {})!", new_heal));
                                self.state = RunState::AwaitingInput;
                            } else {
                                self.log.push("You can only combine potions!".to_string());
                                self.alchemy_selection.clear();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
