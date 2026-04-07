use crate::actions::Action;
use crate::app::{App, RunState};
use crate::components::*;

impl App {
    pub fn handle_alchemy_input(&mut self, action: Action) {
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
                    let item_to_select = self
                        .world
                        .query::<(&Item, &InBackpack)>()
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
                                self.log.push(
                                    "First item selected. Choose a second to combine.".to_string(),
                                );
                            } else if self.alchemy_selection.len() == 2 {
                                let item1 = self.alchemy_selection[0];
                                let item2 = self.alchemy_selection[1];

                                let p1_heal =
                                    self.world.get::<&Potion>(item1).ok().map(|p| p.heal_amount);
                                let p2_heal =
                                    self.world.get::<&Potion>(item2).ok().map(|p| p.heal_amount);

                                let n1 = self
                                    .world
                                    .get::<&Name>(item1)
                                    .map(|n| n.0.clone())
                                    .unwrap_or_default();
                                let n2 = self
                                    .world
                                    .get::<&Name>(item2)
                                    .map(|n| n.0.clone())
                                    .unwrap_or_default();

                                if (n1 == "Potion of Strength" && n2 == "Potion of Speed")
                                    || (n1 == "Potion of Speed" && n2 == "Potion of Strength")
                                {
                                    if let Err(e) = self.world.despawn(item1) {
                                        log::error!("Failed to despawn alchemy item 1: {}", e);
                                    }
                                    if let Err(e) = self.world.despawn(item2) {
                                        log::error!("Failed to despawn alchemy item 2: {}", e);
                                    }

                                    let _new_potion = self.world.spawn((
                                        Renderable {
                                            glyph: '!',
                                            fg: ratatui::prelude::Color::Rgb(255, 215, 0),
                                        },
                                        RenderOrder::Item,
                                        Item,
                                        Name("Potion of Heroism".to_string()),
                                        Strength {
                                            amount: 5,
                                            turns: 20,
                                        },
                                        Speed { turns: 20 },
                                        Consumable,
                                        InBackpack { owner: player_id },
                                    ));
                                    self.identified_items
                                        .insert("Potion of Heroism".to_string());
                                    self.log.push("You created a Potion of Heroism! It grants both Strength and Speed.".to_string());
                                    self.state = RunState::AwaitingInput;
                                } else if let (Some(heal1), Some(heal2)) = (p1_heal, p2_heal) {
                                    let new_heal = ((heal1 + heal2) as f32 * 1.5) as i32;

                                    if let Err(e) = self.world.despawn(item1) {
                                        log::error!("Failed to despawn alchemy item 1: {}", e);
                                    }
                                    if let Err(e) = self.world.despawn(item2) {
                                        log::error!("Failed to despawn alchemy item 2: {}", e);
                                    }

                                    let _new_potion = self.world.spawn((
                                        Renderable {
                                            glyph: '!',
                                            fg: ratatui::prelude::Color::Rgb(255, 255, 255),
                                        },
                                        RenderOrder::Item,
                                        Item,
                                        Name("Greater Potion".to_string()),
                                        Potion {
                                            heal_amount: new_heal,
                                        },
                                        Consumable,
                                        InBackpack { owner: player_id },
                                    ));
                                    self.identify_item(_new_potion);

                                    self.log.push(format!(
                                        "You combined the potions into a Greater Potion (Heals {})!",
                                        new_heal
                                    ));
                                    self.state = RunState::AwaitingInput;
                                } else {
                                    self.log.push("You can only combine potions!".to_string());
                                    self.alchemy_selection.clear();
                                }
                            }
                        }
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
    fn test_alchemy_selection() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let item1 = app.world.spawn((Item, InBackpack { owner: player }));
        let _item2 = app.world.spawn((Item, InBackpack { owner: player }));

        app.inventory_cursor = 0;
        app.handle_alchemy_input(Action::MenuSelect);
        assert_eq!(app.alchemy_selection.len(), 1);
        assert_eq!(app.alchemy_selection[0], item1);

        // Deselect
        app.handle_alchemy_input(Action::MenuSelect);
        assert_eq!(app.alchemy_selection.len(), 0);
    }

    #[test]
    fn test_alchemy_combine_heroism() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let _p1 = app.world.spawn((
            Item,
            Name("Potion of Strength".to_string()),
            Potion { heal_amount: 0 },
            InBackpack { owner: player }
        ));
        let _p2 = app.world.spawn((
            Item,
            Name("Potion of Speed".to_string()),
            Potion { heal_amount: 0 },
            InBackpack { owner: player }
        ));

        app.inventory_cursor = 0;
        app.handle_alchemy_input(Action::MenuSelect);
        app.inventory_cursor = 1;
        app.handle_alchemy_input(Action::MenuSelect);

        assert_eq!(app.state, RunState::AwaitingInput);

        let mut heroism_query = app.world.query::<&Name>();
        let heroism = heroism_query
            .iter()
            .find(|(_, name)| name.0 == "Potion of Heroism");
        assert!(heroism.is_some());
    }

    #[test]
    fn test_alchemy_combine_greater() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        let _p1 = app.world.spawn((
            Item,
            Name("Health Potion".to_string()),
            Potion { heal_amount: 10 },
            InBackpack { owner: player }
        ));
        let _p2 = app.world.spawn((
            Item,
            Name("Mana Potion".to_string()),
            Potion { heal_amount: 10 },
            InBackpack { owner: player }
        ));

        app.inventory_cursor = 0;
        app.handle_alchemy_input(Action::MenuSelect);
        app.inventory_cursor = 1;
        app.handle_alchemy_input(Action::MenuSelect);

        let mut greater_query = app.world.query::<(&Name, &Potion)>();
        let (_, (_name, potion)) = greater_query
            .iter()
            .find(|(_, (name, _))| name.0 == "Greater Potion")
            .unwrap();
        assert_eq!(potion.heal_amount, 30); // (10+10)*1.5
    }

    #[test]
    fn test_alchemy_navigation() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        app.world.spawn((Item, InBackpack { owner: player }));
        app.world.spawn((Item, InBackpack { owner: player }));

        app.handle_alchemy_input(Action::MenuDown);
        assert_eq!(app.inventory_cursor, 1);

        app.handle_alchemy_input(Action::MenuUp);
        assert_eq!(app.inventory_cursor, 0);
    }

    #[test]
    fn test_alchemy_invalid_combination() {
        let mut app = setup_test_app();
        let player = app.world.spawn((Player, Position { x: 0, y: 0 }));
        app.world.spawn((Item, InBackpack { owner: player }));
        app.world.spawn((Item, InBackpack { owner: player }));

        app.inventory_cursor = 0;
        app.handle_alchemy_input(Action::MenuSelect);
        app.inventory_cursor = 1;
        app.handle_alchemy_input(Action::MenuSelect);

        assert!(app.log.last().unwrap().contains("only combine potions"));
        assert_eq!(app.alchemy_selection.len(), 0);
    }
}
