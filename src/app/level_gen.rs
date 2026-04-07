use crate::app::{App, Branch, EntitySnapshot};
use crate::components::*;
use crate::map_builder::MapBuilder;
use hecs::World;
use rand::Rng;

impl App {
    fn handle_traveling_entities(&mut self, traveling_entities: Vec<EntitySnapshot>, player_start: (u16, u16)) {
        self.entities = traveling_entities;
        if self.unpack_entities().is_ok() {
            let mut player_query = self.world.query::<(&mut Position, &Player)>();
            if let Some((_, (pos, _))) = player_query.iter().next() {
                pos.x = player_start.0;
                pos.y = player_start.1;
            }
        }
    }

    fn spawn_starting_equipment(&mut self, player_id: hecs::Entity) {
        let starting_items = ["Torch", "Health Potion", "Dagger", "Leather Armor"];
        for item_name in starting_items {
            if let Some(item_raw) = self
                .content
                .items
                .iter()
                .find(|i| i.name == item_name)
                .cloned()
            {
                let item_id = crate::spawner::spawn_item_in_backpack(
                    &mut self.world,
                    player_id,
                    &item_raw,
                );
                self.identified_items.insert(item_name.to_string());
                if item_name == "Dagger" || item_name == "Leather Armor" || item_name == "Torch"
                {
                    self.equip_item(item_id);
                }
            }
        }
    }

    fn spawn_room_features(&mut self, mb: &MapBuilder, available_items: &[&crate::content::RawItem]) {
        // Spawn Merchant
        if mb.rooms.len() > 1 && !available_items.is_empty() {
            let room = &mb.rooms[1];
            let center = room.center();
            let merchant =
                crate::spawner::spawn_merchant(&mut self.world, center.0 as u16, center.1 as u16);
            for _ in 0..3 {
                let total_chance: f32 = available_items.iter().map(|i| i.spawn_chance).sum();
                let mut roll = self.rng.gen_range(0.0..total_chance);
                let mut selected_item = available_items[0];
                for item in available_items {
                    if roll < item.spawn_chance {
                        selected_item = item;
                        break;
                    }
                    roll -= item.spawn_chance;
                }
                crate::spawner::spawn_item_in_backpack(&mut self.world, merchant, selected_item);
            }
        }

        // Spawn Alchemy Station
        if mb.rooms.len() > 2 {
            let center = mb.rooms[2].center();
            crate::spawner::spawn_alchemy_station(
                &mut self.world,
                center.0 as u16,
                center.1 as u16,
            );
        }
    }

    fn spawn_environmental_features(&mut self, mb: &MapBuilder) {
        for pos in &mb.door_spawns {
            crate::spawner::spawn_door(&mut self.world, pos.0, pos.1);
        }
        for pos in &mb.trap_spawns {
            if self.current_branch == Branch::Gardens {
                crate::spawner::spawn_spore(&mut self.world, pos.0, pos.1);
            } else {
                crate::spawner::spawn_trap(&mut self.world, pos.0, pos.1);
            }
        }
    }

    fn spawn_monsters(&mut self, mb: &MapBuilder, available_monsters: &[&crate::content::RawMonster]) {
        let mut monster_spawns = mb.monster_spawns.clone();
        if self.escaping {
            monster_spawns.extend(mb.monster_spawns.clone());
        }

        for spawn in &monster_spawns {
            if available_monsters.is_empty() {
                break;
            }
            let total_chance: f32 = available_monsters.iter().map(|m| m.spawn_chance).sum();
            let mut roll = self.rng.gen_range(0.0..total_chance);
            let mut selected_monster = available_monsters[0];
            for m in available_monsters {
                if roll < m.spawn_chance {
                    selected_monster = m;
                    break;
                }
                roll -= m.spawn_chance;
            }
            crate::spawner::spawn_monster(
                &mut self.world,
                spawn.0,
                spawn.1,
                selected_monster,
                self.dungeon_level,
            );
        }

        if let Some(spawn) = mb.boss_spawn {
            if let Some(raw) = self
                .content
                .monsters
                .iter()
                .find(|m| m.is_boss == Some(true) && m.min_floor == self.dungeon_level)
            {
                crate::spawner::spawn_monster(
                    &mut self.world,
                    spawn.0,
                    spawn.1,
                    raw,
                    self.dungeon_level,
                );
                self.log.push(format!(
                    "You feel a malevolent presence... {} awaits!",
                    raw.name
                ));
            }
        }
    }

