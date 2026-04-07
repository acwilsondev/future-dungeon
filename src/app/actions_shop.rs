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
                if let Some(player_id) = self.get_player_id() {
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
                                backpack.owner == player_id
                                    && self.world.get::<&Equipped>(*id).is_err()
                            })
                            .count()
                    };
                    if count > 0 && self.shop_cursor < count - 1 {
                        self.shop_cursor += 1;
                    }
                }
            }
            Action::MenuSelect => {
                if let Some(player_id) = self.get_player_id() {
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
                                backpack.owner == player_id
                                    && self.world.get::<&Equipped>(*id).is_err()
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
            }
            _ => {}
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
        app
    }

    #[test]
    fn test_shop_navigation() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let merchant = app.world.spawn((Merchant,));
        app.active_merchant = Some(merchant);
        app.world.spawn((Item, InBackpack { owner: merchant }));
        app.world.spawn((Item, InBackpack { owner: merchant }));

        app.shop_mode = 0; // Buy mode
        app.handle_shop_input(Action::MenuDown);
        assert_eq!(app.shop_cursor, 1);

        app.handle_shop_input(Action::MenuUp);
        assert_eq!(app.shop_cursor, 0);
    }

    #[test]
    fn test_shop_toggle_mode() {
        let mut app = setup_test_app();
        app.shop_mode = 0;
        app.handle_shop_input(Action::ToggleShopMode);
        assert_eq!(app.shop_mode, 1);
        app.handle_shop_input(Action::ToggleShopMode);
        assert_eq!(app.shop_mode, 0);
    }

    #[test]
    fn test_shop_buy_item() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Gold { amount: 100 }, Position { x: 0, y: 0 }));
        let merchant = app.world.spawn((Merchant,));
        app.active_merchant = Some(merchant);
        let item = app.world.spawn((
            Item,
            InBackpack { owner: merchant },
            ItemValue { price: 50 },
            Name("Shiny Sword".to_string())
        ));

        app.shop_mode = 0;
        app.shop_cursor = 0;
        app.handle_shop_input(Action::MenuSelect);

        let backpack = app.world.get::<&InBackpack>(item).unwrap();
        assert_eq!(backpack.owner, player);
        let gold = app.world.get::<&Gold>(player).unwrap();
        assert_eq!(gold.amount, 50);
    }

    #[test]
    fn test_shop_buy_item_fail_no_gold() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Gold { amount: 10 }, Position { x: 0, y: 0 }));
        let merchant = app.world.spawn((Merchant,));
        app.active_merchant = Some(merchant);
        let item = app.world.spawn((
            Item,
            InBackpack { owner: merchant },
            ItemValue { price: 50 },
            Name("Expensive Sword".to_string())
        ));

        app.shop_mode = 0;
        app.shop_cursor = 0;
        app.handle_shop_input(Action::MenuSelect);

        let backpack = app.world.get::<&InBackpack>(item).unwrap();
        assert_eq!(backpack.owner, merchant); // Still with merchant
    }

    #[test]
    fn test_shop_sell_item() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Gold { amount: 0 }, Position { x: 0, y: 0 }));
        let item = app.world.spawn((
            Item,
            InBackpack { owner: player },
            ItemValue { price: 50 },
            Name("Old Boots".to_string())
        ));

        app.shop_mode = 1; // Sell mode
        app.shop_cursor = 0;
        app.handle_shop_input(Action::MenuSelect);

        let gold = app.world.get::<&Gold>(player).unwrap();
        assert_eq!(gold.amount, 25); // 50 / 2
        assert!(app.world.get::<&Item>(item).is_err());
    }
}
