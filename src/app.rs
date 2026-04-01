use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::layout::Rect as RatatuiRect;
use serde::{Deserialize, Serialize};
use crate::map_builder::MapBuilder;
use crate::components::*;
use hecs::World;
use bracket_pathfinding::prelude::*;
use rand::Rng;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
}

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
}

pub enum MonsterAction {
    Move(i16, i16),
    Attack(hecs::Entity),
    RangedAttack(hecs::Entity),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Map {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<TileType>,
    pub revealed: Vec<bool>,
    #[serde(skip)]
    pub visible: Vec<bool>,
    #[serde(skip)]
    pub blocked: Vec<bool>,
    #[serde(skip)]
    pub opaque: Vec<bool>,
    #[serde(skip)]
    pub light: Vec<f32>,
}

impl Map {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            tiles: vec![TileType::Wall; (width * height) as usize],
            revealed: vec![false; (width * height) as usize],
            visible: vec![false; (width * height) as usize],
            blocked: vec![false; (width * height) as usize],
            opaque: vec![false; (width * height) as usize],
            light: vec![0.0; (width * height) as usize],
        }
    }

    pub fn get_tile(&self, x: u16, y: u16) -> TileType {
        if x >= self.width || y >= self.height {
            return TileType::Wall;
        }
        self.tiles[(y * self.width + x) as usize]
    }

    pub fn populate_blocked_and_opaque(&mut self) {
        let size = self.width as usize * self.height as usize;
        self.blocked = vec![false; size];
        self.opaque = vec![false; size];
        for (i, tile) in self.tiles.iter().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
            self.opaque[i] = *tile == TileType::Wall;
        }
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        if idx >= self.opaque.len() { return true; }
        self.opaque[idx]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawMonster {
    pub name: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
    pub faction: FactionKind,
    pub personality: Personality,
    pub viewshed: i32,
    pub xp_reward: i32,
    pub ranged: Option<i32>,
    pub spawn_chance: f32,
    pub min_floor: u32,
    pub max_floor: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawItem {
    pub name: String,
    pub glyph: char,
    pub color: (u8, u8, u8),
    pub price: i32,
    pub potion: Option<i32>,
    pub weapon: Option<i32>,
    pub armor: Option<i32>,
    pub ranged: Option<i32>,
    pub ranged_weapon: Option<(i32, i32)>, // range, damage
    pub aoe: Option<i32>,
    pub confusion: Option<i32>,
    pub poison: Option<(i32, i32)>, // damage, turns
    pub ammo: bool,
    pub consumable: bool,
    pub spawn_chance: f32,
    pub min_floor: u32,
    pub max_floor: u32,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Content {
    pub monsters: Vec<RawMonster>,
    pub items: Vec<RawItem>,
}

/// A snapshot of an entity for serialization
#[derive(Serialize, Deserialize, Clone)]
pub struct EntitySnapshot {
    pub pos: Option<Position>,
    pub render: Renderable,
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
    pub levels: HashMap<u32, LevelData>,
    pub log: Vec<String>,
    pub dungeon_level: u32,
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
}

fn default_runstate() -> RunState { RunState::AwaitingInput }

fn apply_lighting(color: Color, intensity: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            Color::Rgb(
                (r as f32 * intensity).clamp(0.0, 255.0) as u8,
                (g as f32 * intensity).clamp(0.0, 255.0) as u8,
                (b as f32 * intensity).clamp(0.0, 255.0) as u8,
            )
        }
        Color::Indexed(i) => {
            if intensity < 0.2 { Color::Indexed(232) }
            else if intensity < 0.4 { Color::Indexed(236) }
            else if intensity < 0.6 { Color::Indexed(240) }
            else if intensity < 0.8 { Color::Indexed(244) }
            else if intensity < 1.0 { Color::Indexed(248) }
            else { Color::Indexed(i) }
        }
        _ => color,
    }
}

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
        };
        app.generate_level();
        app
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

    pub fn generate_level(&mut self) {
        let mut mb = MapBuilder::new(80, 50);
        mb.build(self.dungeon_level);
        self.map = mb.map;
        self.world = World::new();
        let mut rng = rand::thread_rng();

        self.world.spawn((
            Position { x: mb.player_start.0, y: mb.player_start.1 },
            Renderable { glyph: '@', fg: Color::Yellow },
            Player,
            Faction(FactionKind::Player),
            Viewshed { visible_tiles: 8 },
            LightSource { range: 6, color: (255, 255, 200) },
            Name("Player".to_string()),
            CombatStats { max_hp: 30, hp: 30, defense: 2, power: 5 },
            Experience { level: 1, xp: 0, next_level_xp: 50, xp_reward: 0 },
            Perks { traits: Vec::new() },
            Gold { amount: 0 },
        ));

        // Spawn ambient light sources (Glowing Crystals) in some rooms
        for (i, room) in mb.rooms.iter().enumerate().skip(1) {
            if i % 3 == 0 {
                let center = room.center();
                self.world.spawn((
                    Position { x: center.0 as u16, y: center.1 as u16 },
                    Renderable { glyph: '*', fg: Color::Rgb(100, 149, 237) }, // Cornflower Blue
                    LightSource { range: 4, color: (100, 149, 237) },
                    Name("Glowing Crystal".to_string()),
                ));
            }
            if i % 5 == 0 {
                let center = room.center();
                self.world.spawn((
                    Position { x: center.0 as u16, y: center.1 as u16 },
                    Renderable { glyph: '*', fg: Color::Cyan },
                    LightSource { range: 4, color: (0, 255, 255) },
                    Wisp,
                    Name("Dungeon Wisp".to_string()),
                ));
            }
        }

        self.world.spawn((
            Position { x: mb.stairs_down.0, y: mb.stairs_down.1 },
            Renderable { glyph: '>', fg: Color::White },
            DownStairs,
            Name("Down Stairs".to_string()),
        ));
        self.world.spawn((
            Position { x: mb.stairs_up.0, y: mb.stairs_up.1 },
            Renderable { glyph: '<', fg: Color::White },
            UpStairs,
            Name("Up Stairs".to_string()),
        ));

        let available_items: Vec<&RawItem> = self.content.items.iter()
            .filter(|i| self.dungeon_level >= i.min_floor && self.dungeon_level <= i.max_floor)
            .collect();

        // Spawn a Merchant in a random room (usually the second one)
        if mb.rooms.len() > 1 && !available_items.is_empty() {
            let room = &mb.rooms[1];
            let center = room.center();
            let merchant = self.world.spawn((
                Position { x: center.0 as u16, y: center.1 as u16 },
                Renderable { glyph: 'M', fg: Color::Rgb(255, 165, 0) },
                Merchant,
                Name("Merchant".to_string()),
                CombatStats { max_hp: 100, hp: 100, defense: 10, power: 10 },
                Faction(FactionKind::Player),
                Viewshed { visible_tiles: 8 },
                AIPersonality(Personality::Tactical),
            ));
            
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

                let raw = selected_item;
                let mut cb = hecs::EntityBuilder::new();
                cb.add(Item);
                cb.add(Name(raw.name.clone()));
                cb.add(Renderable { glyph: raw.glyph, fg: Color::Rgb(raw.color.0, raw.color.1, raw.color.2) });
                cb.add(InBackpack { owner: merchant });
                cb.add(ItemValue { price: raw.price });
                
                if let Some(h) = raw.potion { cb.add(Potion { heal_amount: h }); }
                if let Some(p) = raw.weapon { cb.add(Weapon { power_bonus: p }); }
                if let Some(d) = raw.armor { cb.add(Armor { defense_bonus: d }); }
                if let Some(r) = raw.ranged { cb.add(Ranged { range: r }); }
                if let Some((r, d)) = raw.ranged_weapon { cb.add(RangedWeapon { range: r, damage_bonus: d }); }
                if let Some(r) = raw.aoe { cb.add(AreaOfEffect { radius: r }); }
                if let Some(t) = raw.confusion { cb.add(Confusion { turns: t }); }
                if let Some((d, t)) = raw.poison { cb.add(Poison { damage: d, turns: t }); }
                if raw.ammo { cb.add(Ammunition); }
                if raw.consumable { cb.add(Consumable); }
                
                self.world.spawn(cb.build());
            }
        }

        for pos in &mb.door_spawns {
            self.world.spawn((
                Position { x: pos.0, y: pos.1 },
                Renderable { glyph: '+', fg: Color::Indexed(94) },
                Door { open: false },
                Name("Door".to_string()),
            ));
        }

        for pos in &mb.trap_spawns {
            self.world.spawn((
                Position { x: pos.0, y: pos.1 },
                Renderable { glyph: '^', fg: Color::Red },
                Trap { damage: 5, revealed: false },
                Name("Trap".to_string()),
            ));
        }

        let available_monsters: Vec<&RawMonster> = self.content.monsters.iter()
            .filter(|m| self.dungeon_level >= m.min_floor && self.dungeon_level <= m.max_floor)
            .collect();

        for spawn in &mb.monster_spawns {
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
            let hp = raw.hp + (self.dungeon_level as i32 * 2);
            let power = raw.power + (self.dungeon_level as i32 / 2);
            
            let mut cb = hecs::EntityBuilder::new();
            cb.add(Position { x: spawn.0, y: spawn.1 });
            cb.add(Renderable { glyph: raw.glyph, fg: Color::Rgb(raw.color.0, raw.color.1, raw.color.2) });
            cb.add(Monster);
            cb.add(Faction(raw.faction));
            cb.add(AIPersonality(raw.personality));
            cb.add(Viewshed { visible_tiles: raw.viewshed });
            cb.add(Name(raw.name.clone()));
            cb.add(CombatStats { max_hp: hp, hp, defense: raw.defense, power });
            cb.add(Experience { level: self.dungeon_level as i32, xp: 0, next_level_xp: 0, xp_reward: raw.xp_reward + (self.dungeon_level as i32 * 5) });
            
            if let Some(r) = raw.ranged {
                cb.add(RangedWeapon { range: r as i32, damage_bonus: power });
            }
            
            self.world.spawn(cb.build());
        }

        for spawn in &mb.item_spawns {
            // 20% chance for gold, otherwise pick item
            if available_items.is_empty() || rng.gen_bool(0.2) {
                let amount = rng.gen_range(5..25);
                self.world.spawn((
                    Position { x: spawn.0, y: spawn.1 },
                    Renderable { glyph: '*', fg: Color::Yellow },
                    Name(format!("{} Gold", amount)),
                    Gold { amount },
                ));
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

            let raw = selected_item;
            let mut cb = hecs::EntityBuilder::new();
            cb.add(Position { x: spawn.0, y: spawn.1 });
            cb.add(Renderable { glyph: raw.glyph, fg: Color::Rgb(raw.color.0, raw.color.1, raw.color.2) });
            cb.add(Item);
            cb.add(Name(raw.name.clone()));
            cb.add(ItemValue { price: raw.price });
            
            if let Some(h) = raw.potion { cb.add(Potion { heal_amount: h }); }
            if let Some(p) = raw.weapon { cb.add(Weapon { power_bonus: p }); }
            if let Some(d) = raw.armor { cb.add(Armor { defense_bonus: d }); }
            if let Some(r) = raw.ranged { cb.add(Ranged { range: r }); }
            if let Some((r, d)) = raw.ranged_weapon { cb.add(RangedWeapon { range: r, damage_bonus: d }); }
            if let Some(r) = raw.aoe { cb.add(AreaOfEffect { radius: r }); }
            if let Some(t) = raw.confusion { cb.add(Confusion { turns: t }); }
            if let Some((d, t)) = raw.poison { cb.add(Poison { damage: d, turns: t }); }
            if raw.ammo { cb.add(Ammunition); }
            if raw.consumable { cb.add(Consumable); }
            
            self.world.spawn(cb.build());
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
        let player_snapshot = current_entities.iter().find(|e| e.is_player).cloned().unwrap();
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
            self.generate_level();
        }
        self.log.push(format!("You descend to level {}.", self.dungeon_level));
    }

    pub fn go_up_level(&mut self) {
        if self.dungeon_level <= 1 {
            self.log.push("You cannot go further up.".to_string());
            return;
        }

        self.pack_entities();
        let current_entities = self.entities.clone();
        let player_snapshot = current_entities.iter().find(|e| e.is_player).cloned().unwrap();
        let level_entities: Vec<EntitySnapshot> = current_entities.into_iter().filter(|e| !e.is_player).collect();

        self.levels.insert(self.dungeon_level, LevelData {
            map: self.map.clone(),
            entities: level_entities,
        });

        self.dungeon_level -= 1;

        let level_data = self.levels.get(&self.dungeon_level).unwrap();
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

    pub fn update_fov(&mut self) {
        self.update_lighting();
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
        for (id, (render,)) in self.world.query::<(&Renderable,)>().iter() {
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
            let light_source = self.world.get::<&LightSource>(id).ok().map(|l| *l);
            let gold = self.world.get::<&Gold>(id).ok().map(|g| *g);
            let item_value = self.world.get::<&ItemValue>(id).ok().map(|v| *v);
            
            self.entities.push(EntitySnapshot {
                pos, render: *render, name, stats, potion, weapon, armor, door, trap, ranged, 
                ranged_weapon, aoe, confusion, poison, strength, speed,
                faction, viewshed, personality, experience, perks, light_source, gold, item_value,
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
                self.world.insert_one(id, InBackpack { owner: player }).unwrap();
            }
        }

        self.map.visible = vec![false; (self.map.width * self.map.height) as usize];
        self.update_blocked_and_opaque();
        self.update_fov();
    }

    pub fn add_player_xp(&mut self, xp: i32) {
        let player_id = self.world.query::<&Player>().iter().next().map(|(id, _)| id).unwrap();
        let mut level_up = false;
        
        if let Ok(mut exp) = self.world.get::<&mut Experience>(player_id) {
            exp.xp += xp;
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

            let mut dead = false;
            let monster_name = self.world.get::<&Name>(target_id).unwrap().0.clone();
            let mut xp_reward = 0;
            {
                let mut monster_stats = self.world.get::<&mut CombatStats>(target_id).unwrap();
                let damage = (player_power - monster_stats.defense).max(0);
                monster_stats.hp -= damage;
                self.log.push(format!("You hit {} for {} damage!", monster_name, damage));
                self.effects.push(VisualEffect::Flash { x: new_x, y: new_y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
                if monster_stats.hp <= 0 { 
                    dead = true; 
                    if let Ok(exp) = self.world.get::<&Experience>(target_id) {
                        xp_reward = exp.xp_reward;
                    }
                }
            }
            if !dead {
                self.world.insert_one(target_id, LastHitByPlayer).unwrap();
            }
            if dead {
                self.log.push(format!("{} dies!", monster_name));
                self.world.despawn(target_id).unwrap();
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
            let mut door = self.world.get::<&mut Door>(door_id).unwrap();
            door.open = true;
            let mut render = self.world.get::<&mut Renderable>(door_id).unwrap();
            render.glyph = '/';
            self.log.push("You open the door.".to_string());
            drop(door); drop(render);
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
                self.world.despawn(id).unwrap();
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
                let (_, (player_stats, _)) = stats_query.iter().next().unwrap();
                player_stats.hp -= total_damage;
                if player_stats.hp <= 0 { self.death = true; self.state = RunState::Dead; }
                drop(stats_query);
                for trap_id in triggered_traps { self.world.despawn(trap_id).unwrap(); }
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
            let item_name = self.world.get::<&Name>(item_id).unwrap().0.clone();
            self.world.remove_one::<Position>(item_id).unwrap();
            self.world.insert_one(item_id, InBackpack { owner: player_id }).unwrap();
            self.log.push(format!("You pick up the {}.", item_name));
            self.state = RunState::MonsterTurn;
        } else { self.log.push("There is nothing here to pick up.".to_string()); }
    }

    pub fn buy_item(&mut self, item_id: hecs::Entity) {
        let player_id = self.world.query::<&Player>().iter().next().map(|(id, _)| id).expect("Player not found");
        let price = self.world.get::<&ItemValue>(item_id).map(|v| v.price).unwrap_or(0);
        
        let can_afford = {
            let player_gold = self.world.get::<&Gold>(player_id).expect("Player has no gold component");
            player_gold.amount >= price
        };

        if can_afford {
            {
                let mut player_gold = self.world.get::<&mut Gold>(player_id).unwrap();
                player_gold.amount -= price;
            }
            let item_name = self.world.get::<&Name>(item_id).unwrap().0.clone();
            self.log.push(format!("You buy the {} for {} gold.", item_name, price));
            
            // Transfer item
            self.world.insert_one(item_id, InBackpack { owner: player_id }).unwrap();
        } else {
            self.log.push("You cannot afford that!".to_string());
        }
    }

    pub fn sell_item(&mut self, item_id: hecs::Entity) {
        let player_id = self.world.query::<&Player>().iter().next().map(|(id, _)| id).expect("Player not found");
        let price = self.world.get::<&ItemValue>(item_id).map(|v| v.price / 2).unwrap_or(1); // Sell for half price
        
        {
            let mut player_gold = self.world.get::<&mut Gold>(player_id).expect("Player has no gold component");
            player_gold.amount += price;
        }
        
        let item_name = self.world.get::<&Name>(item_id).unwrap().0.clone();
        self.log.push(format!("You sell the {} for {} gold.", item_name, price));
        
        self.world.despawn(item_id).unwrap();
    }

    pub fn use_item(&mut self, item_id: hecs::Entity) {
        let player_id = {
            let mut player_query = self.world.query::<(&Player,)>();
            let (id, (_,)) = player_query.iter().next().expect("Player not found");
            id
        };
        let item_name = self.world.get::<&Name>(item_id).unwrap().0.clone();
        
        let mut handled = false;
        if let Ok(potion) = self.world.get::<&Potion>(item_id) {
            let mut stats = self.world.get::<&mut CombatStats>(player_id).unwrap();
            stats.hp = (stats.hp + potion.heal_amount).min(stats.max_hp);
            self.log.push(format!("You drink the {}, healing for {} HP.", item_name, potion.heal_amount));
            handled = true;
        }

        let poison_effect = self.world.get::<&Poison>(item_id).ok().map(|p| *p);
        if let Some(poison) = poison_effect {
             self.world.insert_one(player_id, poison).unwrap();
             self.log.push(format!("You are poisoned by the {}!", item_name));
             handled = true;
        }
        
        let strength_effect = self.world.get::<&Strength>(item_id).ok().map(|s| *s);
        if let Some(strength) = strength_effect {
             self.world.insert_one(player_id, strength).unwrap();
             let mut stats = self.world.get::<&mut CombatStats>(player_id).unwrap();
             stats.power += strength.amount;
             self.log.push(format!("The {} makes you feel much stronger!", item_name));
             handled = true;
        }

        let speed_effect = self.world.get::<&Speed>(item_id).ok().map(|s| *s);
        if let Some(speed) = speed_effect {
             self.world.insert_one(player_id, speed).unwrap();
             self.log.push(format!("The {} makes you feel incredibly fast!", item_name));
             handled = true;
        }

        if item_name == "Torch" {
            self.world.insert_one(player_id, LightSource { range: 10, color: (255, 255, 100) }).unwrap();
            self.log.push("You light a torch. The shadows retreat.".to_string());
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
            let player_pos = self.world.get::<&Position>(player_id).unwrap();
            self.targeting_cursor = (player_pos.x, player_pos.y);
            self.targeting_item = Some(item_id);
            self.state = RunState::ShowTargeting;
            self.log.push(format!("Select target for {}...", item_name));
            return;
        } else if let Ok(weapon) = self.world.get::<&Weapon>(item_id) {
             self.log.push(format!("You equip the {}. Your power increases!", item_name));
             let mut stats = self.world.get::<&mut CombatStats>(player_id).unwrap();
             stats.power += weapon.power_bonus;
             handled = true;
        } else if let Ok(armor) = self.world.get::<&Armor>(item_id) {
             self.log.push(format!("You equip the {}. Your defense increases!", item_name));
             let mut stats = self.world.get::<&mut CombatStats>(player_id).unwrap();
             stats.defense += armor.defense_bonus;
             handled = true;
        }

        if handled { self.world.despawn(item_id).unwrap(); self.state = RunState::MonsterTurn; }
    }

    pub fn fire_targeting_item(&mut self) {
        if let Some(item_id) = self.targeting_item {
            let item_name = self.world.get::<&Name>(item_id).unwrap().0.clone();
            
            let (player_pos, player_id) = {
                let mut query = self.world.query::<(&Position, &Player)>();
                let (id, (pos, _)) = query.iter().next().expect("Player not found");
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
                    self.world.despawn(aid).unwrap();
                }
            }

            if let Some(radius) = aoe_radius {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    let dist = (((pos.x as f32 - actual_target.0 as f32).powi(2) + (pos.y as f32 - actual_target.1 as f32).powi(2))).sqrt();
                    if dist <= radius as f32 { targets.push(id); }
                }
                self.log.push(format!("The {} explodes!", item_name));
                for target_id in targets {
                    if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target_id) { stats.hp -= power; }
                    let t_pos = *self.world.get::<&Position>(target_id).unwrap();
                    self.effects.push(VisualEffect::Flash { x: t_pos.x, y: t_pos.y, glyph: '*', fg: Color::Indexed(208), bg: None, duration: 10 });
                    self.world.insert_one(target_id, LastHitByPlayer).unwrap();
                }
            } else if let Some(turns) = confusion_turns {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 { targets.push(id); }
                }
                for target_id in targets {
                    self.log.push(format!("The monster is confused by the {}!", item_name));
                    self.world.insert_one(target_id, Confusion { turns }).unwrap();
                    // Not damage, but maybe player caused it? Not needed for XP though.
                }
            } else if let Some(poison) = poison_effect {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 { targets.push(id); }
                }
                for target_id in targets {
                    self.log.push(format!("The monster is poisoned by the {}!", item_name));
                    self.world.insert_one(target_id, poison).unwrap();
                    self.world.insert_one(target_id, LastHitByPlayer).unwrap();
                }
            } else {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == actual_target.0 && pos.y == actual_target.1 { targets.push(id); }
                }
                for target_id in targets {
                    if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target_id) {
                        stats.hp -= power; self.log.push(format!("The {} hits for {} damage!", item_name, power));
                    }
                    let t_pos = *self.world.get::<&Position>(target_id).unwrap();
                    self.effects.push(VisualEffect::Flash { x: t_pos.x, y: t_pos.y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
                    self.world.insert_one(target_id, LastHitByPlayer).unwrap();
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
            for id in to_despawn { self.world.despawn(id).unwrap(); }
            
            if total_xp > 0 {
                self.add_player_xp(total_xp);
            }
            
            if is_ranged_weapon.is_none() {
                self.world.despawn(item_id).unwrap();
            }
            if self.state != RunState::LevelUp {
                self.state = RunState::MonsterTurn;
            }
            self.targeting_item = None;
        }
    }

    pub fn on_turn_tick(&mut self) {
        let mut to_remove_confusion = Vec::new();
        let mut to_remove_poison = Vec::new();
        let mut to_remove_strength = Vec::new();
        let mut to_remove_speed = Vec::new();
        let mut poison_damage = Vec::new();
        let mut strength_expiration = Vec::new();

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

        for id in to_remove_poison { self.world.remove_one::<Poison>(id).unwrap(); }
        for id in to_remove_confusion { 
            self.world.remove_one::<Confusion>(id).unwrap(); 
            if self.world.get::<&Player>(id).is_ok() {
                self.log.push("You are no longer confused.".to_string());
            } else {
                self.log.push("A monster snaps out of confusion.".to_string());
            }
        }
        for id in to_remove_strength { self.world.remove_one::<Strength>(id).unwrap(); }
        for id in to_remove_speed { self.world.remove_one::<Speed>(id).unwrap(); }
        
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
            let name = self.world.get::<&Name>(id).unwrap().0.clone();
            self.log.push(format!("{} dies from poison!", name));
            self.world.despawn(id).unwrap(); 
        }
        
        if total_xp > 0 {
            self.add_player_xp(total_xp);
        }
    }

    pub fn monster_turn(&mut self) {
        self.on_turn_tick();
        if self.state == RunState::Dead { return; }

        let player_id = self.world.query::<&Player>().iter().next().map(|(id, _)| id).unwrap();

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
            let (pos, faction, personality, stats, viewshed) = {
                let p = self.world.get::<&Position>(id).unwrap();
                let f = self.world.get::<&Faction>(id).unwrap();
                let pers = self.world.get::<&AIPersonality>(id).unwrap();
                let s = self.world.get::<&CombatStats>(id).unwrap();
                let v = self.world.get::<&Viewshed>(id).unwrap();
                (*p, *f, *pers, *s, v.visible_tiles)
            };

            if self.world.get::<&Confusion>(id).is_ok() {
                let mut rng = rand::thread_rng();
                actions.push((id, MonsterAction::Move(rng.gen_range(-1..=1), rng.gen_range(-1..=1))));
                continue;
            }

            // Find nearest target of a different faction
            let mut target = None;
            let mut min_dist = viewshed as f32 + 1.0;

            // Check player
            let p_pos = *self.world.get::<&Position>(player_id).unwrap();
            let p_faction = *self.world.get::<&Faction>(player_id).unwrap();
            if faction.0 != p_faction.0 {
                let dist = (((pos.x as f32 - p_pos.x as f32).powi(2) + (pos.y as f32 - p_pos.y as f32).powi(2))).sqrt();
                if dist <= viewshed as f32 && dist < min_dist { min_dist = dist; target = Some((player_id, p_pos)); }
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
        let p_pos = *self.world.get::<&Position>(player_id).unwrap();
        occupied_positions.insert((p_pos.x, p_pos.y));

        for (id, action) in actions {
            match action {
                MonsterAction::Move(dx, dy) => {
                    let (new_x, new_y) = { 
                        let pos = self.world.get::<&Position>(id).unwrap(); 
                        ((pos.x as i16 + dx).max(0) as u16, (pos.y as i16 + dy).max(0) as u16) 
                    };
                    if !occupied_positions.contains(&(new_x, new_y)) && !self.map.blocked[(new_y as u16 * self.map.width + new_x as u16) as usize] {
                        let mut pos = self.world.get::<&mut Position>(id).unwrap();
                        occupied_positions.remove(&(pos.x, pos.y));
                        pos.x = new_x; pos.y = new_y;
                        occupied_positions.insert((new_x, new_y));
                    }
                }
                MonsterAction::Attack(target_id) => {
                    let (monster_name, monster_power) = { 
                        let stats = self.world.get::<&CombatStats>(id).unwrap(); 
                        let name = self.world.get::<&Name>(id).unwrap(); 
                        (name.0.clone(), stats.power) 
                    };
                    let target_name = self.world.get::<&Name>(target_id).unwrap().0.clone();
                    let target_defense = self.world.get::<&CombatStats>(target_id).unwrap().defense;
                    let damage = (monster_power - target_defense).max(0);
                    
                    let target_hp = {
                        let mut target_stats = self.world.get::<&mut CombatStats>(target_id).unwrap();
                        target_stats.hp -= damage;
                        target_stats.hp
                    };
                    
                    if target_id == player_id {
                        self.log.push(format!("{} hits you for {} damage!", monster_name, damage));
                        self.effects.push(VisualEffect::Flash { x: p_pos.x, y: p_pos.y, glyph: '!', fg: Color::Red, bg: Some(Color::Indexed(232)), duration: 5 });
                        if target_hp <= 0 { self.log.push("You are dead!".to_string()); self.state = RunState::Dead; self.death = true; }
                    } else {
                        self.log.push(format!("{} hits {} for {} damage!", monster_name, target_name, damage));
                        let t_pos = *self.world.get::<&Position>(target_id).unwrap();
                        self.effects.push(VisualEffect::Flash { x: t_pos.x, y: t_pos.y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
                        self.world.remove_one::<LastHitByPlayer>(target_id).ok(); // ok() because it might not be there
                        if target_hp <= 0 {
                            self.log.push(format!("{} dies!", target_name));
                            // We can't despawn here easily without affecting the loop, so we'll check later
                        }
                    }
                }
                MonsterAction::RangedAttack(target_id) => {
                    let (monster_name, rw) = {
                        let name = self.world.get::<&Name>(id).unwrap();
                        let r = self.world.get::<&RangedWeapon>(id).unwrap();
                        (name.0.clone(), *r)
                    };
                    let target_name = self.world.get::<&Name>(target_id).unwrap().0.clone();
                    let target_defense = self.world.get::<&CombatStats>(target_id).unwrap().defense;
                    let damage = (rw.damage_bonus - target_defense).max(0);

                    let target_hp = {
                        let mut target_stats = self.world.get::<&mut CombatStats>(target_id).unwrap();
                        target_stats.hp -= damage;
                        target_stats.hp
                    };

                    if target_id == player_id {
                        self.log.push(format!("{} fires at you for {} damage!", monster_name, damage));
                        self.effects.push(VisualEffect::Flash { x: p_pos.x, y: p_pos.y, glyph: '!', fg: Color::Red, bg: Some(Color::Indexed(232)), duration: 5 });
                        if target_hp <= 0 { self.log.push("You are dead!".to_string()); self.state = RunState::Dead; self.death = true; }
                    } else {
                        self.log.push(format!("{} fires at {} for {} damage!", monster_name, target_name, damage));
                        let t_pos = *self.world.get::<&Position>(target_id).unwrap();
                        self.effects.push(VisualEffect::Flash { x: t_pos.x, y: t_pos.y, glyph: '*', fg: Color::Red, bg: None, duration: 5 });
                        self.world.remove_one::<LastHitByPlayer>(target_id).ok();
                        if target_hp <= 0 {
                            self.log.push(format!("{} dies!", target_name));
                        }
                    }
                    
                    // Add projectile animation
                    let m_pos = *self.world.get::<&Position>(id).unwrap();
                    let t_pos = *self.world.get::<&Position>(target_id).unwrap();
                    let line = line2d(LineAlg::Bresenham, Point::new(m_pos.x, m_pos.y), Point::new(t_pos.x, t_pos.y));
                    let path: Vec<(u16, u16)> = line.iter().map(|p| (p.x as u16, p.y as u16)).collect();
                    self.effects.push(VisualEffect::Projectile { path, glyph: '*', fg: Color::Cyan, frame: 0, speed: 2 });
                }
            }
        }

        // Cleanup dead entities
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
        for id in to_despawn { self.world.despawn(id).unwrap(); }

        if total_xp > 0 {
            self.add_player_xp(total_xp);
        }

        for (id, dx, dy) in wisp_moves {
            let (new_x, new_y) = {
                let pos = self.world.get::<&Position>(id).unwrap();
                ((pos.x as i16 + dx).clamp(0, self.map.width as i16 - 1) as u16, (pos.y as i16 + dy).clamp(0, self.map.height as i16 - 1) as u16)
            };
            if !self.map.blocked[new_y as usize * self.map.width as usize + new_x as usize] {
                let mut pos = self.world.get::<&mut Position>(id).unwrap();
                pos.x = new_x; pos.y = new_y;
            }
        }

        if self.state != RunState::Dead && self.state != RunState::LevelUp { self.state = RunState::AwaitingInput; }
    }

    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(0), Constraint::Length(6)]).split(frame.size());
        let top_chunks = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Min(0), Constraint::Length(30)]).split(chunks[0]);
        let map_area = top_chunks[0]; let sidebar_area = top_chunks[1]; let log_area = chunks[1];

        let map_block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Indexed(240))).title(Span::styled(" RustLike Dungeon ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        frame.render_widget(map_block, map_area);
        let inner_map = map_area.inner(&Margin { vertical: 1, horizontal: 1 });
        let buffer = frame.buffer_mut();

        let mut player_query = self.world.query::<(&Position, &Player, &CombatStats)>();
        let (_, (player_pos, _, player_stats)) = player_query.iter().next().expect("Player not found");

        let view_w = inner_map.width as i32; let view_h = inner_map.height as i32;
        let mut camera_x = player_pos.x as i32 - view_w / 2; let mut camera_y = player_pos.y as i32 - view_h / 2;
        camera_x = camera_x.clamp(0, (self.map.width as i32 - view_w).max(0)); 
        camera_y = camera_y.clamp(0, (self.map.height as i32 - view_h).max(0));

        for y in 0..view_h {
            let map_y = y + camera_y; 
            if map_y < 0 || map_y >= self.map.height as i32 { continue; }
            for x in 0..view_w {
                let map_x = x + camera_x; 
                if map_x < 0 || map_x >= self.map.width as i32 { continue; }
                let idx = map_y as usize * self.map.width as usize + map_x as usize;
                if !self.map.revealed[idx] { continue; }
                
                let light = self.map.light[idx];
                let is_visible = self.map.visible[idx];
                
                let (char, mut color) = match self.map.tiles[idx] {
                    TileType::Wall => ("#", if is_visible { Color::Indexed(252) } else { Color::Indexed(238) }),
                    TileType::Floor => (".", if is_visible { Color::Indexed(242) } else { Color::Indexed(234) }),
                };

                if is_visible {
                    color = apply_lighting(color, light.max(0.2)); // Minimum ambient light when in FOV
                } else {
                    color = apply_lighting(color, 0.1); // Very dim for revealed but not in FOV
                }

                buffer.get_mut(inner_map.x + x as u16, inner_map.y + y as u16).set_symbol(char).set_fg(color);
            }
        }

        for (id, (pos, render)) in self.world.query::<(&Position, &Renderable)>().iter() {
            let idx = pos.y as usize * self.map.width as usize + pos.x as usize;
            if !self.map.visible[idx] { continue; }
            
            let light = self.map.light[idx];
            if light < 0.1 && self.world.get::<&Player>(id).is_err() { continue; } // Hide monsters in total darkness

            if let Ok(trap) = self.world.get::<&Trap>(id) { if !trap.revealed { continue; } }
            let screen_x = pos.x as i32 - camera_x; let screen_y = pos.y as i32 - camera_y;
            if screen_x >= 0 && screen_x < view_w && screen_y >= 0 && screen_y < view_h {
                let color = apply_lighting(render.fg, light.max(0.3));
                let mut style = Style::default().fg(color);
                if self.world.get::<&Player>(id).is_ok() { style = style.add_modifier(Modifier::BOLD); }
                buffer.get_mut(inner_map.x + screen_x as u16, inner_map.y + screen_y as u16).set_symbol(&render.glyph.to_string()).set_style(style);
            }
        }

        // Render visual effects
        for effect in &self.effects {
            match effect {
                VisualEffect::Flash { x, y, glyph, fg, bg, .. } => {
                    let idx = *y as usize * self.map.width as usize + *x as usize;
                    if !self.map.visible[idx] { continue; }
                    let sx = *x as i32 - camera_x;
                    let sy = *y as i32 - camera_y;
                    if sx >= 0 && sx < view_w && sy >= 0 && sy < view_h {
                        let cell = buffer.get_mut(inner_map.x + sx as u16, inner_map.y + sy as u16);
                        cell.set_symbol(&glyph.to_string()).set_fg(*fg);
                        if let Some(bg_color) = bg { cell.set_bg(*bg_color); }
                    }
                }
                VisualEffect::Projectile { path, glyph, fg, frame, speed } => {
                    let path_idx = (*frame / *speed) as usize;
                    if let Some(pos) = path.get(path_idx) {
                        let idx = pos.1 as usize * self.map.width as usize + pos.0 as usize;
                        if !self.map.visible[idx] { continue; }
                        let sx = pos.0 as i32 - camera_x;
                        let sy = pos.1 as i32 - camera_y;
                        if sx >= 0 && sx < view_w && sy >= 0 && sy < view_h {
                            buffer.get_mut(inner_map.x + sx as u16, inner_map.y + sy as u16).set_symbol(&glyph.to_string()).set_fg(*fg);
                        }
                    }
                }
            }
        }

        if self.state == RunState::ShowTargeting {
            // Draw line from player to target
            let line = line2d(
                LineAlg::Bresenham, 
                Point::new(player_pos.x, player_pos.y), 
                Point::new(self.targeting_cursor.0, self.targeting_cursor.1)
            );

            for (i, p) in line.iter().enumerate() {
                let sx = p.x - camera_x;
                let sy = p.y - camera_y;
                if sx >= 0 && sx < view_w && sy >= 0 && sy < view_h {
                    let cell = buffer.get_mut(inner_map.x + sx as u16, inner_map.y + sy as u16);
                    if i == 0 {
                        // Player position, don't change it much
                    } else if i == line.len() - 1 {
                        cell.set_bg(Color::Cyan).set_fg(Color::Black);
                    } else {
                        cell.set_bg(Color::Indexed(236));
                    }
                }
            }
        }

        let sidebar = Block::default().borders(Borders::ALL).title(" Character ");
        let hp_percent = (player_stats.hp as f32 / player_stats.max_hp as f32 * 100.0) as u16;
        let hp_color = if hp_percent > 50 { Color::Green } else if hp_percent > 25 { Color::Yellow } else { Color::Red };
        let sidebar_layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)]).split(sidebar_area.inner(&Margin { vertical: 1, horizontal: 1 }));
        frame.render_widget(Gauge::default().block(Block::default().title("HP")).gauge_style(Style::default().fg(hp_color).bg(Color::Indexed(233))).percent(hp_percent).label(format!("{}/{}", player_stats.hp, player_stats.max_hp)), sidebar_layout[0]);
        frame.render_widget(Paragraph::new(format!("ATK: {}  DEF: {}", player_stats.power, player_stats.defense)), sidebar_layout[1]);
        
        let player_id = self.world.query::<&Player>().iter().next().map(|(id, _)| id).unwrap();
        let (level, xp, next_xp) = if let Ok(exp) = self.world.get::<&Experience>(player_id) {
            (exp.level, exp.xp, exp.next_level_xp)
        } else { (1, 0, 50) };

        frame.render_widget(Paragraph::new(format!("Level: {}  XP: {}/{}", level, xp, next_xp)), sidebar_layout[2]);
        
        // Status Effects / Gold
        let mut status_lines = Vec::new();
        let gold_amount = self.world.get::<&Gold>(player_id).map(|g| g.amount).unwrap_or(0);
        status_lines.push(Line::from(Span::styled(format!("Gold: {}", gold_amount), Style::default().fg(Color::Yellow))));
        
        if let Ok(poison) = self.world.get::<&Poison>(player_id) {
            status_lines.push(Line::from(Span::styled(format!("Poisoned ({})", poison.turns), Style::default().fg(Color::Green))));
        }
        if let Ok(strength) = self.world.get::<&Strength>(player_id) {
            status_lines.push(Line::from(Span::styled(format!("Strong ({})", strength.turns), Style::default().fg(Color::Yellow))));
        }
        if let Ok(speed) = self.world.get::<&Speed>(player_id) {
            status_lines.push(Line::from(Span::styled(format!("Fast ({})", speed.turns), Style::default().fg(Color::Cyan))));
        }
        if let Ok(confusion) = self.world.get::<&Confusion>(player_id) {
            status_lines.push(Line::from(Span::styled(format!("Confused ({})", confusion.turns), Style::default().fg(Color::Magenta))));
        }
        
        if let Ok(perks) = self.world.get::<&Perks>(player_id) {
            for perk in &perks.traits {
                let name = match perk {
                    Perk::Toughness => "Toughness",
                    Perk::EagleEye => "Eagle Eye",
                    Perk::Strong => "Strong",
                    Perk::ThickSkin => "Thick Skin",
                };
                status_lines.push(Line::from(Span::styled(format!("* {}", name), Style::default().fg(Color::LightBlue))));
            }
        }
        
        if !status_lines.is_empty() {
             let status_area = sidebar_layout[3];
             frame.render_widget(Paragraph::new(status_lines).block(Block::default().title(" Status/Perks ")), status_area);
        }

        frame.render_widget(sidebar, sidebar_area);

        let log_block = Block::default().borders(Borders::ALL).title(" Message Log ");
        let log_items: Vec<ListItem> = self.log.iter().rev().take(5).enumerate().map(|(i, s)| {
            let mut style = Style::default();
            if i == 0 { style = style.add_modifier(Modifier::BOLD).fg(Color::White); }
            else { style = style.fg(Color::Indexed(245)); }
            
            let mut fg = style.fg.unwrap_or(Color::White);
            if s.contains("damage") || s.contains("dies") || s.contains("dead") { fg = Color::Red; }
            else if s.contains("gold") || s.contains("buy") { fg = Color::Yellow; }
            else if s.contains("level") { fg = Color::Magenta; }
            else if s.contains("health") || s.contains("heal") { fg = Color::Green; }

            ListItem::new(Span::styled(s.clone(), Style::default().fg(fg).add_modifier(if i == 0 { Modifier::BOLD } else { Modifier::empty() })))
        }).collect();
        frame.render_widget(List::new(log_items).block(log_block), log_area);

        if self.state == RunState::ShowInventory { self.render_inventory(frame); }
        else if self.state == RunState::ShowHelp { self.render_help(frame); }
        else if self.state == RunState::Dead { self.render_death_screen(frame); }
        else if self.state == RunState::LevelUp { self.render_level_up(frame); }
        else if self.state == RunState::ShowShop { self.render_shop(frame); }
        else if self.state == RunState::ShowLogHistory { self.render_log_history(frame); }
        else if self.state == RunState::ShowBestiary { self.render_bestiary(frame); }
    }

    fn render_log_history(&self, frame: &mut Frame) {
        let area = centered_rect(80, 80, frame.size());
        frame.render_widget(Clear, area);
        let block = Block::default().borders(Borders::ALL).title(" Message History ");
        
        let log_items: Vec<ListItem> = self.log.iter().enumerate().map(|(i, s)| {
            let mut fg = Color::Indexed(245);
            if s.contains("damage") || s.contains("dies") || s.contains("dead") { fg = Color::Red; }
            else if s.contains("gold") || s.contains("buy") { fg = Color::Yellow; }
            else if s.contains("level") { fg = Color::Magenta; }
            else if s.contains("health") || s.contains("heal") { fg = Color::Green; }

            ListItem::new(Span::styled(format!("{}: {}", i + 1, s), Style::default().fg(fg)))
        }).collect();

        let mut state = ListState::default();
        let scroll_pos = if self.log.len() > area.height as usize - 2 {
            self.log_cursor
        } else { 0 };
        state.select(Some(scroll_pos));

        frame.render_stateful_widget(
            List::new(log_items)
                .block(block.title_bottom(Line::from(" [UP/DOWN] Scroll, [ESC] Close ").alignment(Alignment::Right)))
                .highlight_style(Style::default().bg(Color::Indexed(236)))
                .highlight_symbol("> "),
            area,
            &mut state
        );
    }

    fn render_bestiary(&self, frame: &mut Frame) {
        let area = centered_rect(80, 80, frame.size());
        frame.render_widget(Clear, area);
        let block = Block::default().borders(Borders::ALL).title(" Bestiary ");
        
        let mut encountered: Vec<String> = self.encountered_monsters.iter().cloned().collect();
        encountered.sort();

        if encountered.is_empty() {
            frame.render_widget(Paragraph::new("You haven't encountered any monsters yet.").block(block), area);
            return;
        }

        let list_items: Vec<ListItem> = encountered.iter().map(|name| ListItem::new(name.clone())).collect();
        let mut state = ListState::default();
        state.select(Some(self.bestiary_cursor));

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area.inner(&Margin { vertical: 1, horizontal: 1 }));

        frame.render_stateful_widget(
            List::new(list_items)
                .block(Block::default().borders(Borders::RIGHT))
                .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)),
            layout[0],
            &mut state
        );

        // Details side
        if let Some(selected_name) = encountered.get(self.bestiary_cursor) {
            let details = match selected_name.as_str() {
                "Orc" => "A common dungeon dweller. Fierce and aggressive. They tend to charge directly at you.",
                "Goblin" => "Small, weak, and cowardly. They often flee when their health is low.",
                "Goblin Archer" => "Keeps their distance and fires arrows. Try to corner them!",
                "Spider" => "Fast and dangerous. Their bites can be painful.",
                _ => "A mysterious inhabitant of the deep."
            };
            frame.render_widget(Paragraph::new(details).wrap(Wrap { trim: true }).block(Block::default().title(format!(" {} ", selected_name))), layout[1]);
        }

        frame.render_widget(block, area);
    }

    fn render_shop(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.size());
        frame.render_widget(Clear, area);
        
        let player_id = self.world.query::<&Player>().iter().next().map(|(id, _)| id).unwrap();
        let player_gold = self.world.get::<&Gold>(player_id).map(|g| g.amount).unwrap_or(0);
        
        let title = if self.shop_mode == 0 { format!(" Merchant Shop (Buy) - Your Gold: {} ", player_gold) } else { format!(" Merchant Shop (Sell) - Your Gold: {} ", player_gold) };
        let block = Block::default().borders(Borders::ALL).title(title);
        
        let items: Vec<(hecs::Entity, String, i32)> = if self.shop_mode == 0 {
            // Buy: Merchant's backpack
            if let Some(merchant_id) = self.active_merchant {
                self.world.query::<(&Item, &InBackpack, &Name, &ItemValue)>().iter()
                    .filter(|(_, (_, backpack, _, _))| backpack.owner == merchant_id)
                    .map(|(id, (_, _, name, value))| (id, name.0.clone(), value.price))
                    .collect()
            } else { Vec::new() }
        } else {
            // Sell: Player's backpack
            self.world.query::<(&Item, &InBackpack, &Name, &ItemValue)>().iter()
                .filter(|(_, (_, backpack, _, _))| backpack.owner == player_id)
                .map(|(id, (_, _, name, value))| (id, name.0.clone(), value.price / 2))
                .collect()
        };

        if items.is_empty() {
            frame.render_widget(Paragraph::new("Nothing here. (TAB to switch mode)").block(block), area);
        } else {
            let list_items: Vec<ListItem> = items.iter()
                .map(|(_, name, price)| ListItem::new(format!("{}: {}g", name, price)))
                .collect();
            
            let mut state = ListState::default();
            state.select(Some(self.shop_cursor));
            
            let footer = " [UP/DOWN] Select, [ENTER] Confirm, [TAB] Buy/Sell, [ESC] Leave ";
            frame.render_stateful_widget(
                List::new(list_items)
                    .block(block.title_bottom(Line::from(footer).alignment(Alignment::Right)))
                    .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
                    .highlight_symbol(">> "), 
                area, 
                &mut state
            );
        }
    }

    fn render_level_up(&self, frame: &mut Frame) {
        let area = centered_rect(50, 50, frame.size());
        frame.render_widget(Clear, area);
        let block = Block::default().borders(Borders::ALL).title(" Level Up! Choose a Perk ");
        
        let options = vec![
            ListItem::new("Toughness (+10 Max HP)"),
            ListItem::new("Eagle Eye (+2 FOV)"),
            ListItem::new("Strong (+2 Power)"),
            ListItem::new("Thick Skin (+1 Defense)"),
        ];

        let mut state = ListState::default(); state.select(Some(self.level_up_cursor));
        frame.render_stateful_widget(List::new(options).block(block).highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)).highlight_symbol(">> "), area, &mut state);
    }

    fn render_inventory(&self, frame: &mut Frame) {
        let area = centered_rect(70, 70, frame.size());
        frame.render_widget(Clear, area);
        let block = Block::default().borders(Borders::ALL).title(" Inventory ");
        
        let player_id = self.world.query::<&Player>().iter().next().map(|(id, _)| id).unwrap();
        let items: Vec<(hecs::Entity, String)> = self.world.query::<(&Item, &InBackpack, &Name)>().iter()
            .filter(|(_, (_, backpack, _))| backpack.owner == player_id)
            .map(|(id, (_, _, name))| (id, name.0.clone()))
            .collect();

        if items.is_empty() {
            frame.render_widget(Paragraph::new("Your backpack is empty.").block(block), area);
        } else {
            let list_items: Vec<ListItem> = items.iter().map(|(_, name)| ListItem::new(name.clone())).collect();
            let mut state = ListState::default();
            state.select(Some(self.inventory_cursor));

            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(area.inner(&Margin { vertical: 1, horizontal: 1 }));

            frame.render_stateful_widget(
                List::new(list_items)
                    .block(Block::default().borders(Borders::RIGHT))
                    .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)),
                layout[0],
                &mut state
            );

            // Item Details / Tooltip
            if let Some((item_id, _)) = items.get(self.inventory_cursor) {
                let mut tooltip = Vec::new();
                
                if let Ok(potion) = self.world.get::<&Potion>(*item_id) {
                    tooltip.push(Line::from(vec![Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw("Potion")]));
                    tooltip.push(Line::from(vec![Span::styled("Effect: ", Style::default().add_modifier(Modifier::BOLD)), Span::styled(format!("Heals {} HP", potion.heal_amount), Style::default().fg(Color::Green))]));
                }
                if let Ok(weapon) = self.world.get::<&Weapon>(*item_id) {
                    tooltip.push(Line::from(vec![Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw("Melee Weapon")]));
                    tooltip.push(Line::from(vec![Span::styled("Bonus: ", Style::default().add_modifier(Modifier::BOLD)), Span::styled(format!("+{} Power", weapon.power_bonus), Style::default().fg(Color::Red))]));
                }
                if let Ok(armor) = self.world.get::<&Armor>(*item_id) {
                    tooltip.push(Line::from(vec![Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw("Armor")]));
                    tooltip.push(Line::from(vec![Span::styled("Bonus: ", Style::default().add_modifier(Modifier::BOLD)), Span::styled(format!("+{} Defense", armor.defense_bonus), Style::default().fg(Color::Blue))]));
                }
                if let Ok(ranged) = self.world.get::<&Ranged>(*item_id) {
                    tooltip.push(Line::from(vec![Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw("Consumable Ranged")]));
                    tooltip.push(Line::from(vec![Span::styled("Range: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(format!("{}", ranged.range))]));
                }
                if let Ok(rw) = self.world.get::<&RangedWeapon>(*item_id) {
                    tooltip.push(Line::from(vec![Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw("Ranged Weapon")]));
                    tooltip.push(Line::from(vec![Span::styled("Range: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw(format!("{}", rw.range))]));
                    tooltip.push(Line::from(vec![Span::styled("Bonus: ", Style::default().add_modifier(Modifier::BOLD)), Span::styled(format!("+{} Damage", rw.damage_bonus), Style::default().fg(Color::Red))]));
                }
                if self.world.get::<&Ammunition>(*item_id).is_ok() {
                    tooltip.push(Line::from(vec![Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)), Span::raw("Ammunition")]));
                    tooltip.push(Line::from("Required for bows."));
                }
                if let Ok(aoe) = self.world.get::<&AreaOfEffect>(*item_id) {
                    tooltip.push(Line::from(vec![Span::styled("AoE Radius: ", Style::default().add_modifier(Modifier::BOLD)), Span::styled(format!("{}", aoe.radius), Style::default().fg(Color::Yellow))]));
                }
                if let Ok(poison) = self.world.get::<&Poison>(*item_id) {
                    tooltip.push(Line::from(vec![Span::styled("Poison: ", Style::default().add_modifier(Modifier::BOLD)), Span::styled(format!("{} damage for {} turns", poison.damage, poison.turns), Style::default().fg(Color::Green))]));
                }

                frame.render_widget(Paragraph::new(tooltip).block(Block::default().title(" Item Details ")), layout[1]);
            }

            frame.render_widget(block, area);
        }
    }

    fn render_help(&self, frame: &mut Frame) {
        let area = centered_rect(50, 60, frame.size()); frame.render_widget(Clear, area);
        let text = vec![
            Line::from(vec![Span::styled("Move:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" Arrows/HJKL")]),
            Line::from(vec![Span::styled("Pick Up:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" G")]),
            Line::from(vec![Span::styled("Inventory:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" I")]),
            Line::from(vec![Span::styled("Log History:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" M")]),
            Line::from(vec![Span::styled("Bestiary:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" B")]),
            Line::from(vec![Span::styled("Targeting:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" Arrows/HJKL + Enter")]),
            Line::from(vec![Span::styled("Help:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" ? or /")]),
            Line::from(vec![Span::styled("Quit:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" Q")]),
        ];
        frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).title(" Controls ")).alignment(Alignment::Center), area);
    }

    fn render_death_screen(&self, frame: &mut Frame) {
        let area = centered_rect(40, 20, frame.size()); frame.render_widget(Clear, area);
        let text = vec![Line::from(Span::styled("YOU HAVE PERISHED", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))), Line::from(""), Line::from("Press Q or Esc to exit.")];
        frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red))).alignment(Alignment::Center), area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: RatatuiRect) -> RatatuiRect {
    let popup_layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)]).split(r);
    Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)]).split(popup_layout[1])[1]
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
