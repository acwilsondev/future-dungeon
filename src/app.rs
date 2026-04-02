use ratatui::prelude::*;
use serde::{Deserialize, Serialize};
use crate::map_builder::MapBuilder;
use crate::components::*;
use hecs::World;
use bracket_pathfinding::prelude::*;
use rand::Rng;
use std::collections::HashMap;

use crate::map::{Map, TileType};
use crate::actions::Action;
use crate::content::{Content, RawItem, RawMonster};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunState {
    AwaitingInput,
    MonsterTurn,
    ShowInventory,
    ShowHelp,
    ShowTargeting,
    LevelUp,
    ShowShop,
    ShowLogHistory,
    ShowBestiary,
    Dead,
    Victory,
}

pub enum MonsterAction {
    Move(i16, i16),
    Attack(hecs::Entity),
    RangedAttack(hecs::Entity),
}

/// A snapshot of an entity for serialization
#[derive(Serialize, Deserialize, Clone)]
pub struct EntitySnapshot {
    pub pos: Option<Position>,
    pub render: Renderable,
    pub render_order: RenderOrder,
    pub name: Option<Name>,
    pub stats: Option<CombatStats>,
    #[serde(default)]
    pub potion: Option<Potion>,
    #[serde(default)]
    pub weapon: Option<Weapon>,
    #[serde(default)]
    pub armor: Option<Armor>,
    #[serde(default)]
    pub door: Option<Door>,
    #[serde(default)]
    pub trap: Option<Trap>,
    #[serde(default)]
    pub ranged: Option<Ranged>,
    #[serde(default)]
    pub ranged_weapon: Option<RangedWeapon>,
    #[serde(default)]
    pub aoe: Option<AreaOfEffect>,
    #[serde(default)]
    pub confusion: Option<Confusion>,
    #[serde(default)]
    pub poison: Option<Poison>,
    #[serde(default)]
    pub strength: Option<Strength>,
    #[serde(default)]
    pub speed: Option<Speed>,
    #[serde(default)]
    pub faction: Option<Faction>,
    #[serde(default)]
    pub viewshed: Option<Viewshed>,
    #[serde(default)]
    pub personality: Option<AIPersonality>,
    #[serde(default)]
    pub experience: Option<Experience>,
    #[serde(default)]
    pub perks: Option<Perks>,
    #[serde(default)]
    pub alert_state: Option<AlertState>,
    #[serde(default)]
    pub hearing: Option<Hearing>,
    #[serde(default)]
    pub boss: Option<Boss>,
    #[serde(default)]
    pub light_source: Option<LightSource>,
    #[serde(default)]
    pub gold: Option<Gold>,
    #[serde(default)]
    pub item_value: Option<ItemValue>,
    #[serde(default)]
    pub last_hit_by_player: bool,
    #[serde(default)]
    pub is_merchant: bool,
    #[serde(default)]
    pub ammo: bool,
    #[serde(default)]
    pub consumable: bool,
    #[serde(default)]
    pub in_backpack: bool,
    pub is_player: bool,
    pub is_monster: bool,
    #[serde(default)]
    pub is_wisp: bool,
    #[serde(default)]
    pub is_item: bool,
    #[serde(default)]
    pub is_down_stairs: bool,
    #[serde(default)]
    pub is_up_stairs: bool,
}

#[derive(Serialize, Deserialize)]
pub struct LevelData {
    pub map: Map,
    pub entities: Vec<EntitySnapshot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VisualEffect {
    Flash { x: u16, y: u16, glyph: char, fg: Color, bg: Option<Color>, duration: u32 },
    Projectile { path: Vec<(u16, u16)>, glyph: char, fg: Color, frame: u32, speed: u32 },
}

#[derive(Serialize, Deserialize)]
pub struct App {
    #[serde(skip)]
    pub exit: bool,
    #[serde(skip)]
    pub death: bool,
    #[serde(skip, default = "World::new")]
    pub world: World,
    pub map: Map,
    pub entities: Vec<EntitySnapshot>,
    pub levels: HashMap<u16, LevelData>,
    pub log: Vec<String>,
    pub dungeon_level: u16,
    #[serde(skip, default = "default_runstate")]
    pub state: RunState,
    #[serde(skip)]
    pub inventory_cursor: usize,
    #[serde(skip)]
    pub targeting_cursor: (u16, u16),
    #[serde(skip)]
    pub targeting_item: Option<hecs::Entity>,
    #[serde(skip)]
    pub speed_toggle: bool,
    #[serde(skip)]
    pub level_up_cursor: usize,
    #[serde(skip)]
    pub shop_cursor: usize,
    #[serde(skip)]
    pub active_merchant: Option<hecs::Entity>,
    #[serde(skip)]
    pub shop_mode: usize, // 0 = Buy, 1 = Sell
    #[serde(skip)]
    pub effects: Vec<VisualEffect>,
    #[serde(skip)]
    pub log_cursor: usize,
    pub encountered_monsters: std::collections::HashSet<String>,
    #[serde(skip)]
    pub bestiary_cursor: usize,
    pub content: Content,
    #[serde(skip)]
    pub fps: f32,
    pub escaping: bool,
    pub monsters_killed: u32,
}

fn default_runstate() -> RunState { RunState::AwaitingInput }

impl App {
    pub fn new() -> Self {
        Self::new_random()
    }

    pub fn new_random() -> Self {
        let mut app = Self {
            exit: false,
            death: false,
            world: World::new(),
            map: Map::new(80, 50),
            entities: Vec::new(),
            levels: HashMap::new(),
            log: vec!["Welcome to RustLike!".to_string()],
            dungeon_level: 1,
            state: RunState::AwaitingInput,
            inventory_cursor: 0,
            targeting_cursor: (0, 0),
            targeting_item: None,
            speed_toggle: true,
            level_up_cursor: 0,
            shop_cursor: 0,
            active_merchant: None,
            shop_mode: 0,
            effects: Vec::new(),
            log_cursor: 0,
            encountered_monsters: std::collections::HashSet::new(),
            bestiary_cursor: 0,
            content: Self::load_content(),
            fps: 0.0,
            escaping: false,
            monsters_killed: 0,
        };
        app.generate_level(None);
        app
    }

    pub fn get_player_id(&self) -> Option<hecs::Entity> {
        self.world.query::<&Player>().iter().next().map(|(id, _)| id)
    }

    fn load_content() -> Content {
        let path = std::path::Path::new("content.json");
        if path.exists() {
            if let Ok(json) = std::fs::read_to_string(path) {
                if let Ok(content) = serde_json::from_str(&json) {
                    return content;
                }
            }
        }
        Content::default()
    }

    pub fn on_tick(&mut self) {
        let mut still_active = Vec::new();

        for effect in self.effects.drain(..) {
            match effect {
                VisualEffect::Flash { x, y, glyph, fg, bg, duration } => {
                    if duration > 1 {
                        still_active.push(VisualEffect::Flash { x, y, glyph, fg, bg, duration: duration - 1 });
                    }
                }
                VisualEffect::Projectile { path, glyph, fg, frame, speed } => {
                    let new_frame = frame + 1;
                    if new_frame < (path.len() as u32 * speed) {
                        still_active.push(VisualEffect::Projectile { path, glyph, fg, frame: new_frame, speed });
                    }
                }
            }
        }
        self.effects = still_active;
    }

