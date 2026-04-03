use crate::app::{App, Branch, EntitySnapshot};
use crate::components::*;
use crate::map_builder::MapBuilder;
use hecs::World;
use rand::Rng;

impl App {
    pub fn generate_level(&mut self, traveling_entities: Vec<EntitySnapshot>) {
        let mut mb = MapBuilder::new(80, 50);
        mb.build(self.dungeon_level);
        self.map = mb.map.clone();
        self.world = World::new();
        let mut rng = rand::thread_rng();

        if !traveling_entities.is_empty() {
            self.entities = traveling_entities;
            if self.unpack_entities().is_ok() {
                let mut player_query = self.world.query::<(&mut Position, &Player)>();
                if let Some((_, (pos, _))) = player_query.iter().next() {
                    pos.x = mb.player_start.0;
                    pos.y = mb.player_start.1;
                }
            }
        } else {
            let player_id =
                crate::spawner::spawn_player(&mut self.world, mb.player_start.0, mb.player_start.1);
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
                    if item_name == "Dagger" || item_name == "Leather Armor" || item_name == "Torch" {
                        self.equip_item(item_id);
                    }
                }
            }
        }

        self.spawn_ambient_features(&mb);
        self.spawn_stairs(&mb);

        let branch_str = match self.current_branch {
            Branch::Main => "Main",
            Branch::Gardens => "Gardens",
            Branch::Vaults => "Vaults",
        };
        let available_items: Vec<&crate::content::RawItem> = self
            .content
            .items
            .iter()
            .filter(|i| self.dungeon_level >= i.min_floor && self.dungeon_level <= i.max_floor)
            .filter(|i| {
                i.branches
                    .as_ref()
                    .is_none_or(|b| b.contains(&branch_str.to_string()))
            })
            .collect();

        if mb.rooms.len() > 1 && !available_items.is_empty() {
            let room = &mb.rooms[1];
            let center = room.center();
            let merchant =
                crate::spawner::spawn_merchant(&mut self.world, center.0 as u16, center.1 as u16);
            for _ in 0..3 {
                let total_chance: f32 = available_items.iter().map(|i| i.spawn_chance).sum();
                let mut roll = rng.gen_range(0.0..total_chance);
                let mut selected_item = available_items[0];
                for item in &available_items {
                    if roll < item.spawn_chance {
                        selected_item = item;
                        break;
                    }
                    roll -= item.spawn_chance;
                }
                crate::spawner::spawn_item_in_backpack(&mut self.world, merchant, selected_item);
            }
        }

        if mb.rooms.len() > 2 {
            let center = mb.rooms[2].center();
            crate::spawner::spawn_alchemy_station(
                &mut self.world,
                center.0 as u16,
                center.1 as u16,
            );
        }

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

        let available_monsters: Vec<&crate::content::RawMonster> = self
            .content
            .monsters
            .iter()
            .filter(|m| self.dungeon_level >= m.min_floor && self.dungeon_level <= m.max_floor)
            .filter(|m| {
                m.branches
                    .as_ref()
                    .is_none_or(|b| b.contains(&branch_str.to_string()))
            })
            .collect();

        let mut monster_spawns = mb.monster_spawns.clone();
        if self.escaping {
            monster_spawns.extend(mb.monster_spawns.clone());
        }

        for spawn in &monster_spawns {
            if available_monsters.is_empty() {
                break;
            }
            let total_chance: f32 = available_monsters.iter().map(|m| m.spawn_chance).sum();
            let mut roll = rng.gen_range(0.0..total_chance);
            let mut selected_monster = available_monsters[0];
            for m in &available_monsters {
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
            if available_items.is_empty() || rng.gen_bool(0.2) {
                crate::spawner::spawn_gold(&mut self.world, spawn.0, spawn.1, rng.gen_range(5..25));
                continue;
            }
            let total_chance: f32 = available_items.iter().map(|i| i.spawn_chance).sum();
            let mut roll = rng.gen_range(0.0..total_chance);
            let mut selected_item = available_items[0];
            for item in &available_items {
                if roll < item.spawn_chance {
                    selected_item = item;
                    break;
                }
                roll -= item.spawn_chance;
            }
            crate::spawner::spawn_item(&mut self.world, spawn.0, spawn.1, selected_item);
        }

        self.update_blocked_and_opaque();
        self.update_fov();
    }
}
