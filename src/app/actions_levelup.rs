use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;

impl App {
    pub fn handle_level_up_input(&mut self, action: Action) {
        match action {
            Action::MenuUp => {
                if self.level_up_cursor > 0 {
                    self.level_up_cursor -= 1;
                }
            }
            Action::MenuDown => {
                if self.level_up_cursor < 3 {
                    self.level_up_cursor += 1;
                }
            }
            Action::MenuSelect => {
                if let Some(player_id) = self.get_player_id() {
                    match self.level_up_cursor {
                    0 => {
                        if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                            stats.max_hp += 10;
                            stats.hp += 10;
                        }
                        if let Ok(mut perks) = self.world.get::<&mut Perks>(player_id) {
                            perks.traits.push(Perk::Toughness);
                        }
                        self.log
                            .push("You chose Toughness! Max HP increased.".to_string());
                    }
                    1 => {
                        if let Ok(mut viewshed) = self.world.get::<&mut Viewshed>(player_id) {
                            viewshed.visible_tiles += 2;
                        }
                        if let Ok(mut perks) = self.world.get::<&mut Perks>(player_id) {
                            perks.traits.push(Perk::EagleEye);
                        }
                        self.log
                            .push("You chose Eagle Eye! FOV increased.".to_string());
                    }
                    2 => {
                        if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                            stats.power += 2;
                        }
                        if let Ok(mut perks) = self.world.get::<&mut Perks>(player_id) {
                            perks.traits.push(Perk::Strong);
                        }
                        self.log
                            .push("You chose Strong! Power increased.".to_string());
                    }
                    3 => {
                        if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                            stats.defense += 1;
                        }
                        if let Ok(mut perks) = self.world.get::<&mut Perks>(player_id) {
                            perks.traits.push(Perk::ThickSkin);
                        }
                        self.log
                            .push("You chose Thick Skin! Defense increased.".to_string());
                    }
                    _ => {}
                    }
                }
                self.state = RunState::MonsterTurn;
            }
            _ => {}
        }
    }
}