    fn spawn_items(&mut self, mb: &MapBuilder, available_items: &[&crate::content::RawItem]) {
        if self.dungeon_level == 10 && !self.escaping {
            if let Some(amulet) = self
                .content
                .items
                .iter()
                .find(|i| i.name == "Amulet of the Ancients")
            {
                let spawn_pos = mb.item_spawns.last().unwrap_or(&mb.player_start);
                crate::spawner::spawn_item(&mut self.world, spawn_pos.0, spawn_pos.1, amulet);
            }
        }

        for spawn in &mb.item_spawns {
            if available_items.is_empty() || self.rng.gen_bool(0.2) {
                crate::spawner::spawn_gold(&mut self.world, spawn.0, spawn.1, self.rng.gen_range(5..25));
                continue;
            }
            let total_chance: f32 = available_items.iter().map(|i| i.spawn_chance).sum();
            let mut roll = self.rng.gen_range(0.0..total_chance);
            let mut selected_item = available_items[0];
            for item in available_items {
                if roll < item.spawn_chance {
                    selected_item = item;
                    break;
                }
                roll -= item.spawn_chance;
            }
            crate::spawner::spawn_item(&mut self.world, spawn.0, spawn.1, selected_item);
        }
    }

    pub fn generate_level(&mut self, traveling_entities: Vec<EntitySnapshot>) {
        let mut mb = MapBuilder::new(80, 50);
        mb.build(self.dungeon_level, &mut self.rng);
        self.map = mb.map.clone();
        self.world = World::new();

        if !traveling_entities.is_empty() {
            self.handle_traveling_entities(traveling_entities, mb.player_start);
        } else {
            let player_id =
                crate::spawner::spawn_player(&mut self.world, mb.player_start.0, mb.player_start.1);
            self.spawn_starting_equipment(player_id);
        }

        self.spawn_ambient_features(&mb);
        self.spawn_stairs(&mb);

        let branch_str = match self.current_branch {
            Branch::Main => "Main",
            Branch::Gardens => "Gardens",
            Branch::Vaults => "Vaults",
        };

        let available_items: Vec<crate::content::RawItem> = self
            .content
            .items
            .iter()
            .filter(|i| self.dungeon_level >= i.min_floor && self.dungeon_level <= i.max_floor)
            .filter(|i| {
                i.branches
                    .as_ref()
                    .is_none_or(|b| b.contains(&branch_str.to_string()))
            })
            .cloned()
            .collect();

        let items_ref: Vec<&crate::content::RawItem> = available_items.iter().collect();
        self.spawn_room_features(&mb, &items_ref);
        self.spawn_environmental_features(&mb);

        let available_monsters: Vec<crate::content::RawMonster> = self
            .content
            .monsters
            .iter()
            .filter(|m| self.dungeon_level >= m.min_floor && self.dungeon_level <= m.max_floor)
            .filter(|m| {
                m.branches
                    .as_ref()
                    .is_none_or(|b| b.contains(&branch_str.to_string()))
            })
            .cloned()
            .collect();

        let monsters_ref: Vec<&crate::content::RawMonster> = available_monsters.iter().collect();
        self.spawn_monsters(&mb, &monsters_ref);
        self.spawn_items(&mb, &items_ref);

        self.update_blocked_and_opaque();
        self.update_fov();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_starting_equipment() {
        let app = App::new_random();
        // App::new_random calls generate_level(Vec::new()) which should spawn starting items

        let mut player_query = app.world.query::<&Player>();
        let (player_id, _) = player_query.iter().next().expect("Player not spawned");

        let mut items_in_backpack = 0;
        let mut items_equipped = 0;
        let mut has_torch = false;
        let mut has_dagger = false;
        let mut has_leather_armor = false;
        let mut has_health_potion = false;

        for (id, (name, backpack)) in app.world.query::<(&Name, &InBackpack)>().iter() {
            if backpack.owner == player_id {
                items_in_backpack += 1;
                if name.0 == "Torch" {
                    has_torch = true;
                }
                if name.0 == "Dagger" {
                    has_dagger = true;
                }
                if name.0 == "Leather Armor" {
                    has_leather_armor = true;
                }
                if name.0 == "Health Potion" {
                    has_health_potion = true;
                }

                if app.world.get::<&Equipped>(id).is_ok() {
                    items_equipped += 1;
                }
            }
        }

        assert!(has_torch, "Missing Torch");
        assert!(has_dagger, "Missing Dagger");
        assert!(has_leather_armor, "Missing Leather Armor");
        assert!(has_health_potion, "Missing Health Potion");
        assert_eq!(items_in_backpack, 4, "Should have 4 starting items");
        assert_eq!(items_equipped, 3, "Torch, Dagger, and Armor should be equipped");
    }
}
