use crate::app::{App, Branch, EntitySnapshot};
use crate::components::*;
use crate::map_builder::MapBuilder;
use hecs::World;
use rand::Rng;

impl App {
    fn select_weighted<'a, T>(
        &mut self,
        items: &'a [T],
        spawn_chance: impl Fn(&T) -> f32,
    ) -> &'a T {
        let total: f32 = items.iter().map(&spawn_chance).sum();
        let mut roll = self.rng.random_range(0.0..total);
        for item in items.iter().take(items.len() - 1) {
            let chance = spawn_chance(item);
            if roll < chance {
                return item;
            }
            roll -= chance;
        }
        items.last().unwrap_or(&items[0])
    }

    fn handle_traveling_entities(
        &mut self,
        traveling_entities: Vec<EntitySnapshot>,
        player_start: (u16, u16),
    ) {
        self.entities = traveling_entities;
        if self.unpack_entities().is_ok() {
            let mut player_query = self.world.query::<(&mut Position, &Player)>();
            if let Some((_, (pos, _))) = player_query.iter().next() {
                pos.x = player_start.0;
                pos.y = player_start.1;
            }
        }
    }

    fn spawn_room_features(
        &mut self,
        mb: &MapBuilder,
        available_items: &[&crate::content::RawItem],
    ) {
        if self.dungeon_level % 10 == 5 {
            // Merchant Haven
            let center = mb.rooms[0].center();
            let merchant = crate::spawner::spawn_merchant(
                &mut self.world,
                center.0 as u16 + 2,
                center.1 as u16,
            );
            for _ in 0..5 {
                let selected_item = self.select_weighted(available_items, |i| i.spawn_chance);
                crate::spawner::spawn_item_in_backpack(&mut self.world, merchant, selected_item);
            }

            crate::spawner::spawn_holy_altar(&mut self.world, center.0 as u16 - 2, center.1 as u16);
            return;
        }

        if self.dungeon_level.is_multiple_of(20) {
            // Reset Shrine hidden somewhere
            let room_idx = self.rng.random_range(0..mb.rooms.len());
            let pos = mb.rooms[room_idx].center();
            crate::spawner::spawn_reset_shrine(&mut self.world, pos.0 as u16, pos.1 as u16);
        }

        // Normal spawning for non-haven floors
        // Spawn Merchant
        if mb.rooms.len() > 1 && !available_items.is_empty() && self.rng.random_bool(0.1) {
            let room_idx = self.rng.random_range(1..mb.rooms.len());
            let center = mb.rooms[room_idx].center();
            let merchant =
                crate::spawner::spawn_merchant(&mut self.world, center.0 as u16, center.1 as u16);
            for _ in 0..3 {
                let selected_item = self.select_weighted(available_items, |i| i.spawn_chance);
                crate::spawner::spawn_item_in_backpack(&mut self.world, merchant, selected_item);
            }
        }

        // Spawn Alchemy Station
        if mb.rooms.len() > 2 && self.rng.random_bool(0.2) {
            let room_idx = self.rng.random_range(1..mb.rooms.len());
            let center = mb.rooms[room_idx].center();
            crate::spawner::spawn_alchemy_station(
                &mut self.world,
                center.0 as u16,
                center.1 as u16,
            );
        }

        // Spawn Mana Shrine on every level while the magic system is being tuned.
        // Alternate color by floor parity so both Solari and Nihil appear.
        if mb.rooms.len() > 1 {
            let room_idx = self.rng.random_range(1..mb.rooms.len());
            let center = mb.rooms[room_idx].center();
            let color = if self.dungeon_level.is_multiple_of(2) {
                ManaColor::Orange
            } else {
                ManaColor::Purple
            };
            crate::spawner::spawn_mana_shrine(
                &mut self.world,
                center.0 as u16,
                center.1 as u16,
                color,
            );
        }

        // Spawn a Tome (rare). Pick a random spell from content at a level
        // appropriate for this floor.
        if mb.rooms.len() > 2 && !self.content.spells.is_empty() && self.rng.random_bool(0.05) {
            let idx = self.rng.random_range(0..self.content.spells.len());
            let raw = self.content.spells[idx].clone();
            if let Ok(baked) = raw.bake() {
                let room_idx = self.rng.random_range(1..mb.rooms.len());
                let center = mb.rooms[room_idx].center();
                let (color, level) = if baked.mana_cost.orange >= baked.mana_cost.purple {
                    (ManaColor::Orange, baked.mana_cost.orange.max(1))
                } else {
                    (ManaColor::Purple, baked.mana_cost.purple.max(1))
                };
                crate::spawner::spawn_tome(
                    &mut self.world,
                    center.0 as u16,
                    center.1 as u16,
                    &baked.title,
                    color,
                    level,
                );
            }
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
        self.spawn_partial_cover(mb);
    }

    fn spawn_partial_cover(&mut self, mb: &MapBuilder) {
        let reserved: Vec<(u16, u16)> = std::iter::once(mb.player_start)
            .chain(std::iter::once(mb.stairs_down))
            .chain(std::iter::once(mb.stairs_up))
            .chain(mb.stairs_down_alt)
            .chain(mb.door_spawns.iter().copied())
            .chain(mb.trap_spawns.iter().copied())
            .collect();

        // Skip room 0 (player spawn room) to avoid cramping the opening.
        for room in mb.rooms.iter().skip(1) {
            if !self.rng.random_bool(0.33) {
                continue;
            }
            let count = self.rng.random_range(1..=3);
            for _ in 0..count {
                if room.x2 - room.x1 < 3 || room.y2 - room.y1 < 3 {
                    break;
                }
                let x = self.rng.random_range((room.x1 + 1)..room.x2) as u16;
                let y = self.rng.random_range((room.y1 + 1)..room.y2) as u16;
                if reserved.iter().any(|&(rx, ry)| rx == x && ry == y) {
                    continue;
                }
                crate::spawner::spawn_partial_cover(&mut self.world, x, y);
            }
        }
    }

    fn spawn_monsters(
        &mut self,
        mb: &MapBuilder,
        available_monsters: &[&crate::content::RawMonster],
    ) {
        let mut monster_spawns = mb.monster_spawns.clone();
        if self.escaping {
            monster_spawns.extend_from_slice(&mb.monster_spawns);
        }

        for spawn in &monster_spawns {
            if available_monsters.is_empty() {
                break;
            }
            let selected_monster = self.select_weighted(available_monsters, |m| m.spawn_chance);
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
            if available_items.is_empty() || self.rng.random_bool(0.2) {
                crate::spawner::spawn_gold(
                    &mut self.world,
                    spawn.0,
                    spawn.1,
                    self.rng.random_range(5..25),
                );
                continue;
            }
            let selected_item = self.select_weighted(available_items, |i| i.spawn_chance);
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
            let _player_id =
                crate::spawner::spawn_player(&mut self.world, mb.player_start.0, mb.player_start.1);
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
                    .is_none_or(|b| b.iter().any(|s| s == branch_str))
            })
            .filter(|i| i.biomes.as_ref().is_none_or(|b| b.contains(&mb.biome)))
            .cloned()
            .collect();

        let items_ref: Vec<&crate::content::RawItem> = available_items.iter().collect();
        self.spawn_room_features(&mb, &items_ref);
        self.spawn_environmental_features(&mb);

        if self.dungeon_level % 10 == 5 {
            // Haven floors are safe
            self.update_blocked_and_opaque();
            self.update_fov();
            return;
        }

        let available_monsters: Vec<crate::content::RawMonster> = self
            .content
            .monsters
            .iter()
            .filter(|m| self.dungeon_level >= m.min_floor && self.dungeon_level <= m.max_floor)
            .filter(|m| {
                m.branches
                    .as_ref()
                    .is_none_or(|b| b.iter().any(|s| s == branch_str))
            })
            .filter(|m| m.biomes.as_ref().is_none_or(|b| b.contains(&mb.biome)))
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
        let mut app = App::new_random().expect("content.json must be present for tests");
        app.generate_level(Vec::new());
        app.apply_class_selection();

        let mut player_query = app.world.query::<&Player>();
        let (player_id, _) = player_query.iter().next().expect("Player not spawned");

        let mut items_in_backpack = 0;
        let mut items_equipped = 0;
        let mut has_torch = false;
        let mut has_health_potion = false;
        let mut has_longsword = false;
        let mut has_shield = false;
        let mut has_chainmail = false;

        for (id, (name, backpack)) in app.world.query::<(&Name, &InBackpack)>().iter() {
            if backpack.owner == player_id {
                items_in_backpack += 1;
                if name.0 == "Torch" {
                    has_torch = true;
                }
                if name.0 == "Health Potion" {
                    has_health_potion = true;
                }
                if name.0 == "Longsword" {
                    has_longsword = true;
                }
                if name.0 == "Shield" {
                    has_shield = true;
                }
                if name.0 == "Chainmail" {
                    has_chainmail = true;
                }

                if app.world.get::<&Equipped>(id).is_ok() {
                    items_equipped += 1;
                }
            }
        }

        assert!(has_torch, "Missing Torch");
        assert!(has_health_potion, "Missing Health Potion");
        assert!(has_longsword, "Missing Longsword");
        assert!(has_shield, "Missing Shield");
        assert!(has_chainmail, "Missing Chainmail");
        assert_eq!(items_in_backpack, 5, "Should have 5 starting items");
        assert_eq!(
            items_equipped, 3,
            "Longsword, Torch, Chainmail should be equipped (Shield is in backpack)"
        );
    }

    #[test]
    fn test_dungeon_rhythm_merchant_haven() {
        let mut app = App::new_random().expect("content.json must be present for tests");
        app.dungeon_level = 5;
        app.generate_level(Vec::new());

        // Floor 5 should have a Merchant and a Holy Altar
        let merchant_exists = app.world.query::<&Merchant>().iter().count() > 0;
        let altar_exists = app.world.query::<&HolyAltar>().iter().count() > 0;

        assert!(merchant_exists, "Floor 5 should have a Merchant");
        assert!(altar_exists, "Floor 5 should have a Holy Altar");

        // Should have NO monsters on haven floors
        let monster_count = app.world.query::<&Monster>().iter().count();
        assert_eq!(monster_count, 0, "Floor 5 should have no monsters");
    }

    #[test]
    fn test_dungeon_rhythm_boss_arena() {
        let mut app = App::new_random().expect("content.json must be present for tests");
        app.dungeon_level = 10;
        app.generate_level(Vec::new());

        // Floor 10 should have a Boss
        let boss_exists = app.world.query::<&Boss>().iter().count() > 0;
        assert!(boss_exists, "Floor 10 should have a Boss");
    }

    #[test]
    fn test_select_weighted_last_item_selectable() {
        let mut app = App::new_test(42);
        // All weight on the last item — it must always be returned.
        let items = vec![("first", 0.0f32), ("middle", 0.0f32), ("last", 1.0f32)];
        for _ in 0..20 {
            let result = app.select_weighted(&items, |i| i.1);
            assert_eq!(
                result.0, "last",
                "last item should always be selected when it has all the weight"
            );
        }
    }

    #[test]
    fn test_select_weighted_first_item_selectable() {
        let mut app = App::new_test(42);
        // All weight on the first item.
        let items = vec![("first", 1.0f32), ("last", 0.0f32)];
        for _ in 0..20 {
            let result = app.select_weighted(&items, |i| i.1);
            assert_eq!(result.0, "first");
        }
    }

    #[test]
    fn test_escaping_doubles_monster_spawns() {
        // Non-haven, non-boss floor so monsters spawn normally
        let seed = 12345u64;

        let mut normal_app = App::new_test(seed);
        normal_app.dungeon_level = 3;
        normal_app.generate_level(Vec::new());
        let normal_count = normal_app.world.query::<&Monster>().iter().count();

        let mut escaping_app = App::new_test(seed);
        escaping_app.dungeon_level = 3;
        escaping_app.escaping = true;
        escaping_app.generate_level(Vec::new());
        let escaping_count = escaping_app.world.query::<&Monster>().iter().count();

        assert!(
            escaping_count > normal_count,
            "escaping flag should increase monster count ({normal_count} → {escaping_count})"
        );
    }
}
