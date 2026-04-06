use crate::app::App;
use crate::components::*;

impl App {
    pub fn unequip_item(&mut self, item_id: hecs::Entity) -> bool {
        let item_name = self.get_item_name(item_id);
        if self.world.get::<&Cursed>(item_id).is_ok() {
            self.identify_item(item_id);
            self.log.push(format!(
                "You cannot unequip the {}; it's cursed!",
                self.get_item_name(item_id)
            ));
            return false;
        }

        if let Err(e) = self.world.remove_one::<Equipped>(item_id) {
            log::error!("Failed to remove Equipped component from {:?}: {}", item_id, e);
        }
        self.log.push(format!("You unequip the {}.", item_name));
        self.refresh_player_render();
        true
    }

    pub fn equip_item(&mut self, item_id: hecs::Entity) {
        let (player_id, slot) = {
            let Some(player_id) = self.get_player_id() else {
                log::error!("Player not found during equip_item");
                return;
            };
            let Ok(equippable) = self.world.get::<&Equippable>(item_id) else {
                log::error!("Item {:?} not equippable during equip_item", item_id);
                return;
            };
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

        if let Err(e) = self.world.insert_one(item_id, Equipped { slot }) {
            log::error!("Failed to insert Equipped component for {:?}: {}", item_id, e);
        }
        let item_name = self.get_item_name(item_id);
        self.log.push(format!("You equip the {}.", item_name));
        self.identify_item(item_id);
        self.refresh_player_render();
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
    fn test_equip_unequip_item() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let item = app.world.spawn((
            Item,
            Name("Sword".to_string()),
            Equippable { slot: EquipmentSlot::Melee },
            InBackpack { owner: player }
        ));

        app.equip_item(item);
        assert!(app.world.get::<&Equipped>(item).is_ok());

        let success = app.unequip_item(item);
        assert!(success);
        assert!(app.world.get::<&Equipped>(item).is_err());
    }

    #[test]
    fn test_cursed_item_prevents_unequip() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let item = app.world.spawn((
            Item,
            Name("Cursed Sword".to_string()),
            Equippable { slot: EquipmentSlot::Melee },
            InBackpack { owner: player },
            Equipped { slot: EquipmentSlot::Melee },
            Cursed
        ));

        let success = app.unequip_item(item);
        assert!(!success);
        assert!(app.world.get::<&Equipped>(item).is_ok());
    }

    #[test]
    fn test_equip_replaces_old_item() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let old_item = app.world.spawn((
            Item,
            Name("Old Sword".to_string()),
            Equippable { slot: EquipmentSlot::Melee },
            InBackpack { owner: player },
            Equipped { slot: EquipmentSlot::Melee }
        ));
        let new_item = app.world.spawn((
            Item,
            Name("New Sword".to_string()),
            Equippable { slot: EquipmentSlot::Melee },
            InBackpack { owner: player }
        ));

        app.equip_item(new_item);

        assert!(app.world.get::<&Equipped>(old_item).is_err());
        assert!(app.world.get::<&Equipped>(new_item).is_ok());
    }
}
