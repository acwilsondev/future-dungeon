use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;

impl App {
    pub fn handle_inventory_input(&mut self, action: Action) {
        match action {
            Action::CloseMenu | Action::OpenInventory => self.state = RunState::AwaitingInput,
            Action::MenuUp => {
                if self.inventory_cursor > 0 {
                    self.inventory_cursor -= 1;
                }
            }
            Action::MenuDown => {
                if let Some(player_id) = self.get_player_id() {
                    let count = self
                        .world
                        .query::<(&InBackpack,)>()
                        .iter()
                        .filter(|(_, (backpack,))| backpack.owner == player_id)
                        .count();
                    if count > 0 && self.inventory_cursor < count - 1 {
                        self.inventory_cursor += 1;
                    }
                }
            }
            Action::MenuSelect => {
                if let Some(player_id) = self.get_player_id() {
                    let item_to_use = self
                        .world
                        .query::<(&Item, &InBackpack)>()
                        .iter()
                        .filter(|(_, (_, backpack))| backpack.owner == player_id)
                        .nth(self.inventory_cursor)
                        .map(|(id, _)| id);

                    if let Some(id) = item_to_use {
                        self.use_item(id);
                        self.inventory_cursor = 0;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_identify_input(&mut self, action: Action) {
        match action {
            Action::CloseMenu => self.state = RunState::AwaitingInput,
            Action::MenuUp => {
                if self.inventory_cursor > 0 {
                    self.inventory_cursor -= 1;
                }
            }
            Action::MenuDown => {
                if let Some(player_id) = self.get_player_id() {
                    let count = self
                        .world
                        .query::<(&InBackpack,)>()
                        .iter()
                        .filter(|(_, (backpack,))| backpack.owner == player_id)
                        .count();
                    if count > 0 && self.inventory_cursor < count - 1 {
                        self.inventory_cursor += 1;
                    }
                }
            }
            Action::MenuSelect => {
                if let Some(player_id) = self.get_player_id() {
                    let item_to_identify = self
                        .world
                        .query::<(&Item, &InBackpack)>()
                        .iter()
                        .filter(|(_, (_, backpack))| backpack.owner == player_id)
                        .nth(self.inventory_cursor)
                        .map(|(id, _)| id);

                    if let Some(id) = item_to_identify {
                        let real_name = self
                            .world
                            .get::<&Name>(id)
                            .map(|n| n.0.clone())
                            .unwrap_or("Item".to_string());
                        if !self.identified_items.contains(&real_name) {
                            self.identified_items.insert(real_name.clone());
                            self.log.push(format!("You identify the {}!", real_name));

                            if let Some(scroll_id) = self.targeting_item {
                                if let Err(e) = self.world.despawn(scroll_id) {
                                    log::error!("Failed to despawn identify scroll: {}", e);
                                }
                            }
                            self.state = RunState::AwaitingInput;
                            self.targeting_item = None;
                            self.inventory_cursor = 0;
                        } else {
                            self.log
                                .push("That item is already identified.".to_string());
                        }
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
                let new_x = (self.targeting_cursor.0 as i16 + dx)
                    .clamp(0, self.map.width as i16 - 1) as u16;
                let new_y = (self.targeting_cursor.1 as i16 + dy)
                    .clamp(0, self.map.height as i16 - 1) as u16;
                self.targeting_cursor = (new_x, new_y);
            }
            Action::MenuSelect => self.fire_targeting_item(),
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
    fn test_inventory_navigation() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        app.world.spawn((Item, InBackpack { owner: player }));
        app.world.spawn((Item, InBackpack { owner: player }));

        app.handle_inventory_input(Action::MenuDown);
        assert_eq!(app.inventory_cursor, 1);

        app.handle_inventory_input(Action::MenuUp);
        assert_eq!(app.inventory_cursor, 0);
    }

    #[test]
    fn test_inventory_use_item() {
        let mut app = setup_test_app();
        let player = app.world.spawn((
            Player,
            CombatStats { hp: 5, max_hp: 20, defense: 0, power: 5 },
            Position { x: 0, y: 0 }
        ));
        let potion = app.world.spawn((
            Item,
            Name("Health Potion".to_string()),
            Potion { heal_amount: 10 },
            Consumable,
            InBackpack { owner: player }
        ));

        app.inventory_cursor = 0;
        app.handle_inventory_input(Action::MenuSelect);

        let stats = app.world.get::<&CombatStats>(player).unwrap();
        assert_eq!(stats.hp, 15);
        assert!(app.world.get::<&Item>(potion).is_err());
    }

    #[test]
    fn test_identify_item() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let _item = app.world.spawn((
            Item,
            Name("Mysterious Potion".to_string()),
            InBackpack { owner: player },
        ));

        app.inventory_cursor = 0;
        app.handle_identify_input(Action::MenuSelect);

        assert!(app.identified_items.contains("Mysterious Potion"));
        assert_eq!(app.state, RunState::AwaitingInput);
    }

    #[test]
    fn test_targeting_input() {
        let mut app = setup_test_app();
        app.map = crate::map::Map::new(80, 50);
        app.targeting_cursor = (10, 10);

        app.handle_targeting_input(Action::MovePlayer(1, 0));
        assert_eq!(app.targeting_cursor, (11, 10));

        app.handle_targeting_input(Action::MovePlayer(0, 1));
        assert_eq!(app.targeting_cursor, (11, 11));
    }

    #[test]
    fn test_targeting_select() {
        let mut app = setup_test_app();
        app.map = crate::map::Map::new(80, 50);
        for t in app.map.tiles.iter_mut() {
            *t = crate::map::TileType::Floor;
        }
        app.map.populate_blocked_and_opaque();

        let _player = app.world.spawn((Player, Position { x: 10, y: 10 }));
        let monster = app.world.spawn((
            Monster,
            Position { x: 12, y: 10 },
            CombatStats {
                hp: 10,
                max_hp: 10,
                defense: 0,
                power: 1,
            },
        ));

        let wand = app.world.spawn((
            Item,
            Name("Wand".to_string()),
            CombatStats {
                hp: 1,
                max_hp: 1,
                defense: 0,
                power: 5,
            },
        ));

        app.targeting_item = Some(wand);
        app.targeting_cursor = (12, 10);

        app.handle_targeting_input(Action::MenuSelect);

        let m_stats = app.world.get::<&CombatStats>(monster).unwrap();
        assert_eq!(m_stats.hp, 5);
    }
}