    pub fn process_action(&mut self, action: Action) {
        match self.state {
            RunState::AwaitingInput => {
                match action {
                    Action::Quit => self.exit = true,
                    Action::MovePlayer(dx, dy) => self.move_player(dx, dy),
                    Action::PickUpItem => self.pick_up_item(),
                    Action::OpenInventory => self.state = RunState::ShowInventory,
                    Action::OpenHelp => self.state = RunState::ShowHelp,
                    Action::OpenLogHistory => { self.state = RunState::ShowLogHistory; self.log_cursor = self.log.len().saturating_sub(1); }
                    Action::OpenBestiary => { self.state = RunState::ShowBestiary; self.bestiary_cursor = 0; }
                    Action::TryLevelTransition => self.try_level_transition(),
                    Action::Wait => self.state = RunState::MonsterTurn,
                    _ => {}
                }
            }
            RunState::ShowLogHistory => {
                match action {
                    Action::CloseMenu | Action::OpenLogHistory => self.state = RunState::AwaitingInput,
                    Action::MenuUp => if self.log_cursor > 0 { self.log_cursor -= 1; },
                    Action::MenuDown => if self.log_cursor < self.log.len().saturating_sub(1) { self.log_cursor += 1; },
                    _ => {}
                }
            }
            RunState::ShowBestiary => {
                match action {
                    Action::CloseMenu | Action::OpenBestiary => self.state = RunState::AwaitingInput,
                    Action::MenuUp => if self.bestiary_cursor > 0 { self.bestiary_cursor -= 1; },
                    Action::MenuDown => {
                        let count = self.encountered_monsters.len();
                        if count > 0 && self.bestiary_cursor < count - 1 { self.bestiary_cursor += 1; }
                    },
                    _ => {}
                }
            }
            RunState::ShowInventory => {
                match action {
                    Action::CloseMenu | Action::OpenInventory => self.state = RunState::AwaitingInput,
                    Action::MenuUp => if self.inventory_cursor > 0 { self.inventory_cursor -= 1; },
                    Action::MenuDown => {
                        let player_id = self.get_player_id().expect("Player not found during inventory browsing");
                        let count = self.world.query::<(&crate::components::InBackpack,)>().iter()
                            .filter(|(_, (backpack,))| backpack.owner == player_id).count();
                        if count > 0 && self.inventory_cursor < count - 1 { self.inventory_cursor += 1; }
                    },
                    Action::MenuSelect => {
                        let player_id = self.get_player_id().expect("Player not found during item selection");
                        let item_to_use = self.world.query::<(&crate::components::Item, &crate::components::InBackpack)>()
                            .iter()
                            .filter(|(_, (_, backpack))| backpack.owner == player_id)
                            .nth(self.inventory_cursor)
                            .map(|(id, _)| id);
                        
                        if let Some(id) = item_to_use {
                            self.use_item(id);
                            self.inventory_cursor = 0;
                        }
                    }
                    _ => {}
                }
            }
            RunState::ShowTargeting => {
                match action {
                    Action::CloseMenu => self.state = RunState::AwaitingInput,
                    Action::MovePlayer(dx, dy) => {
                        let new_x = (self.targeting_cursor.0 as i16 + dx).clamp(0, self.map.width as i16 - 1) as u16;
                        let new_y = (self.targeting_cursor.1 as i16 + dy).clamp(0, self.map.height as i16 - 1) as u16;
                        self.targeting_cursor = (new_x, new_y);
                    }
                    Action::MenuSelect => self.fire_targeting_item(),
                    _ => {}
                }
            }
            RunState::ShowHelp => {
                if let Action::CloseMenu | Action::OpenHelp = action {
                    self.state = RunState::AwaitingInput;
                }
            }
            RunState::LevelUp => {
                match action {
                    Action::MenuUp => if self.level_up_cursor > 0 { self.level_up_cursor -= 1; },
                    Action::MenuDown => if self.level_up_cursor < 3 { self.level_up_cursor += 1; },
                    Action::MenuSelect => {
                        let player_id = self.get_player_id().expect("Player not found during level up");
                        match self.level_up_cursor {
                            0 => {
                                if let Ok(mut stats) = self.world.get::<&mut crate::components::CombatStats>(player_id) {
                                    stats.max_hp += 10; stats.hp += 10;
                                }
                                if let Ok(mut perks) = self.world.get::<&mut crate::components::Perks>(player_id) {
                                    perks.traits.push(crate::components::Perk::Toughness);
                                }
                                self.log.push("You chose Toughness! Max HP increased.".to_string());
                            }
                            1 => {
                                if let Ok(mut viewshed) = self.world.get::<&mut crate::components::Viewshed>(player_id) {
                                    viewshed.visible_tiles += 2;
                                }
                                if let Ok(mut perks) = self.world.get::<&mut crate::components::Perks>(player_id) {
                                    perks.traits.push(crate::components::Perk::EagleEye);
                                }
                                self.log.push("You chose Eagle Eye! FOV increased.".to_string());
                            }
                            2 => {
                                if let Ok(mut stats) = self.world.get::<&mut crate::components::CombatStats>(player_id) {
                                    stats.power += 2;
                                }
                                if let Ok(mut perks) = self.world.get::<&mut crate::components::Perks>(player_id) {
                                    perks.traits.push(crate::components::Perk::Strong);
                                }
                                self.log.push("You chose Strong! Power increased.".to_string());
                            }
                            3 => {
                                if let Ok(mut stats) = self.world.get::<&mut crate::components::CombatStats>(player_id) {
                                    stats.defense += 1;
                                }
                                if let Ok(mut perks) = self.world.get::<&mut crate::components::Perks>(player_id) {
                                    perks.traits.push(crate::components::Perk::ThickSkin);
                                }
                                self.log.push("You chose Thick Skin! Defense increased.".to_string());
                            }
                            _ => {}
                        }
                        self.state = RunState::MonsterTurn;
                    }
                    _ => {}
                }
            }
            RunState::ShowShop => {
                match action {
                    Action::CloseMenu => self.state = RunState::AwaitingInput,
                    Action::ToggleShopMode => {
                        self.shop_mode = (self.shop_mode + 1) % 2;
                        self.shop_cursor = 0;
                    }
                    Action::MenuUp => if self.shop_cursor > 0 { self.shop_cursor -= 1; },
                    Action::MenuDown => {
                        let player_id = self.get_player_id().expect("Player not found during shop browsing");
                        let count = if self.shop_mode == 0 {
                            if let Some(m_id) = self.active_merchant {
                                self.world.query::<(&crate::components::InBackpack,)>().iter()
                                    .filter(|(_, (backpack,))| backpack.owner == m_id).count()
                            } else { 0 }
                        } else {
                            self.world.query::<(&crate::components::InBackpack,)>().iter()
                                .filter(|(_, (backpack,))| backpack.owner == player_id).count()
                        };
                        if count > 0 && self.shop_cursor < count - 1 { self.shop_cursor += 1; }
                    }
                    Action::MenuSelect => {
                        let player_id = self.get_player_id().expect("Player not found during shop transaction");
                        let item_to_trade = if self.shop_mode == 0 {
                            if let Some(m_id) = self.active_merchant {
                                self.world.query::<(&crate::components::InBackpack,)>().iter()
                                    .filter(|(_, (backpack,))| backpack.owner == m_id)
                                    .nth(self.shop_cursor).map(|(id, _)| id)
                            } else { None }
                        } else {
                            self.world.query::<(&crate::components::InBackpack,)>().iter()
                                .filter(|(_, (backpack,))| backpack.owner == player_id)
                                .nth(self.shop_cursor).map(|(id, _)| id)
                        };
                        if let Some(id) = item_to_trade {
                            if self.shop_mode == 0 { self.buy_item(id); } else { self.sell_item(id); }
                            self.shop_cursor = 0;
                        }
                    }
                    _ => {}
                }
            }
            RunState::Dead | RunState::Victory => {
                if let Action::Quit | Action::CloseMenu = action { self.exit = true; }
            }
            _ => {}
        }
    }

