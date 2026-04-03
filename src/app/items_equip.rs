use crate::app::App;
use crate::components::*;

impl App {
    pub fn unequip_item(&mut self, item_id: hecs::Entity) -> bool {
        let item_name = self.get_item_name(item_id);
        if self.world.get::<&Cursed>(item_id).is_ok() {
            self.identify_item(item_id);
            self.log.push(format!("You cannot unequip the {}; it's cursed!", self.get_item_name(item_id)));
            return false;
        }

        self.world.remove_one::<Equipped>(item_id).expect("Failed to remove Equipped component");
        self.log.push(format!("You unequip the {}.", item_name));
        self.refresh_player_render();
        true
    }

    pub fn equip_item(&mut self, item_id: hecs::Entity) {
        let (player_id, slot) = {
            let player_id = self.get_player_id().expect("Player not found");
            let equippable = self.world.get::<&Equippable>(item_id).expect("Item not equippable");
            (player_id, equippable.slot)
        };

        // Find if something is already in that slot
        let mut to_unequip = None;
        for (id, (eq, backpack)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
            if backpack.owner == player_id && eq.slot == slot {
                to_unequip = Some(id);
                break;
            }
        }

        if let Some(old_item) = to_unequip {
            if !self.unequip_item(old_item) {
                return; // Couldn't unequip cursed item
            }
        }

        self.world.insert_one(item_id, Equipped { slot }).expect("Failed to insert Equipped component");
        let item_name = self.get_item_name(item_id);
        self.log.push(format!("You equip the {}.", item_name));
        self.identify_item(item_id);
        self.refresh_player_render();
    }
}
