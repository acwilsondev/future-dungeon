use crate::app::{App, RunState};
use crate::components::*;

impl App {
    pub fn pick_up_item(&mut self) {
        let (player_pos, player_id) = {
            let mut player_query = self.world.query::<(&Position, &Player)>();
            let (id, (pos, _)) = player_query.iter().next().expect("Player not found");
            (*pos, id)
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
            self.world
                .remove_one::<Position>(item_id)
                .expect("Failed to remove Position component");
            self.world
                .insert_one(item_id, InBackpack { owner: player_id })
                .expect("Failed to insert InBackpack component");
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
        let player_id = self.get_player_id().expect("Player not found");
        let price = self
            .world
            .get::<&ItemValue>(item_id)
            .map(|v| v.price)
            .unwrap_or(0);

        let can_afford = {
            let player_gold = self
                .world
                .get::<&Gold>(player_id)
                .expect("Player has no gold component");
            player_gold.amount >= price
        };

        if can_afford {
            if let Ok(mut player_gold) = self.world.get::<&mut Gold>(player_id) {
                player_gold.amount -= price;
            }
            let item_name = self.get_item_name(item_id);
            self.log
                .push(format!("You buy the {} for {} gold.", item_name, price));

            // Transfer item
            self.world
                .insert_one(item_id, InBackpack { owner: player_id })
                .expect("Failed to insert InBackpack component");
        } else {
            self.log.push("You cannot afford that!".to_string());
        }
    }

    pub fn sell_item(&mut self, item_id: hecs::Entity) {
        if self.world.get::<&Equipped>(item_id).is_ok() {
            self.log.push("You cannot sell equipped items!".to_string());
            return;
        }
        let player_id = self.get_player_id().expect("Player not found");
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

        self.world.despawn(item_id).expect("Failed to despawn item");
    }
}