    pub fn generate_level(&mut self, player_snapshot: Option<EntitySnapshot>) {
        let mut mb = MapBuilder::new(80, 50);
        mb.build(self.dungeon_level);
        self.map = mb.map;
        self.world = World::new();
        let mut rng = rand::thread_rng();

        if let Some(snapshot) = player_snapshot {
            self.entities = vec![snapshot];
            self.unpack_entities();
            let mut player_query = self.world.query::<(&mut Position, &Player)>();
            let (_, (pos, _)) = player_query.iter().next().expect("Player not found after unpack");
            pos.x = mb.player_start.0;
            pos.y = mb.player_start.1;
        } else {
            crate::spawner::spawn_player(&mut self.world, mb.player_start.0, mb.player_start.1);
        }

        // Spawn ambient light sources (Glowing Crystals) in some rooms
        for (i, room) in mb.rooms.iter().enumerate().skip(1) {
            if i % 3 == 0 {
                let center = room.center();
                crate::spawner::spawn_light_crystal(&mut self.world, center.0 as u16, center.1 as u16);
            }
            if i % 5 == 0 {
                let center = room.center();
                crate::spawner::spawn_wisp(&mut self.world, center.0 as u16, center.1 as u16);
            }
        }

        crate::spawner::spawn_stairs(&mut self.world, mb.stairs_down.0, mb.stairs_down.1, true);
        crate::spawner::spawn_stairs(&mut self.world, mb.stairs_up.0, mb.stairs_up.1, false);

        let available_items: Vec<&RawItem> = self.content.items.iter()
            .filter(|i| self.dungeon_level >= i.min_floor && self.dungeon_level <= i.max_floor)
            .collect();

        // Spawn a Merchant in a random room (usually the second one)
        if mb.rooms.len() > 1 && !available_items.is_empty() {
            let room = &mb.rooms[1];
            let center = room.center();
            let merchant = crate::spawner::spawn_merchant(&mut self.world, center.0 as u16, center.1 as u16);
            
            // Give merchant 3 random items from available items
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

        for pos in &mb.door_spawns {
            crate::spawner::spawn_door(&mut self.world, pos.0, pos.1);
        }

        for pos in &mb.trap_spawns {
            crate::spawner::spawn_trap(&mut self.world, pos.0, pos.1);
        }

        let available_monsters: Vec<&RawMonster> = self.content.monsters.iter()
            .filter(|m| self.dungeon_level >= m.min_floor && self.dungeon_level <= m.max_floor)
            .collect();

        let mut monster_spawns = mb.monster_spawns.clone();
        if self.escaping {
            // Double the monster spawns for extra pressure
            monster_spawns.extend(mb.monster_spawns.clone());
        }

        for spawn in &monster_spawns {
            if available_monsters.is_empty() { break; }
            
            // weighted selection
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
            
            let raw = selected_monster;
            crate::spawner::spawn_monster(&mut self.world, spawn.0, spawn.1, raw, self.dungeon_level);
        }

        if let Some(spawn) = mb.boss_spawn {
            let boss_raw = self.content.monsters.iter()
                .find(|m| m.is_boss == Some(true) && m.min_floor == self.dungeon_level);
            
            if let Some(raw) = boss_raw {
                crate::spawner::spawn_monster(&mut self.world, spawn.0, spawn.1, raw, self.dungeon_level);
                self.log.push(format!("You feel a malevolent presence... {} awaits!", raw.name));
            }
        }
        
        // Spawn Amulet on floor 10
        if self.dungeon_level == 10 && !self.escaping {
            let amulet_raw = self.content.items.iter().find(|i| i.name == "Amulet of the Ancients");
            if let Some(amulet) = amulet_raw {
                let spawn_pos = mb.item_spawns.pop().unwrap_or(mb.player_start); // Use the last item spawn or fallback to player start
                crate::spawner::spawn_item(&mut self.world, spawn_pos.0, spawn_pos.1, amulet);
            }
        }

        for spawn in &mb.item_spawns {
            // 20% chance for gold, otherwise pick item
            if available_items.is_empty() || rng.gen_bool(0.2) {
                let amount = rng.gen_range(5..25);
                crate::spawner::spawn_gold(&mut self.world, spawn.0, spawn.1, amount);
                continue;
            }

            // weighted selection
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

    pub fn update_blocked_and_opaque(&mut self) {
        self.map.populate_blocked_and_opaque();
        let mut doors = Vec::new();
        for (_, (pos, door)) in self.world.query::<(&Position, &Door)>().iter() {
            doors.push((pos.x, pos.y, door.open));
        }
        for (x, y, open) in doors {
            let idx = (y * self.map.width + x) as usize;
            if !open {
                self.map.blocked[idx] = true;
                self.map.opaque[idx] = true;
            }
        }
    }

    pub fn go_down_level(&mut self) {
        self.pack_entities();
        let current_entities = self.entities.clone();
        let player_snapshot = current_entities.iter().find(|e| e.is_player).cloned().expect("Player entity not found during level transition");
        let level_entities: Vec<EntitySnapshot> = current_entities.into_iter().filter(|e| !e.is_player).collect();

        self.levels.insert(self.dungeon_level, LevelData {
            map: self.map.clone(),
            entities: level_entities,
        });

        self.dungeon_level += 1;

        if let Some(level_data) = self.levels.get(&self.dungeon_level) {
            self.map = level_data.map.clone();
            self.entities = level_data.entities.clone();
            self.entities.push(player_snapshot);
            self.unpack_entities();
            let mut up_stairs_pos = (0, 0);
            for (_, (pos, _)) in self.world.query::<(&Position, &UpStairs)>().iter() {
                up_stairs_pos = (pos.x, pos.y);
            }
            let mut player_query = self.world.query::<(&mut Position, &Player)>();
            let (_, (pos, _)) = player_query.iter().next().expect("Player not found after transition");
            pos.x = up_stairs_pos.0;
            pos.y = up_stairs_pos.1;
        } else {
            self.generate_level(Some(player_snapshot));
        }
        self.log.push(format!("You descend to level {}.", self.dungeon_level));
    }

    pub fn go_up_level(&mut self) {
        if self.dungeon_level <= 1 {
            if self.escaping {
                self.state = RunState::Victory;
                self.log.push("You escape the dungeon with the Amulet! You win!".to_string());
            } else {
                self.log.push("You cannot go further up without the Amulet!".to_string());
            }
            return;
        }

        self.pack_entities();
        let current_entities = self.entities.clone();
        let player_snapshot = current_entities.iter().find(|e| e.is_player).cloned().expect("Player entity not found during level transition");
        let level_entities: Vec<EntitySnapshot> = current_entities.into_iter().filter(|e| !e.is_player).collect();

        self.levels.insert(self.dungeon_level, LevelData {
            map: self.map.clone(),
            entities: level_entities,
        });

        self.dungeon_level -= 1;

        let level_data = self.levels.get(&self.dungeon_level).expect("Level data not found for target level");
        self.map = level_data.map.clone();
        self.entities = level_data.entities.clone();
        self.entities.push(player_snapshot);
        self.unpack_entities();

        let mut down_stairs_pos = (0, 0);
        for (_, (pos, _)) in self.world.query::<(&Position, &DownStairs)>().iter() {
            down_stairs_pos = (pos.x, pos.y);
        }
        let mut player_query = self.world.query::<(&mut Position, &Player)>();
        let (_, (pos, _)) = player_query.iter().next().expect("Player not found after transition up");
        pos.x = down_stairs_pos.0;
        pos.y = down_stairs_pos.1;

        self.log.push(format!("You ascend to level {}.", self.dungeon_level));
    }

    pub fn update_lighting(&mut self) {
        for l in self.map.light.iter_mut() { *l = 0.0; }

        let mut light_sources = Vec::new();
        for (_id, (pos, light)) in self.world.query::<(&Position, &LightSource)>().iter() {
            light_sources.push((*pos, *light));
        }

        for (pos, light) in light_sources {
            let idx_source = pos.y as usize * self.map.width as usize + pos.x as usize;
            self.map.light[idx_source] = (self.map.light[idx_source] + 1.0).min(1.5);

            let fov = field_of_view(Point::new(pos.x, pos.y), light.range, &self.map);
            for p in fov {
                if p.x >= 0 && p.x < self.map.width as i32 && p.y >= 0 && p.y < self.map.height as i32 {
                    let idx = p.y as usize * self.map.width as usize + p.x as usize;
                    let dist = (((p.x as f32 - pos.x as f32).powi(2) + (p.y as f32 - pos.y as f32).powi(2))).sqrt();
                    let intensity = 1.0 - (dist / light.range as f32);
                    self.map.light[idx] = (self.map.light[idx] + intensity).min(1.5); // Can be slightly over-bright
                }
            }
        }
    }

    pub fn update_sound(&mut self) {
        for s in self.map.sound.iter_mut() { *s = 0.0; }

        let mut noise_sources = Vec::new();
        for (_id, (pos, noise)) in self.world.query::<(&Position, &Noise)>().iter() {
            noise_sources.push((*pos, noise.amount));
        }

        for (pos, amount) in noise_sources {
            // Sound propagation using Dijkstra-like approach to "bend" around corners
            let mut dijkstra = DijkstraMap::new(self.map.width, self.map.height, &[], &self.map, 20.0);
            dijkstra.map[self.map.point2d_to_index(Point::new(pos.x, pos.y))] = 0.0;
            
            // We need a custom Dijkstra that accounts for wall muffling
            // But for now let's use a simpler approach or the built-in one if it fits.
            // bracket-lib's DijkstraMap is more for pathfinding.
            
            // Let's use a BFS for sound propagation
            let mut queue = std::collections::VecDeque::new();
            queue.push_back((pos.x, pos.y, amount));
            
            let start_idx = (pos.y as u16 * self.map.width + pos.x as u16) as usize;
            self.map.sound[start_idx] += amount;

            let mut visited = std::collections::HashSet::new();
            visited.insert((pos.x, pos.y));

            while let Some((cx, cy, current_amount)) = queue.pop_front() {
                if current_amount <= 0.1 { continue; }

                for (dx, dy) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = cx as i16 + dx;
                    let ny = cy as i16 + dy;
                    if nx >= 0 && nx < self.map.width as i16 && ny >= 0 && ny < self.map.height as i16 {
                        let nux = nx as u16;
                        let nuy = ny as u16;
                        if visited.contains(&(nux, nuy)) { continue; }
                        
                        let idx = (nuy * self.map.width + nux) as usize;
                        let attenuation = if self.map.tiles[idx] == TileType::Wall {
                            4.0 // Walls muffle sound significantly
                        } else {
                            1.1 // Open air attenuation
                        };

                        let next_amount = current_amount - attenuation;
                        if next_amount > 0.0 {
                            self.map.sound[idx] += next_amount;
                            visited.insert((nux, nuy));
                            queue.push_back((nux, nuy, next_amount));
                        }
                    }
                }
            }
        }
    }

    pub fn update_fov(&mut self) {
        self.update_lighting();
        self.update_sound();
        let (pos, range) = {
            let mut player_query = self.world.query::<(&Position, &Player)>();
            let (id, (pos, _)) = player_query.iter().next().expect("Player not found");
            let range = self.world.get::<&Viewshed>(id).map(|v| v.visible_tiles).unwrap_or(8);
            (*pos, range)
        };

        // Calculate broad LOS (max 20 tiles)
        let fov = field_of_view(Point::new(pos.x, pos.y), 25, &self.map);
        for v in &mut self.map.visible { *v = false; }
        for p in fov {
            if p.x >= 0 && p.x < self.map.width as i32 && p.y >= 0 && p.y < self.map.height as i32 {
                let idx = (p.y as u16 * self.map.width + p.x as u16) as usize;
                let dist = (((p.x as f32 - pos.x as f32).powi(2) + (p.y as f32 - pos.y as f32).powi(2))).sqrt();

                // Visible if within player's sight range OR if the tile is lit
                if dist <= range as f32 || self.map.light[idx] > 0.1 {
                    self.map.visible[idx] = true;
                    self.map.revealed[idx] = true;
                }
            }
        }


        // Record encountered monsters
        for (_id, (pos, name, _)) in self.world.query::<(&Position, &Name, &Monster)>().iter() {
            let idx = (pos.y as u16 * self.map.width + pos.x as u16) as usize;
            if self.map.visible[idx] {
                self.encountered_monsters.insert(name.0.clone());
            }
        }
    }

    pub fn pack_entities(&mut self) {
        self.entities.clear();
        for (id, (render, render_order)) in self.world.query::<(&Renderable, &RenderOrder)>().iter() {
            let pos = self.world.get::<&Position>(id).ok().map(|p| *p);
            let name = self.world.get::<&Name>(id).ok().map(|n| (*n).clone());
            let stats = self.world.get::<&CombatStats>(id).ok().map(|s| *s);
            let potion = self.world.get::<&Potion>(id).ok().map(|p| *p);
            let weapon = self.world.get::<&Weapon>(id).ok().map(|w| *w);
            let armor = self.world.get::<&Armor>(id).ok().map(|a| *a);
            let door = self.world.get::<&Door>(id).ok().map(|d| *d);
            let trap = self.world.get::<&Trap>(id).ok().map(|t| *t);
            let ranged = self.world.get::<&Ranged>(id).ok().map(|r| *r);
            let ranged_weapon = self.world.get::<&RangedWeapon>(id).ok().map(|rw| *rw);
            let aoe = self.world.get::<&AreaOfEffect>(id).ok().map(|a| *a);
            let confusion = self.world.get::<&Confusion>(id).ok().map(|c| *c);
            let poison = self.world.get::<&Poison>(id).ok().map(|p| *p);
            let strength = self.world.get::<&Strength>(id).ok().map(|s| *s);
            let speed = self.world.get::<&Speed>(id).ok().map(|s| *s);
            let faction = self.world.get::<&Faction>(id).ok().map(|f| *f);
            let viewshed = self.world.get::<&Viewshed>(id).ok().map(|v| *v);
            let personality = self.world.get::<&AIPersonality>(id).ok().map(|p| *p);
            let experience = self.world.get::<&Experience>(id).ok().map(|e| *e);
            let perks = self.world.get::<&Perks>(id).ok().map(|p| (*p).clone());
            let alert_state = self.world.get::<&AlertState>(id).ok().map(|a| *a);
            let hearing = self.world.get::<&Hearing>(id).ok().map(|h| *h);
            let boss = self.world.get::<&Boss>(id).ok().map(|b| (*b).clone());
            let light_source = self.world.get::<&LightSource>(id).ok().map(|l| *l);
            let gold = self.world.get::<&Gold>(id).ok().map(|g| *g);
            let item_value = self.world.get::<&ItemValue>(id).ok().map(|v| *v);
            
            self.entities.push(EntitySnapshot {
                pos, render: *render, render_order: *render_order, name, stats, potion, weapon, armor, door, trap, ranged, 
                ranged_weapon, aoe, confusion, poison, strength, speed,
                faction, viewshed, personality, experience, perks, alert_state, hearing, boss, light_source, gold, item_value,
                last_hit_by_player: self.world.get::<&LastHitByPlayer>(id).is_ok(),
                is_merchant: self.world.get::<&Merchant>(id).is_ok(),
                ammo: self.world.get::<&Ammunition>(id).is_ok(),
                consumable: self.world.get::<&Consumable>(id).is_ok(),
                in_backpack: self.world.get::<&InBackpack>(id).is_ok(),
                is_player: self.world.get::<&Player>(id).is_ok(),
                is_monster: self.world.get::<&Monster>(id).is_ok(),
                is_wisp: self.world.get::<&Wisp>(id).is_ok(),
                is_item: self.world.get::<&Item>(id).is_ok(),
                is_down_stairs: self.world.get::<&DownStairs>(id).is_ok(),
                is_up_stairs: self.world.get::<&UpStairs>(id).is_ok(),
            });
        }
    }

    pub fn unpack_entities(&mut self) {
        self.world = World::new();
        let mut player_entity = None;
        let mut in_backpack_markers = Vec::new();

        for e in &self.entities {
            let mut cb = hecs::EntityBuilder::new();
            if let Some(pos) = e.pos { cb.add(pos); }
            cb.add(e.render);
            cb.add(e.render_order);
            if let Some(ref name) = e.name { cb.add(name.clone()); }
            if let Some(stats) = e.stats { cb.add(stats); }
            if let Some(potion) = e.potion { cb.add(potion); }
            if let Some(weapon) = e.weapon { cb.add(weapon); }
            if let Some(armor) = e.armor { cb.add(armor); }
            if let Some(door) = e.door { cb.add(door); }
            if let Some(trap) = e.trap { cb.add(trap); }
            if let Some(ranged) = e.ranged { cb.add(ranged); }
            if let Some(ranged_weapon) = e.ranged_weapon { cb.add(ranged_weapon); }
            if let Some(aoe) = e.aoe { cb.add(aoe); }
            if let Some(confusion) = e.confusion { cb.add(confusion); }
            if let Some(poison) = e.poison { cb.add(poison); }
            if let Some(strength) = e.strength { cb.add(strength); }
            if let Some(speed) = e.speed { cb.add(speed); }
            if let Some(faction) = e.faction { cb.add(faction); }
            if let Some(viewshed) = e.viewshed { cb.add(viewshed); }
            if let Some(personality) = e.personality { cb.add(personality); }
            if let Some(experience) = e.experience { cb.add(experience); }
            if let Some(perks) = e.perks.clone() { cb.add(perks); }
            if let Some(alert_state) = e.alert_state { cb.add(alert_state); }
            if let Some(hearing) = e.hearing { cb.add(hearing); }
            if let Some(boss) = e.boss.clone() { cb.add(boss); }
            if let Some(light_source) = e.light_source { cb.add(light_source); }
            if let Some(gold) = e.gold { cb.add(gold); }
            if let Some(item_value) = e.item_value { cb.add(item_value); }
            if e.last_hit_by_player { cb.add(LastHitByPlayer); }
            if e.is_merchant { cb.add(Merchant); }
            if e.ammo { cb.add(Ammunition); }
            if e.consumable { cb.add(Consumable); }
            if e.is_player { cb.add(Player); }
            if e.is_monster { cb.add(Monster); }
            if e.is_wisp { cb.add(Wisp); }
            if e.is_item { cb.add(Item); }
            if e.is_down_stairs { cb.add(DownStairs); }
            if e.is_up_stairs { cb.add(UpStairs); }
            let entity = self.world.spawn(cb.build());
            if e.is_player { player_entity = Some(entity); }
            if e.in_backpack { in_backpack_markers.push(entity); }
        }

        if let Some(player) = player_entity {
            for id in in_backpack_markers {
                self.world.insert_one(id, InBackpack { owner: player }).expect("Failed to insert InBackpack component during unpack");
            }
        }

        self.map.visible = vec![false; (self.map.width * self.map.height) as usize];
        self.update_blocked_and_opaque();
        self.update_fov();
    }

    pub fn add_player_xp(&mut self, xp: i32) {
        let Some(player_id) = self.get_player_id() else { return; };
        let mut level_up = false;

        if let Ok(mut exp) = self.world.get::<&mut Experience>(player_id) {            exp.xp += xp;
            self.log.push(format!("You gained {} XP.", xp));
            
            if exp.xp >= exp.next_level_xp {
                exp.level += 1;
                exp.xp -= exp.next_level_xp;
                exp.next_level_xp = exp.next_level_xp.saturating_add(exp.next_level_xp / 2);
                level_up = true;
            }
        }
        
        if level_up {
            self.state = RunState::LevelUp;
            self.level_up_cursor = 0;
            self.log.push("You leveled up!".to_string());
        }
    }

    pub fn move_player(&mut self, dx: i16, dy: i16) {
        let (new_x, new_y, player_power) = {
            let mut player_query = self.world.query::<(&Position, &Player, &CombatStats)>();
            let (_, (pos, _, player_stats)) = player_query.iter().next().expect("Player not found");
            ((pos.x as i16 + dx).max(0) as u16, (pos.y as i16 + dy).max(0) as u16, player_stats.power)
        };

        let mut target_interactable = None;
        for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
            if pos.x == new_x && pos.y == new_y { target_interactable = Some(id); break; }
        }
        if target_interactable.is_none() {
            for (id, (pos, _)) in self.world.query::<(&Position, &Merchant)>().iter() {
                if pos.x == new_x && pos.y == new_y { target_interactable = Some(id); break; }
            }
        }

        if let Some(target_id) = target_interactable {
            // Check if it's a Merchant
            if self.world.get::<&Merchant>(target_id).is_ok() {
                self.active_merchant = Some(target_id);
                self.state = RunState::ShowShop;
                self.shop_cursor = 0;
                self.log.push("You talk to the Merchant.".to_string());
                return;
            }

            let mut monster_damaged = false;
            let mut monster_died = false;
            let monster_name = self.world.get::<&Name>(target_id).map(|n| n.0.clone()).unwrap_or("Something".to_string());
            let mut xp_reward = 0;
            
            {
                if let Ok(mut monster_stats) = self.world.get::<&mut CombatStats>(target_id) {
                    let mut damage = (player_power - monster_stats.defense).max(0);
                    
                    // Sneak Attack?
                    if let Ok(alert) = self.world.get::<&AlertState>(target_id) {
                        if *alert != AlertState::Aggressive {
                            damage *= 2;
                            self.log.push(format!("Sneak Attack on {}!", monster_name));
                        }
                    }

                    monster_stats.hp -= damage;
                    self.log.push(format!("You hit {} for {} damage!", monster_name, damage));
                    self.effects.push(VisualEffect::Flash { x: new_x, y: new_y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
                    monster_damaged = true;

                    if monster_stats.hp <= 0 { 
                        monster_died = true; 
                        if let Ok(exp) = self.world.get::<&Experience>(target_id) {
                            xp_reward = exp.xp_reward;
                        }
                    }
                }
            }
            if monster_damaged {
                self.generate_noise(new_x, new_y, 8.0); // Combat is loud
            }

            if !monster_died && monster_damaged {
                self.world.insert_one(target_id, LastHitByPlayer).expect("Failed to insert LastHitByPlayer");
                self.world.insert_one(target_id, AlertState::Aggressive).expect("Failed to alert monster");
            }
            if monster_died {
                self.log.push(format!("{} dies!", monster_name));
                self.world.despawn(target_id).expect("Failed to despawn monster");
                self.monsters_killed += 1;
                self.add_player_xp(xp_reward);
            }
            if self.state != RunState::LevelUp {
                self.state = RunState::MonsterTurn;
            }
            return;
        }

        let mut target_door = None;
        for (id, (d_pos, door)) in self.world.query::<(&Position, &Door)>().iter() {
            if d_pos.x == new_x && d_pos.y == new_y && !door.open { target_door = Some(id); break; }
        }

        if let Some(door_id) = target_door {
            if let Ok(mut door) = self.world.get::<&mut Door>(door_id) {
                door.open = true;
            }
            if let Ok(mut render) = self.world.get::<&mut Renderable>(door_id) {
                render.glyph = '/';
            }
            self.log.push("You open the door.".to_string());
            self.generate_noise(new_x, new_y, 10.0); // Opening doors is very loud
            self.update_blocked_and_opaque();
            self.update_fov();
            self.state = RunState::MonsterTurn;
            return;
        }

        if !self.map.blocked[(new_y as u16 * self.map.width + new_x as u16) as usize] {
            let mut player_query = self.world.query::<(&mut Position, &Player)>();
            let (player_id, (pos, _)) = player_query.iter().next().expect("Player not found");
            pos.x = new_x; pos.y = new_y;
            drop(player_query);
            self.generate_noise(new_x, new_y, 3.0); // Moving is quiet but not silent

            // Gold pickup - ensure we don't pick up the player!
            let mut gold_to_pick = Vec::new();
            for (id, (g_pos, gold)) in self.world.query::<(&Position, &Gold)>().iter() {
                if id != player_id && g_pos.x == new_x && g_pos.y == new_y {
                    gold_to_pick.push((id, gold.amount));
                }
            }
            
            for (id, amount) in gold_to_pick {
                if let Ok(mut player_gold) = self.world.get::<&mut Gold>(player_id) {
                    player_gold.amount += amount;
                    self.log.push(format!("You pick up {} gold.", amount));
                }
                self.world.despawn(id).expect("Failed to despawn gold");
            }

            let mut total_damage = 0;
            let mut triggered_traps = Vec::new();
            for (id, (t_pos, trap)) in self.world.query::<(&Position, &mut Trap)>().iter() {
                if t_pos.x == new_x && t_pos.y == new_y {
                    triggered_traps.push(id); total_damage += trap.damage; trap.revealed = true;
                }
            }
            if total_damage > 0 {
                self.log.push(format!("A trap deals {} damage to you!", total_damage));
                let mut stats_query = self.world.query::<(&mut CombatStats, &Player)>();
                if let Some((_, (player_stats, _))) = stats_query.iter().next() {
                    player_stats.hp -= total_damage;
                    if player_stats.hp <= 0 { self.death = true; self.state = RunState::Dead; }
                }
                drop(stats_query);
                for trap_id in triggered_traps { self.world.despawn(trap_id).expect("Failed to despawn trap"); }
            }
            self.update_fov();
            self.state = RunState::MonsterTurn;
        }
    }

    pub fn try_level_transition(&mut self) {
        let player_pos = {
            let mut player_query = self.world.query::<(&Position, &Player)>();
            let (_, (pos, _)) = player_query.iter().next().expect("Player not found");
            *pos
        };
        let mut transition = None;
        for (_, (pos, _)) in self.world.query::<(&Position, &DownStairs)>().iter() { if pos.x == player_pos.x && pos.y == player_pos.y { transition = Some(true); } }
        for (_, (pos, _)) in self.world.query::<(&Position, &UpStairs)>().iter() { if pos.x == player_pos.x && pos.y == player_pos.y { transition = Some(false); } }
        if let Some(down) = transition { if down { self.go_down_level(); } else { self.go_up_level(); } }
        else { self.log.push("There are no stairs here.".to_string()); }
    }

    pub fn pick_up_item(&mut self) {
        let (player_pos, player_id) = {
            let mut player_query = self.world.query::<(&Position, &Player)>();
            let (id, (pos, _)) = player_query.iter().next().expect("Player not found");
            (*pos, id)
        };
        let mut item_to_pick = None;
        for (id, (pos, _)) in self.world.query::<(&Position, &Item)>().iter() {
            if pos.x == player_pos.x && pos.y == player_pos.y { item_to_pick = Some(id); break; }
        }
        if let Some(item_id) = item_to_pick {
            let item_name = self.world.get::<&Name>(item_id).map(|n| n.0.clone()).unwrap_or("Item".to_string());
            self.world.remove_one::<Position>(item_id).expect("Failed to remove Position component");
            self.world.insert_one(item_id, InBackpack { owner: player_id }).expect("Failed to insert InBackpack component");
            self.log.push(format!("You pick up the {}.", item_name));
            self.generate_noise(player_pos.x, player_pos.y, 2.0);
            
            if item_name == "Amulet of the Ancients" {
                self.escaping = true;
                self.log.push("You hold the Amulet! The dungeon rumbles... Escaping time!".to_string());
            }
            
            self.state = RunState::MonsterTurn;
        } else { self.log.push("There is nothing here to pick up.".to_string()); }
    }

    pub fn buy_item(&mut self, item_id: hecs::Entity) {
        let player_id = self.get_player_id().expect("Player not found");
        let price = self.world.get::<&ItemValue>(item_id).map(|v| v.price).unwrap_or(0);
        
        let can_afford = {
            let player_gold = self.world.get::<&Gold>(player_id).expect("Player has no gold component");
            player_gold.amount >= price
        };

        if can_afford {
            if let Ok(mut player_gold) = self.world.get::<&mut Gold>(player_id) {
                player_gold.amount -= price;
            }
            let item_name = self.world.get::<&Name>(item_id).map(|n| n.0.clone()).unwrap_or("Item".to_string());
            self.log.push(format!("You buy the {} for {} gold.", item_name, price));
            
            // Transfer item
            self.world.insert_one(item_id, InBackpack { owner: player_id }).expect("Failed to insert InBackpack component");
        } else {
            self.log.push("You cannot afford that!".to_string());
        }
    }

    pub fn sell_item(&mut self, item_id: hecs::Entity) {
        let player_id = self.get_player_id().expect("Player not found");
        let price = self.world.get::<&ItemValue>(item_id).map(|v| v.price / 2).unwrap_or(1); // Sell for half price
        
        {
            if let Ok(mut player_gold) = self.world.get::<&mut Gold>(player_id) {
                player_gold.amount += price;
            }
        }
        
        let item_name = self.world.get::<&Name>(item_id).map(|n| n.0.clone()).unwrap_or("Item".to_string());
        self.log.push(format!("You sell the {} for {} gold.", item_name, price));
        
        self.world.despawn(item_id).expect("Failed to despawn item");
    }

    pub fn use_item(&mut self, item_id: hecs::Entity) {
        let player_id = self.get_player_id().expect("Player not found");
        let item_name = self.world.get::<&Name>(item_id).map(|n| n.0.clone()).unwrap_or("Item".to_string());
        let player_pos = self.world.get::<&Position>(player_id).ok().map(|p| *p).unwrap_or(Position { x: 0, y: 0 });

        let mut handled = false;
        
        let potion_heal = self.world.get::<&Potion>(item_id).ok().map(|p| p.heal_amount);
        if let Some(heal_amount) = potion_heal {
            if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                stats.hp = (stats.hp + heal_amount).min(stats.max_hp);
            }
            self.log.push(format!("You drink the {}, healing for {} HP.", item_name, heal_amount));
            self.generate_noise(player_pos.x, player_pos.y, 1.0);
            handled = true;
        }

        let poison_effect = self.world.get::<&Poison>(item_id).ok().map(|p| *p);
        if let Some(poison) = poison_effect {
             self.world.insert_one(player_id, poison).expect("Failed to insert Poison component");
             self.log.push(format!("You are poisoned by the {}!", item_name));
             handled = true;
        }
        
        let strength_effect = self.world.get::<&Strength>(item_id).ok().map(|s| *s);
        if let Some(strength) = strength_effect {
             self.world.insert_one(player_id, strength).expect("Failed to insert Strength component");
             if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                stats.power += strength.amount;
             }
             self.log.push(format!("The {} makes you feel much stronger!", item_name));
             handled = true;
        }

        let speed_effect = self.world.get::<&Speed>(item_id).ok().map(|s| *s);
        if let Some(speed) = speed_effect {
             self.world.insert_one(player_id, speed).expect("Failed to insert Speed component");
             self.log.push(format!("The {} makes you feel incredibly fast!", item_name));
             handled = true;
        }

        if item_name == "Torch" {
            self.world.insert_one(player_id, LightSource { range: 10, color: (255, 255, 100) }).expect("Failed to insert LightSource component");
            self.log.push("You light a torch. The shadows retreat.".to_string());
            self.generate_noise(player_pos.x, player_pos.y, 2.0);
            handled = true;
        }


        if self.world.get::<&Ranged>(item_id).is_ok() || self.world.get::<&RangedWeapon>(item_id).is_ok() {
            if self.world.get::<&RangedWeapon>(item_id).is_ok() {
                // Check for ammo
                let has_ammo = self.world.query::<(&Ammunition, &InBackpack)>().iter().any(|(_, (_, backpack))| backpack.owner == player_id);
                if !has_ammo {
                    self.log.push("You have no ammunition for this weapon!".to_string());
                    return;
                }
            }
            if let Ok(player_pos) = self.world.get::<&Position>(player_id) {
                self.targeting_cursor = (player_pos.x, player_pos.y);
                self.targeting_item = Some(item_id);
                self.state = RunState::ShowTargeting;
                self.log.push(format!("Select target for {}...", item_name));
            }
            return;
        } else if let Ok(weapon) = self.world.get::<&Weapon>(item_id) {
             self.log.push(format!("You equip the {}. Your power increases!", item_name));
             if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                stats.power += weapon.power_bonus;
             }
             handled = true;
        } else if let Ok(armor) = self.world.get::<&Armor>(item_id) {
             self.log.push(format!("You equip the {}. Your defense increases!", item_name));
             if let Ok(mut stats) = self.world.get::<&mut CombatStats>(player_id) {
                stats.defense += armor.defense_bonus;
             }
             handled = true;
        }

        if handled { self.world.despawn(item_id).expect("Failed to despawn item after use"); self.state = RunState::MonsterTurn; }
    }

