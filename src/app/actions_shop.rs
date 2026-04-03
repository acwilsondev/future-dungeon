use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;

impl App {
    pub fn handle_shop_input(&mut self, action: Action) {
        match action {
            Action::CloseMenu => self.state = RunState::AwaitingInput,
            Action::ToggleShopMode => {
                self.shop_mode = (self.shop_mode + 1) % 2;
                self.shop_cursor = 0;
            }
            Action::MenuUp => {
                if self.shop_cursor > 0 {
                    self.shop_cursor -= 1;
                }
            }
            Action::MenuDown => {
                let player_id = self
                    .get_player_id()
                    .expect("Player not found during shop browsing");
                let count = if self.shop_mode == 0 {
                    if let Some(m_id) = self.active_merchant {
                        self.world
                            .query::<(&InBackpack,)>()
                            .iter()
                            .filter(|(_, (backpack,))| backpack.owner == m_id)
                            .count()
                    } else {
                        0
                    }
                } else {
                    self.world
                        .query::<(&InBackpack,)>()
                        .iter()
                        .filter(|(id, (backpack,))| {
                            backpack.owner == player_id && self.world.get::<&Equipped>(*id).is_err()
                        })
                        .count()
                };
                if count > 0 && self.shop_cursor < count - 1 {
                    self.shop_cursor += 1;
                }
            }
            Action::MenuSelect => {
                let player_id = self
                    .get_player_id()
                    .expect("Player not found during shop transaction");
                let item_to_trade = if self.shop_mode == 0 {
                    if let Some(m_id) = self.active_merchant {
                        self.world
                            .query::<(&InBackpack,)>()
                            .iter()
                            .filter(|(_, (backpack,))| backpack.owner == m_id)
                            .nth(self.shop_cursor)
                            .map(|(id, _)| id)
                    } else {
                        None
                    }
                } else {
                    self.world
                        .query::<(&InBackpack,)>()
                        .iter()
                        .filter(|(id, (backpack,))| {
                            backpack.owner == player_id && self.world.get::<&Equipped>(*id).is_err()
                        })
                        .nth(self.shop_cursor)
                        .map(|(id, _)| id)
                };
                if let Some(id) = item_to_trade {
                    if self.shop_mode == 0 {
                        self.buy_item(id);
                    } else {
                        self.sell_item(id);
                    }
                    self.shop_cursor = 0;
                }
            }
            _ => {}
        }
    }
}
