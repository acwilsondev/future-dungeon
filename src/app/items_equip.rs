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
            log::error!(
                "Failed to remove Equipped component from {:?}: {}",
                item_id,
                e
            );
        }
        self.log.push(format!("You unequip the {}.", item_name));
        self.refresh_player_render();
        true
    }

    pub fn equip_item(&mut self, item_id: hecs::Entity) {
        let (player_id, mut slot, is_two_handed) = {
            let Some(player_id) = self.get_player_id() else {
                log::error!("Player not found during equip_item");
                return;
            };
            let Ok(equippable) = self.world.get::<&Equippable>(item_id) else {
                log::error!("Item {:?} not equippable during equip_item", item_id);
                return;
            };
            let two_handed = self
                .world
                .get::<&Weapon>(item_id)
                .map(|w| w.two_handed)
                .unwrap_or(false);
            (player_id, equippable.slot, two_handed)
        };

        if slot == EquipmentSlot::AnyHand {
            // Pick a hand
            let mut main_hand_item = None;
            let mut off_hand_item = None;

            for (id, (eq, bp)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
                if bp.owner == player_id {
                    if eq.slot == EquipmentSlot::MainHand {
                        main_hand_item = Some(id);
                    } else if eq.slot == EquipmentSlot::OffHand {
                        off_hand_item = Some(id);
                    }
                }
            }

            let main_is_2h = main_hand_item
                .map(|id| {
                    self.world
                        .get::<&Weapon>(id)
                        .map(|w| w.two_handed)
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if main_is_2h {
                // If main is 2H, we MUST replace it if we want to use AnyHand
                slot = EquipmentSlot::MainHand;
            } else if main_hand_item.is_none() {
                slot = EquipmentSlot::MainHand;
            } else if off_hand_item.is_none() {
                slot = EquipmentSlot::OffHand;
            } else {
                // Both full, replace MainHand
                slot = EquipmentSlot::MainHand;
            }
        }

        // Handle slot conflicts
        let mut to_unequip = Vec::new();

        // 1. Items in the same slot
        for (id, (eq, backpack)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
            if backpack.owner == player_id && eq.slot == slot {
                to_unequip.push(id);
            }
        }

        // 2. Two-handed conflicts
        if slot == EquipmentSlot::MainHand && is_two_handed {
            // New item is 2H MainHand, unequip anything in OffHand
            for (id, (eq, backpack)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
                if backpack.owner == player_id && eq.slot == EquipmentSlot::OffHand {
                    to_unequip.push(id);
                }
            }
        } else if slot == EquipmentSlot::OffHand {
            // New item is OffHand, unequip any 2H weapon in MainHand
            for (id, (eq, backpack)) in self.world.query::<(&Equipped, &InBackpack)>().iter() {
                if backpack.owner == player_id && eq.slot == EquipmentSlot::MainHand {
                    if let Ok(weapon) = self.world.get::<&Weapon>(id) {
                        if weapon.two_handed {
                            to_unequip.push(id);
                        }
                    }
                }
            }
        } else if slot == EquipmentSlot::MainHand && !is_two_handed {
            // New item is 1H MainHand, if OffHand has something that is NOT compatible (none currently, but good to be safe)
            // Actually, if we had a "requires both hands" armor maybe? Not for now.
        }

        for old_item in to_unequip {
            if !self.unequip_item(old_item) {
                return; // Couldn't unequip cursed item
            }
        }

        if let Err(e) = self.world.insert_one(item_id, Equipped { slot }) {
            log::error!(
                "Failed to insert Equipped component for {:?}: {}",
                item_id,
                e
            );
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
        let mut app = App::new_test(42);
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
            Equippable {
                slot: EquipmentSlot::MainHand,
            },
            InBackpack { owner: player },
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
            Equippable {
                slot: EquipmentSlot::MainHand,
            },
            InBackpack { owner: player },
            Equipped {
                slot: EquipmentSlot::MainHand,
            },
            Cursed,
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
            Equippable {
                slot: EquipmentSlot::MainHand,
            },
            InBackpack { owner: player },
            Equipped {
                slot: EquipmentSlot::MainHand,
            },
        ));
        let new_item = app.world.spawn((
            Item,
            Name("New Sword".to_string()),
            Equippable {
                slot: EquipmentSlot::MainHand,
            },
            InBackpack { owner: player },
        ));

        app.equip_item(new_item);

        assert!(app.world.get::<&Equipped>(old_item).is_err());
        assert!(app.world.get::<&Equipped>(new_item).is_ok());
    }
}