    pub fn fire_targeting_item(&mut self) {
        if let Some(item_id) = self.targeting_item {
            let item_name = self.world.get::<&Name>(item_id).map(|n| n.0.clone()).unwrap_or("Item".to_string());
            
            let (player_pos, player_id) = {
                let Some(id) = self.get_player_id() else { return; };
                let Ok(pos) = self.world.get::<&Position>(id) else { return; };
                (*pos, id)
            };

            let line = line2d(
                LineAlg::Bresenham,
                Point::new(player_pos.x, player_pos.y),
                Point::new(self.targeting_cursor.0, self.targeting_cursor.1)
            );

            let mut actual_target = self.targeting_cursor;
            for p in line.iter().skip(1) {
                let idx = (p.y as u16 * self.map.width + p.x as u16) as usize;
                if self.map.blocked[idx] {
                    actual_target = (p.x as u16, p.y as u16);
                    self.log.push(format!("The {} is blocked!", item_name));
                    break;
                }
            }

            // Add projectile animation
            let path: Vec<(u16, u16)> = line.iter()
                .take_while(|p| (p.x as u16, p.y as u16) != actual_target)
                .map(|p| (p.x as u16, p.y as u16))
                .chain(std::iter::once(actual_target))
                .collect();
            self.effects.push(VisualEffect::Projectile { path, glyph: '*', fg: Color::Yellow, frame: 0, speed: 1 });

            let mut targets = Vec::new();
            
            // Collect info before mutations
            let aoe_radius = self.world.get::<&AreaOfEffect>(item_id).ok().map(|a| a.radius);
            let confusion_turns = self.world.get::<&Confusion>(item_id).ok().map(|c| c.turns);
            let poison_effect = self.world.get::<&Poison>(item_id).ok().map(|p| *p);
            let mut power = self.world.get::<&CombatStats>(item_id).map(|s| s.power).unwrap_or(10);
            let is_ranged_weapon = self.world.get::<&RangedWeapon>(item_id).ok().map(|rw| *rw);

            if let Some(rw) = is_ranged_weapon {
                power = rw.damage_bonus;
                // Consume ammo
                let ammo_id = self.world.query::<(&Ammunition, &InBackpack)>()
                    .iter()
                    .filter(|(_, (_, backpack))| backpack.owner == player_id)
                    .map(|(id, _)| id)
                    .next();
                if let Some(aid) = ammo_id {
                    self.world.despawn(aid).expect("Failed to despawn ammunition");
                }
            }

            if let Some(radius) = aoe_radius {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    let dist = (((pos.x as f32 - actual_target.0 as f32).powi(2) + (pos.y as f32 - actual_target.1 as f32).powi(2))).sqrt();
                    if dist <= radius as f32 { targets.push(id); }
                }
                self.log.push(format!("The {} explodes!", item_name));
                self.generate_noise(actual_target.0, actual_target.1, 15.0); // Explosions are very loud
                for target_id in targets {
                    if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target_id) { stats.hp -= power; }
                    if let Ok(t_pos) = self.world.get::<&Position>(target_id) {
                        self.effects.push(VisualEffect::Flash { x: t_pos.x, y: t_pos.y, glyph: '*', fg: Color::Indexed(208), bg: None, duration: 10 });
                    }
                    self.world.insert_one(target_id, LastHitByPlayer).expect("Failed to insert LastHitByPlayer");
                    self.world.insert_one(target_id, AlertState::Aggressive).expect("Failed to alert monster");
                }
            } else if let Some(turns) = confusion_turns {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 { targets.push(id); }
                }
                self.generate_noise(actual_target.0, actual_target.1, 4.0);
                for target_id in targets {
                    self.log.push(format!("The monster is confused by the {}!", item_name));
                    self.world.insert_one(target_id, Confusion { turns }).expect("Failed to insert Confusion component");
                    self.world.insert_one(target_id, AlertState::Aggressive).expect("Failed to alert monster");
                }
            } else if let Some(poison) = poison_effect {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 { targets.push(id); }
                }
                self.generate_noise(actual_target.0, actual_target.1, 4.0);
                for target_id in targets {
                    self.log.push(format!("The monster is poisoned by the {}!", item_name));
                    self.world.insert_one(target_id, poison).expect("Failed to insert Poison component");
                    self.world.insert_one(target_id, LastHitByPlayer).expect("Failed to insert LastHitByPlayer");
                    self.world.insert_one(target_id, AlertState::Aggressive).expect("Failed to alert monster");
                }
            } else {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 { targets.push(id); }
                }
                self.generate_noise(actual_target.0, actual_target.1, 6.0);
                for target_id in targets {
                    if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target_id) {
                        stats.hp -= power; self.log.push(format!("The {} hits for {} damage!", item_name, power));
                    }
                    if let Ok(t_pos) = self.world.get::<&Position>(target_id) {
                        self.effects.push(VisualEffect::Flash { x: t_pos.x, y: t_pos.y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
                    }
                    self.world.insert_one(target_id, LastHitByPlayer).expect("Failed to insert LastHitByPlayer");
                    self.world.insert_one(target_id, AlertState::Aggressive).expect("Failed to alert monster");
                }
            }

            let mut to_despawn = Vec::new();
            let mut total_xp: i32 = 0;
            for (id, (stats, _)) in self.world.query::<(&CombatStats, &Monster)>().iter() { 
                if stats.hp <= 0 { 
                    to_despawn.push(id); 
                    if let Ok(exp) = self.world.get::<&Experience>(id) {
                        total_xp = total_xp.saturating_add(exp.xp_reward);
                    }
                } 
            }
            for id in to_despawn { 
                self.world.despawn(id).expect("Failed to despawn monster"); 
                self.monsters_killed += 1;
            }
            
            if total_xp > 0 {
                self.add_player_xp(total_xp);
            }
            
            if is_ranged_weapon.is_none() {
                self.world.despawn(item_id).expect("Failed to despawn consumable item after use");
            }
            if self.state != RunState::LevelUp {
                self.state = RunState::MonsterTurn;
            }
            self.targeting_item = None;
        }
    }

    pub fn generate_noise(&mut self, x: u16, y: u16, amount: f32) {
        self.world.spawn((Position { x, y }, Noise { amount }));
    }

    pub fn on_turn_tick(&mut self) {
        let mut to_remove_confusion = Vec::new();
        let mut to_remove_poison = Vec::new();
        let mut to_remove_strength = Vec::new();
        let mut to_remove_speed = Vec::new();
        let mut to_despawn_noise = Vec::new();
        let mut poison_damage = Vec::new();
        let mut strength_expiration = Vec::new();

        for (id, _) in self.world.query::<&Noise>().iter() {
            to_despawn_noise.push(id);
        }
        for id in to_despawn_noise { self.world.despawn(id).expect("Failed to despawn noise"); }

        for (id, (_stats,)) in self.world.query::<(&CombatStats,)>().iter() {
            // Poison
            if let Ok(mut poison) = self.world.get::<&mut Poison>(id) {
                poison_damage.push((id, poison.damage));
                poison.turns -= 1;
                if poison.turns <= 0 { to_remove_poison.push(id); }
            }
            
            // Confusion
            if let Ok(mut confusion) = self.world.get::<&mut Confusion>(id) {
                confusion.turns -= 1;
                if confusion.turns <= 0 { to_remove_confusion.push(id); }
            }

            // Strength
            if let Ok(mut strength) = self.world.get::<&mut Strength>(id) {
                strength.turns -= 1;
                if strength.turns <= 0 { 
                    strength_expiration.push((id, strength.amount));
                    to_remove_strength.push(id); 
                }
            }

            // Speed
            if let Ok(mut speed) = self.world.get::<&mut Speed>(id) {
                speed.turns -= 1;
                if speed.turns <= 0 { to_remove_speed.push(id); }
            }
        }

        for (id, damage) in poison_damage {
            if let Ok(mut stats) = self.world.get::<&mut CombatStats>(id) {
                stats.hp -= damage;
                if self.world.get::<&Player>(id).is_ok() {
                    self.log.push(format!("You suffer {} damage from poison!", damage));
                    if stats.hp <= 0 { self.death = true; self.state = RunState::Dead; }
                }
            }
        }

        for (id, amount) in strength_expiration {
            if let Ok(mut stats) = self.world.get::<&mut CombatStats>(id) {
                stats.power -= amount;
                if self.world.get::<&Player>(id).is_ok() {
                    self.log.push("You feel your extra strength wear off.".to_string());
                }
            }
        }

        for id in to_remove_poison { self.world.remove_one::<Poison>(id).expect("Failed to remove Poison component"); }
        for id in to_remove_confusion { 
            self.world.remove_one::<Confusion>(id).expect("Failed to remove Confusion component"); 
            if self.world.get::<&Player>(id).is_ok() {
                self.log.push("You are no longer confused.".to_string());
            } else {
                self.log.push("A monster snaps out of confusion.".to_string());
            }
        }
        for id in to_remove_strength { self.world.remove_one::<Strength>(id).expect("Failed to remove Strength component"); }
        for id in to_remove_speed { self.world.remove_one::<Speed>(id).expect("Failed to remove Speed component"); }
        
        // Despawn dead monsters from poison
        let mut to_despawn = Vec::new();
        let mut total_xp: i32 = 0;
        for (id, (stats, _)) in self.world.query::<(&CombatStats, &Monster)>().iter() {
            if stats.hp <= 0 { 
                to_despawn.push(id); 
                if self.world.get::<&LastHitByPlayer>(id).is_ok() {
                    if let Ok(exp) = self.world.get::<&Experience>(id) {
                        total_xp = total_xp.saturating_add(exp.xp_reward);
                    }
                }
            }
        }
        for id in to_despawn { 
            let name = self.world.get::<&Name>(id).map(|n| n.0.clone()).unwrap_or("Monster".to_string());
            self.log.push(format!("{} dies from poison!", name));
            self.world.despawn(id).expect("Failed to despawn monster"); 
            self.monsters_killed += 1;
        }
        
        if total_xp > 0 {
            self.add_player_xp(total_xp);
        }
    }

    pub fn monster_turn(&mut self) {
        self.on_turn_tick();
        if self.state == RunState::Dead { return; }

        let player_id = self.get_player_id().expect("Player not found in monster_turn");

        if self.world.get::<&Speed>(player_id).is_ok() {
            if self.speed_toggle {
                self.speed_toggle = false;
                self.state = RunState::AwaitingInput;
                self.log.push("You move with supernatural speed!".to_string());
                return;
            }
        }
        self.speed_toggle = true;

        let mut actions = Vec::new();
        let mut actors: Vec<hecs::Entity> = self.world.query::<&Monster>().iter().map(|(id, _)| id).collect();
        for (id, _) in self.world.query::<&Merchant>().iter() { actors.push(id); }
        
        let mut wisp_moves = Vec::new();
        for (id, _) in self.world.query::<&Wisp>().iter() {
            let mut rng = rand::thread_rng();
            wisp_moves.push((id, rng.gen_range(-1..=1), rng.gen_range(-1..=1)));
        }

        for id in actors {
            let is_merchant = self.world.get::<&Merchant>(id).is_ok();
            
            // Boss Phase Triggering
            let mut boss_actions = Vec::new();
            if let Ok(mut boss) = self.world.get::<&mut Boss>(id) {
                if let Ok(stats) = self.world.get::<&CombatStats>(id) {
                    for phase in boss.phases.iter_mut() {
                        if !phase.triggered && stats.hp <= phase.hp_threshold {
                            phase.triggered = true;
                            boss_actions.push(phase.action);
                        }
                    }
                }
            }

            for action in boss_actions {
                let boss_name = self.world.get::<&Name>(id).map(|n| n.0.clone()).unwrap_or("Boss".to_string());
                match action {
                    BossPhaseAction::SummonMinions => {
                        self.log.push(format!("{} bellows: 'To my side, my children!'", boss_name));
                        let boss_pos = self.world.get::<&Position>(id).ok().map(|p| *p);
                        if let Some(pos) = boss_pos {
                            let minion_name = if boss_name.contains("Broodmother") { "Spider" } else { "Goblin" };
                            let minion_raw = self.content.monsters.iter().find(|m| m.name == minion_name).expect("Minion not found");
                            for (dx, dy) in &[(-1, -1), (1, -1), (-1, 1), (1, 1)] {
                                let (mx, my) = ((pos.x as i16 + dx).max(0) as u16, (pos.y as i16 + dy).max(0) as u16);
                                if !self.map.blocked[(my * self.map.width + mx) as usize] {
                                    crate::spawner::spawn_monster(&mut self.world, mx, my, minion_raw, self.dungeon_level);
                                }
                            }
                        }
                    }
                    BossPhaseAction::Enrage => {
                        self.log.push(format!("{} enters a bloodthirsty rage!", boss_name));
                        if let Ok(mut stats) = self.world.get::<&mut CombatStats>(id) {
                            stats.power += 4;
                            stats.defense += 2;
                        }
                    }
                }
            }

            let (pos, faction, personality, stats, viewshed, alert) = {
                if let (Ok(p), Ok(f), Ok(pers), Ok(s), Ok(v), Ok(a)) = (
                    self.world.get::<&Position>(id),
                    self.world.get::<&Faction>(id),
                    self.world.get::<&AIPersonality>(id),
                    self.world.get::<&CombatStats>(id),
                    self.world.get::<&Viewshed>(id),
                    self.world.get::<&AlertState>(id)
                ) {
                    (*p, *f, *pers, *s, v.visible_tiles, *a)
                } else { continue; }
            };

            if self.world.get::<&Confusion>(id).is_ok() {
                let mut rng = rand::thread_rng();
                actions.push((id, MonsterAction::Move(rng.gen_range(-1..=1), rng.gen_range(-1..=1))));
                continue;
            }

            let mut current_alert = alert;

            // 1. Check for player visibility (transition to Aggressive)
            let mut can_see_player = false;
            if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                let dist = (((pos.x as f32 - p_pos.x as f32).powi(2) + (pos.y as f32 - p_pos.y as f32).powi(2))).sqrt();
                if dist <= viewshed as f32 {
                    // Check LOS
                    let line = line2d(LineAlg::Bresenham, Point::new(pos.x, pos.y), Point::new(p_pos.x, p_pos.y));
                    let mut blocked = false;
                    for p in line.iter().skip(1).take(line.len() - 2) {
                        let idx = (p.y as u16 * self.map.width + p.x as u16) as usize;
                        if self.map.blocked[idx] { blocked = true; break; }
                    }
                    if !blocked { can_see_player = true; }
                }
            }

            if can_see_player {
                current_alert = AlertState::Aggressive;
                self.world.insert_one(id, AlertState::Aggressive).expect("Failed to update AlertState");
            }

            // 2. Check for noise if not Aggressive
            if current_alert != AlertState::Aggressive {
                let idx = (pos.y as u16 * self.map.width + pos.x as u16) as usize;
                let sound_level = self.map.sound[idx];
                if sound_level > 1.0 {
                    let mut noise_pos = None;
                    if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                        noise_pos = Some((*p_pos).clone());
                    }
                    if let Some(p_pos) = noise_pos {
                        current_alert = AlertState::Curious { x: p_pos.x, y: p_pos.y };
                        self.world.insert_one(id, current_alert).expect("Failed to update AlertState");
                    }
                }
            }

            if current_alert == AlertState::Sleeping {
                continue; // Sleeping monsters do nothing
            }

            // Find nearest target of a different faction
            let mut target = None;
            let mut min_dist = viewshed as f32 + 1.0;

            if current_alert == AlertState::Aggressive {
                // Check player
                if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                    if let Ok(p_faction) = self.world.get::<&Faction>(player_id) {
                        if faction.0 != p_faction.0 {
                            let dist = (((pos.x as f32 - p_pos.x as f32).powi(2) + (pos.y as f32 - p_pos.y as f32).powi(2))).sqrt();
                            if dist <= viewshed as f32 && dist < min_dist { min_dist = dist; target = Some((player_id, *p_pos)); }
                        }
                    }
                }

                // Check other monsters
                for (other_id, (other_pos, other_faction)) in self.world.query::<(&Position, &Faction)>().iter() {
                    if id == other_id { continue; }
                    if self.world.get::<&Wisp>(other_id).is_ok() { continue; } // Don't target wisps
                    if faction.0 != other_faction.0 {
                        let dist = (((pos.x as f32 - other_pos.x as f32).powi(2) + (pos.y as f32 - other_pos.y as f32).powi(2))).sqrt();
                        if dist <= viewshed as f32 && dist < min_dist { min_dist = dist; target = Some((other_id, *other_pos)); }
                    }
                }
            } else if let AlertState::Curious { x, y } = current_alert {
                // Move towards the noise
                let dist = (((pos.x as f32 - x as f32).powi(2) + (pos.y as f32 - y as f32).powi(2))).sqrt();
                if dist < 1.5 {
                    // Reached the spot, go back to sleeping (or standing guard)
                    self.world.insert_one(id, AlertState::Sleeping).expect("Failed to update AlertState");
                } else {
                    target = Some((id, Position { x, y })); // Dummy target id to indicate movement
                }
            }

            if let Some((target_id, target_pos)) = target {
                // Personality check
                let mut move_vec = None;
                let mut attack = false;

                if personality.0 == Personality::Cowardly && stats.hp < stats.max_hp / 2 && !is_merchant {
                    // Flee!
                    let mut dx = 0; let mut dy = 0;
                    if pos.x < target_pos.x { dx = -1; } else if pos.x > target_pos.x { dx = 1; }
                    if pos.y < target_pos.y { dy = -1; } else if pos.y > target_pos.y { dy = 1; }
                    move_vec = Some((dx, dy));
                } else if personality.0 == Personality::Tactical && min_dist < 4.0 && !is_merchant {
                    // Try to maintain distance if too close
                    let mut dx = 0; let mut dy = 0;
                    if pos.x < target_pos.x { dx = -1; } else if pos.x > target_pos.x { dx = 1; }
                    if pos.y < target_pos.y { dy = -1; } else if pos.y > target_pos.y { dy = 1; }
                    move_vec = Some((dx, dy));
                } else if min_dist < 1.5 {
                    attack = true;
                } else if !is_merchant {
                    let mut dx = 0; let mut dy = 0;
                    if pos.x < target_pos.x { dx = 1; } else if pos.x > target_pos.x { dx = -1; }
                    if pos.y < target_pos.y { dy = 1; } else if pos.y > target_pos.y { dy = -1; }
                    move_vec = Some((dx, dy));
                }

                if attack {
                    actions.push((id, MonsterAction::Attack(target_id)));
                } else if let Some((dx, dy)) = move_vec {
                    actions.push((id, MonsterAction::Move(dx, dy)));
                }

                // Ranged attack check (even if moving or fleeing, if tactical)
                if personality.0 == Personality::Tactical && min_dist > 1.5 && min_dist < 8.0 {
                    if self.world.get::<&RangedWeapon>(id).is_ok() {
                        // Check LOS
                        let line = line2d(
                            LineAlg::Bresenham,
                            Point::new(pos.x, pos.y),
                            Point::new(target_pos.x, target_pos.y)
                        );
                        let mut blocked = false;
                        for p in line.iter().skip(1).take(line.len() - 2) { // Skip own tile and target tile
                            let idx = (p.y as u16 * self.map.width + p.x as u16) as usize;
                            if self.map.blocked[idx] {
                                blocked = true;
                                break;
                            }
                        }
                        if !blocked {
                            actions.push((id, MonsterAction::RangedAttack(target_id)));
                        }
                    }
                }
            }
        }

        let mut occupied_positions: std::collections::HashSet<(u16, u16)> = self.world.query::<(&Position, &Monster)>().iter().map(|(_, (p, _))| (p.x, p.y)).collect();
        for (_, (p, _)) in self.world.query::<(&Position, &Merchant)>().iter() { occupied_positions.insert((p.x, p.y)); }
        if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
            occupied_positions.insert((p_pos.x, p_pos.y));
        }

        for (id, action) in actions {
            match action {
                MonsterAction::Move(dx, dy) => {
                    let (new_x, new_y) = { 
                        if let Ok(pos) = self.world.get::<&Position>(id) {
                            ((pos.x as i16 + dx).max(0) as u16, (pos.y as i16 + dy).max(0) as u16)
                        } else { continue; }
                    };
                    if !occupied_positions.contains(&(new_x, new_y)) && !self.map.blocked[(new_y as u16 * self.map.width + new_x as u16) as usize] {
                        if let Ok(mut pos) = self.world.get::<&mut Position>(id) {
                            occupied_positions.remove(&(pos.x, pos.y));
                            pos.x = new_x; pos.y = new_y;
                            occupied_positions.insert((new_x, new_y));
                        }
                    }
                }
                MonsterAction::Attack(target_id) => {
                    let (monster_name, monster_power) = { 
                        let stats = self.world.get::<&CombatStats>(id).expect("Monster has no stats"); 
                        let name = self.world.get::<&Name>(id).expect("Monster has no name"); 
                        (name.0.clone(), stats.power) 
                    };
                    let target_name = self.world.get::<&Name>(target_id).map(|n| n.0.clone()).unwrap_or("Something".to_string());
                    let target_defense = self.world.get::<&CombatStats>(target_id).map(|s| s.defense).unwrap_or(0);
                    let damage = (monster_power - target_defense).max(0);
                    
                    let target_hp = {
                        if let Ok(mut target_stats) = self.world.get::<&mut CombatStats>(target_id) {
                            target_stats.hp -= damage;
                            target_stats.hp
                        } else { 0 }
                    };
                    
                    if target_id == player_id {
                        self.log.push(format!("{} hits you for {} damage!", monster_name, damage));
                        if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                            self.effects.push(VisualEffect::Flash { x: p_pos.x, y: p_pos.y, glyph: '!', fg: Color::Red, bg: Some(Color::Indexed(232)), duration: 5 });
                        }
                        if target_hp <= 0 { self.log.push("You are dead!".to_string()); self.state = RunState::Dead; self.death = true; }
                    } else {
                        self.log.push(format!("{} hits {} for {} damage!", monster_name, target_name, damage));
                        if let Ok(t_pos) = self.world.get::<&Position>(target_id) {
                            self.effects.push(VisualEffect::Flash { x: t_pos.x, y: t_pos.y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
                        }
                        self.world.remove_one::<LastHitByPlayer>(target_id).ok(); // ok() because it might not be there
                        if target_hp <= 0 {
                            self.log.push(format!("{} dies!", target_name));
                        }
                    }
                }
                MonsterAction::RangedAttack(target_id) => {
                    let (monster_name, rw) = {
                        let name = self.world.get::<&Name>(id).expect("Monster has no name");
                        let r = self.world.get::<&RangedWeapon>(id).expect("Monster has no ranged weapon");
                        (name.0.clone(), *r)
                    };
                    let target_name = self.world.get::<&Name>(target_id).map(|n| n.0.clone()).unwrap_or("Something".to_string());
                    let target_defense = self.world.get::<&CombatStats>(target_id).map(|s| s.defense).unwrap_or(0);
                    let damage = (rw.damage_bonus - target_defense).max(0);

                    let target_hp = {
                        if let Ok(mut target_stats) = self.world.get::<&mut CombatStats>(target_id) {
                            target_stats.hp -= damage;
                            target_stats.hp
                        } else { 0 }
                    };

                    if target_id == player_id {
                        self.log.push(format!("{} fires at you for {} damage!", monster_name, damage));
                        if let Ok(p_pos) = self.world.get::<&Position>(player_id) {
                            self.effects.push(VisualEffect::Flash { x: p_pos.x, y: p_pos.y, glyph: '!', fg: Color::Red, bg: Some(Color::Indexed(232)), duration: 5 });
                        }
                        if target_hp <= 0 { self.log.push("You are dead!".to_string()); self.state = RunState::Dead; self.death = true; }
                    } else {
                        self.log.push(format!("{} fires at {} for {} damage!", monster_name, target_name, damage));
                        if let Ok(t_pos) = self.world.get::<&Position>(target_id) {
                            self.effects.push(VisualEffect::Flash { x: t_pos.x, y: t_pos.y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
                        }
                        self.world.remove_one::<LastHitByPlayer>(target_id).ok();
                        if target_hp <= 0 {
                            self.log.push(format!("{} dies!", target_name));
                        }
                    }
                    
                    // Add projectile animation
                    if let (Ok(m_pos), Ok(t_pos)) = (self.world.get::<&Position>(id), self.world.get::<&Position>(target_id)) {
                        let line = line2d(LineAlg::Bresenham, Point::new(m_pos.x, m_pos.y), Point::new(t_pos.x, t_pos.y));
                        let path: Vec<(u16, u16)> = line.iter().map(|p| (p.x as u16, p.y as u16)).collect();
                        self.effects.push(VisualEffect::Projectile { path, glyph: '*', fg: Color::Cyan, frame: 0, speed: 2 });
                    }
                }
            }
        }

        // Cleanup dead entities
        let mut to_despawn = Vec::new();
        let mut total_xp: i32 = 0;
        let mut drops = Vec::new();
        for (id, (stats, _)) in self.world.query::<(&CombatStats, &Monster)>().iter() {
            if stats.hp <= 0 { 
                to_despawn.push(id); 
                if self.world.get::<&LastHitByPlayer>(id).is_ok() {
                    if let Ok(exp) = self.world.get::<&Experience>(id) {
                        total_xp = total_xp.saturating_add(exp.xp_reward);
                    }
                }
                
                // Collect drop info
                if let Ok(name) = self.world.get::<&Name>(id) {
                    if let Some(pos) = self.world.get::<&Position>(id).ok() {
                        let boss_raw = self.content.monsters.iter().find(|m| m.name == name.0);
                        if let Some(raw) = boss_raw {
                            if let Some(loot_name) = &raw.guaranteed_loot {
                                if let Some(item_raw) = self.content.items.iter().find(|i| &i.name == loot_name) {
                                    drops.push((*pos, item_raw.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }
        for id in to_despawn { 
            self.world.despawn(id).expect("Failed to despawn monster"); 
            self.monsters_killed += 1;
        }

        for (pos, raw) in drops {
            crate::spawner::spawn_item(&mut self.world, pos.x, pos.y, &raw);
            self.log.push(format!("{} dropped {}!", "The boss", raw.name));
        }

        if total_xp > 0 {
            self.add_player_xp(total_xp);
        }

        for (id, dx, dy) in wisp_moves {
            let (new_x, new_y) = {
                if let Ok(pos) = self.world.get::<&Position>(id) {
                    ((pos.x as i16 + dx).clamp(0, self.map.width as i16 - 1) as u16, (pos.y as i16 + dy).clamp(0, self.map.height as i16 - 1) as u16)
                } else { continue; }
            };
            if !self.map.blocked[new_y as usize * self.map.width as usize + new_x as usize] {
                if let Ok(mut pos) = self.world.get::<&mut Position>(id) {
                    pos.x = new_x; pos.y = new_y;
                }
            }
        }

        if self.state != RunState::Dead && self.state != RunState::LevelUp { self.state = RunState::AwaitingInput; }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_map_get_tile() {
        let mut map = Map::new(10, 10);
        map.tiles[0] = TileType::Wall; map.tiles[1] = TileType::Floor;
        assert_eq!(map.get_tile(0, 0), TileType::Wall); assert_eq!(map.get_tile(1, 0), TileType::Floor);
    }
}
