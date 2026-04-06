use crate::app::{App, RunState};
use crate::components::*;

impl App {
    pub fn pick_up_item(&mut self) {
        let (player_pos, player_id) = {
            let Some(player_id) = self.get_player_id() else {
                return;
            };
            let Ok(pos) = self.world.get::<&Position>(player_id) else {
                return;
            };
            (*pos, player_id)
        };
        let mut item_to_pick = None;
        for (id, (pos, _)) in self.world.query::<(&Position, &Item)>().iter() {
            if pos.x == player_pos.x && pos.y == player_pos.y {
                item_to_pick = Some(id);
                break;
            }
        }
        if let Some(item_id) = item_to_pick {
            let item_name = self.get_item_name(item_id);
            let _ = self.world.remove_one::<Position>(item_id);
            let _ = self.world.insert_one(item_id, InBackpack { owner: player_id });
            self.log.push(format!("You pick up the {}.", item_name));
            self.generate_noise(player_pos.x, player_pos.y, 2.0);

            if item_name == "Amulet of the Ancients" {
                self.escaping = true;
                self.log
                    .push("You hold the Amulet! The dungeon rumbles... Escaping time!".to_string());
            }

            self.state = RunState::MonsterTurn;
        } else {
            self.log
                .push("There is nothing here to pick up.".to_string());
        }
    }

    pub fn buy_item(&mut self, item_id: hecs::Entity) {
        let Some(player_id) = self.get_player_id() else {
            return;
        };
        let price = self
            .world
            .get::<&ItemValue>(item_id)
            .map(|v| v.price)
            .unwrap_or(0);

        let can_afford = {
            if let Ok(player_gold) = self.world.get::<&Gold>(player_id) {
                player_gold.amount >= price
            } else {
                false
            }
        };

        if can_afford {
            if let Ok(mut player_gold) = self.world.get::<&mut Gold>(player_id) {
                player_gold.amount -= price;
            }
            let item_name = self.get_item_name(item_id);
            self.log
                .push(format!("You buy the {} for {} gold.", item_name, price));

            // Transfer item
            let _ = self.world.insert_one(item_id, InBackpack { owner: player_id });
        } else {
            self.log.push("You cannot afford that!".to_string());
        }
    }

    pub fn sell_item(&mut self, item_id: hecs::Entity) {
        if self.world.get::<&Equipped>(item_id).is_ok() {
            self.log.push("You cannot sell equipped items!".to_string());
            return;
        }
        let Some(player_id) = self.get_player_id() else {
            return;
        };
        let price = self
            .world
            .get::<&ItemValue>(item_id)
            .map(|v| v.price / 2)
            .unwrap_or(1); // Sell for half price

        {
            if let Ok(mut player_gold) = self.world.get::<&mut Gold>(player_id) {
                player_gold.amount += price;
            }
        }

        let item_name = self.get_item_name(item_id);
        self.log
            .push(format!("You sell the {} for {} gold.", item_name, price));

        if let Err(e) = self.world.despawn(item_id) {
            log::error!("Failed to despawn sold item {:?}: {}", item_id, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::World;

    fn setup_test_app() -> App {
        let mut app = App::new_random();
        app.world = World::new();
        app
    }

    #[test]
    fn test_pick_up_item() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 10, y: 10 }));
        let item = app.world.spawn((Item, Position { x: 10, y: 10 }, Name("Test Item".to_string())));

        app.pick_up_item();

        let backpack = app.world.get::<&InBackpack>(item).unwrap();
        assert_eq!(backpack.owner, player);
        assert!(app.world.get::<&Position>(item).is_err());
    }

    #[test]
    fn test_buy_item_success() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Gold { amount: 100 }));
        let item = app.world.spawn((Item, ItemValue { price: 40 }));

        app.buy_item(item);

        let backpack = app.world.get::<&InBackpack>(item).unwrap();
        assert_eq!(backpack.owner, player);
        let gold = app.world.get::<&Gold>(player).unwrap();
        assert_eq!(gold.amount, 60);
    }

    #[test]
    fn test_buy_item_fail_no_gold() {
        let mut app = setup_test_app();
        let _player = app.world.spawn((Player, Gold { amount: 10 }));
        let item = app.world.spawn((Item, ItemValue { price: 40 }));

        app.buy_item(item);

        assert!(app.world.get::<&InBackpack>(item).is_err());
    }

    #[test]
    fn test_sell_item() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Gold { amount: 0 }));
        let item = app.world.spawn((
            Item,
            InBackpack { owner: player },
            ItemValue { price: 40 },
            Name("Old Sword".to_string())
        ));

        app.sell_item(item);

        let gold = app.world.get::<&Gold>(player).unwrap();
        assert_eq!(gold.amount, 20); // 40 / 2
        assert!(app.world.get::<&Item>(item).is_err()); // Despawned
    }
}
