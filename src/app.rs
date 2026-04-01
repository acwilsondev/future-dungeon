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
    Dead,
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
        }
    }

    pub fn get_tile(&self, x: u16, y: u16) -> TileType {
        if x >= self.width || y >= self.height {
            return TileType::Wall;
        }
        self.tiles[(y * self.width + x) as usize]
    }

    pub fn populate_blocked_and_opaque(&mut self) {
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
    pub aoe: Option<AreaOfEffect>,
    #[serde(default)]
    pub confusion: Option<Confusion>,
    #[serde(default)]
    pub consumable: bool,
    #[serde(default)]
    pub in_backpack: bool,
    pub is_player: bool,
    pub is_monster: bool,
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
        };
        app.generate_level();
        app
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
            Name("Player".to_string()),
            CombatStats { max_hp: 30, hp: 30, defense: 2, power: 5 },
        ));

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

        for spawn in &mb.monster_spawns {
            let hp = 10 + (self.dungeon_level as i32 * 2);
            let power = 3 + (self.dungeon_level as i32 / 2);
            self.world.spawn((
                Position { x: spawn.0, y: spawn.1 },
                Renderable { glyph: 'o', fg: Color::Red },
                Monster,
                Name("Orc".to_string()),
                CombatStats { max_hp: hp, hp, defense: 1, power },
            ));
        }

        for spawn in &mb.item_spawns {
            let roll = rng.gen_range(0..6);
            match roll {
                0 => {
                    self.world.spawn((
                        Position { x: spawn.0, y: spawn.1 },
                        Renderable { glyph: '!', fg: Color::Magenta },
                        Item,
                        Consumable,
                        Name("Health Potion".to_string()),
                        Potion { heal_amount: 8 },
                    ));
                }
                1 => {
                    self.world.spawn((
                        Position { x: spawn.0, y: spawn.1 },
                        Renderable { glyph: '/', fg: Color::Cyan },
                        Item,
                        Name("Dagger".to_string()),
                        Weapon { power_bonus: 2 },
                    ));
                }
                2 => {
                    self.world.spawn((
                        Position { x: spawn.0, y: spawn.1 },
                        Renderable { glyph: ')', fg: Color::Yellow },
                        Item,
                        Consumable,
                        Name("Magic Missile Scroll".to_string()),
                        Ranged { range: 6 },
                        CombatStats { max_hp: 0, hp: 0, defense: 0, power: 10 },
                    ));
                }
                3 => {
                    self.world.spawn((
                        Position { x: spawn.0, y: spawn.1 },
                        Renderable { glyph: ')', fg: Color::Red },
                        Item,
                        Consumable,
                        Name("Fire Scroll".to_string()),
                        Ranged { range: 6 },
                        AreaOfEffect { radius: 3 },
                        CombatStats { max_hp: 0, hp: 0, defense: 0, power: 15 },
                    ));
                }
                4 => {
                    self.world.spawn((
                        Position { x: spawn.0, y: spawn.1 },
                        Renderable { glyph: ')', fg: Color::Magenta },
                        Item,
                        Consumable,
                        Name("Confusion Scroll".to_string()),
                        Ranged { range: 6 },
                        Confusion { turns: 4 },
                    ));
                }
                _ => {
                    self.world.spawn((
                        Position { x: spawn.0, y: spawn.1 },
                        Renderable { glyph: '[', fg: Color::Green },
                        Item,
                        Name("Leather Armor".to_string()),
                        Armor { defense_bonus: 1 },
                    ));
                }
            }
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
            let (_, (pos, _)) = player_query.iter().next().unwrap();
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
        let (_, (pos, _)) = player_query.iter().next().unwrap();
        pos.x = down_stairs_pos.0;
        pos.y = down_stairs_pos.1;

        self.log.push(format!("You ascend to level {}.", self.dungeon_level));
    }

    pub fn update_fov(&mut self) {
        let mut player_query = self.world.query::<(&Position, &Player)>();
        let (_, (pos, _)) = player_query.iter().next().expect("Player not found");
        
        let fov = field_of_view(Point::new(pos.x, pos.y), 8, &self.map);
        
        for v in &mut self.map.visible { *v = false; }
        for p in fov {
            if p.x >= 0 && p.x < self.map.width as i32 && p.y >= 0 && p.y < self.map.height as i32 {
                let idx = (p.y as u16 * self.map.width + p.x as u16) as usize;
                self.map.visible[idx] = true;
                self.map.revealed[idx] = true;
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
            let aoe = self.world.get::<&AreaOfEffect>(id).ok().map(|a| *a);
            let confusion = self.world.get::<&Confusion>(id).ok().map(|c| *c);
            
            self.entities.push(EntitySnapshot {
                pos, render: *render, name, stats, potion, weapon, armor, door, trap, ranged, aoe, confusion,
                consumable: self.world.get::<&Consumable>(id).is_ok(),
                in_backpack: self.world.get::<&InBackpack>(id).is_ok(),
                is_player: self.world.get::<&Player>(id).is_ok(),
                is_monster: self.world.get::<&Monster>(id).is_ok(),
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
            if let Some(aoe) = e.aoe { cb.add(aoe); }
            if let Some(confusion) = e.confusion { cb.add(confusion); }
            if e.consumable { cb.add(Consumable); }
            if e.is_player { cb.add(Player); }
            if e.is_monster { cb.add(Monster); }
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

    pub fn move_player(&mut self, dx: i16, dy: i16) {
        let (new_x, new_y, player_power) = {
            let mut player_query = self.world.query::<(&Position, &Player, &CombatStats)>();
            let (_, (pos, _, player_stats)) = player_query.iter().next().expect("Player not found");
            ((pos.x as i16 + dx).max(0) as u16, (pos.y as i16 + dy).max(0) as u16, player_stats.power)
        };

        let mut target_monster = None;
        for (id, (m_pos, _, _)) in self.world.query::<(&Position, &Monster, &CombatStats)>().iter() {
            if m_pos.x == new_x && m_pos.y == new_y { target_monster = Some(id); break; }
        }

        if let Some(monster_id) = target_monster {
            let mut dead = false;
            let monster_name = self.world.get::<&Name>(monster_id).unwrap().0.clone();
            {
                let mut monster_stats = self.world.get::<&mut CombatStats>(monster_id).unwrap();
                let damage = (player_power - monster_stats.defense).max(0);
                monster_stats.hp -= damage;
                self.log.push(format!("You hit {} for {} damage!", monster_name, damage));
                if monster_stats.hp <= 0 { dead = true; }
            }
            if dead {
                self.log.push(format!("{} dies!", monster_name));
                self.world.despawn(monster_id).unwrap();
            }
            self.state = RunState::MonsterTurn;
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
            let (_, (pos, _)) = player_query.iter().next().expect("Player not found");
            pos.x = new_x; pos.y = new_y;
            drop(player_query);

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
        } else if self.world.get::<&Ranged>(item_id).is_ok() {
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
            let mut targets = Vec::new();
            
            // Collect info before mutations
            let aoe_radius = self.world.get::<&AreaOfEffect>(item_id).ok().map(|a| a.radius);
            let confusion_turns = self.world.get::<&Confusion>(item_id).ok().map(|c| c.turns);
            let power = self.world.get::<&CombatStats>(item_id).map(|s| s.power).unwrap_or(10);

            if let Some(radius) = aoe_radius {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    let dist = (((pos.x as i32 - self.targeting_cursor.0 as i32).pow(2) + (pos.y as i32 - self.targeting_cursor.1 as i32).pow(2)) as f32).sqrt();
                    if dist <= radius as f32 { targets.push(id); }
                }
                self.log.push(format!("The {} explodes!", item_name));
                for target_id in targets {
                    if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target_id) { stats.hp -= power; }
                }
            } else if let Some(turns) = confusion_turns {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == self.targeting_cursor.0 && pos.y == self.targeting_cursor.1 { targets.push(id); }
                }
                for target_id in targets {
                    self.log.push(format!("The monster is confused by the {}!", item_name));
                    self.world.insert_one(target_id, Confusion { turns }).unwrap();
                }
            } else {
                for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
                    if pos.x == self.targeting_cursor.0 && pos.y == self.targeting_cursor.1 { targets.push(id); }
                }
                for target_id in targets {
                    if let Ok(mut stats) = self.world.get::<&mut CombatStats>(target_id) {
                        stats.hp -= power; self.log.push(format!("The {} hits for {} damage!", item_name, power));
                    }
                }
            }

            let mut to_despawn = Vec::new();
            for (id, (stats, _)) in self.world.query::<(&CombatStats, &Monster)>().iter() { if stats.hp <= 0 { to_despawn.push(id); } }
            for id in to_despawn { self.world.despawn(id).unwrap(); }
            self.world.despawn(item_id).unwrap();
            self.targeting_item = None; self.state = RunState::MonsterTurn;
        }
    }

    pub fn monster_turn(&mut self) {
        let (player_pos, player_id) = {
            let mut player_query = self.world.query::<(&Position, &Player)>();
            let (id, (pos, _)) = player_query.iter().next().expect("Player not found");
            (*pos, id)
        };
        let mut actions = Vec::new();
        let mut to_remove_confusion = Vec::new();

        for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
            if let Ok(mut confusion) = self.world.get::<&mut Confusion>(id) {
                confusion.turns -= 1;
                if confusion.turns <= 0 { to_remove_confusion.push(id); }
                else { let mut rng = rand::thread_rng(); actions.push((id, Some((rng.gen_range(-1..=1), rng.gen_range(-1..=1))))); continue; }
            }
            let idx = (pos.y * self.map.width + pos.x) as usize;
            if !self.map.visible[idx] { continue; }
            let distance = (((pos.x as i32 - player_pos.x as i32).pow(2) + (pos.y as i32 - player_pos.y as i32).pow(2)) as f32).sqrt();
            if distance < 1.5 { actions.push((id, None)); }
            else {
                let mut dx = 0; let mut dy = 0;
                if pos.x < player_pos.x { dx = 1; } else if pos.x > player_pos.x { dx = -1; }
                if pos.y < player_pos.y { dy = 1; } else if pos.y > player_pos.y { dy = -1; }
                actions.push((id, Some((dx, dy))));
            }
        }
        for id in to_remove_confusion { self.world.remove_one::<Confusion>(id).unwrap(); self.log.push("A monster snaps out of confusion.".to_string()); }

        let mut occupied_positions: std::collections::HashSet<(u16, u16)> = self.world.query::<(&Position, &Monster)>().iter().map(|(_, (p, _))| (p.x, p.y)).collect();
        occupied_positions.insert((player_pos.x, player_pos.y));

        for (id, action) in actions {
            if let Some((dx, dy)) = action {
                let (new_x, new_y) = { let pos = self.world.get::<&Position>(id).unwrap(); ((pos.x as i16 + dx).max(0) as u16, (pos.y as i16 + dy).max(0) as u16) };
                if !occupied_positions.contains(&(new_x, new_y)) && !self.map.blocked[(new_y as u16 * self.map.width + new_x as u16) as usize] {
                    let mut pos = self.world.get::<&mut Position>(id).unwrap();
                    occupied_positions.remove(&(pos.x, pos.y)); pos.x = new_x; pos.y = new_y; occupied_positions.insert((new_x, new_y));
                }
            } else {
                let (monster_name, monster_power) = { let stats = self.world.get::<&CombatStats>(id).unwrap(); let name = self.world.get::<&Name>(id).unwrap(); (name.0.clone(), stats.power) };
                let player_defense = self.world.get::<&CombatStats>(player_id).unwrap().defense;
                let damage = (monster_power - player_defense).max(0);
                let mut player_stats = self.world.get::<&mut CombatStats>(player_id).unwrap();
                player_stats.hp -= damage; self.log.push(format!("{} hits you for {} damage!", monster_name, damage));
                if player_stats.hp <= 0 { self.log.push("You are dead!".to_string()); self.state = RunState::Dead; self.death = true; }
            }
        }
        if self.state != RunState::Dead { self.state = RunState::AwaitingInput; }
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
        camera_x = camera_x.clamp(0, (self.map.width as i32 - view_w).max(0)); camera_y = camera_y.clamp(0, (self.map.height as i32 - view_h).max(0));

        for y in 0..view_h {
            let map_y = y + camera_y; if map_y >= self.map.height as i32 { break; }
            for x in 0..view_w {
                let map_x = x + camera_x; if map_x >= self.map.width as i32 { break; }
                let idx = (map_y as u16 * self.map.width + map_x as u16) as usize;
                if !self.map.revealed[idx] { continue; }
                let (char, color) = match self.map.tiles[idx] {
                    TileType::Wall => ("#", if self.map.visible[idx] { Color::Indexed(252) } else { Color::Indexed(238) }),
                    TileType::Floor => (".", if self.map.visible[idx] { Color::Indexed(242) } else { Color::Indexed(234) }),
                };
                buffer.get_mut(inner_map.x + x as u16, inner_map.y + y as u16).set_symbol(char).set_fg(color);
            }
        }

        for (id, (pos, render)) in self.world.query::<(&Position, &Renderable)>().iter() {
            let idx = (pos.y * self.map.width + pos.x) as usize;
            if !self.map.visible[idx] { continue; }
            if let Ok(trap) = self.world.get::<&Trap>(id) { if !trap.revealed { continue; } }
            let screen_x = pos.x as i32 - camera_x; let screen_y = pos.y as i32 - camera_y;
            if screen_x >= 0 && screen_x < view_w && screen_y >= 0 && screen_y < view_h {
                let mut style = Style::default().fg(render.fg);
                if self.world.get::<&Player>(id).is_ok() { style = style.add_modifier(Modifier::BOLD); }
                buffer.get_mut(inner_map.x + screen_x as u16, inner_map.y + screen_y as u16).set_symbol(&render.glyph.to_string()).set_style(style);
            }
        }

        if self.state == RunState::ShowTargeting {
            let screen_x = self.targeting_cursor.0 as i32 - camera_x; let screen_y = self.targeting_cursor.1 as i32 - camera_y;
            if screen_x >= 0 && screen_x < view_w && screen_y >= 0 && screen_y < view_h {
                buffer.get_mut(inner_map.x + screen_x as u16, inner_map.y + screen_y as u16).set_bg(Color::Cyan).set_fg(Color::Black);
            }
        }

        let sidebar = Block::default().borders(Borders::ALL).title(" Character ");
        let hp_percent = (player_stats.hp as f32 / player_stats.max_hp as f32 * 100.0) as u16;
        let hp_color = if hp_percent > 50 { Color::Green } else if hp_percent > 25 { Color::Yellow } else { Color::Red };
        let sidebar_layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(3), Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)]).split(sidebar_area.inner(&Margin { vertical: 1, horizontal: 1 }));
        frame.render_widget(Gauge::default().block(Block::default().title("HP")).gauge_style(Style::default().fg(hp_color).bg(Color::Indexed(233))).percent(hp_percent).label(format!("{}/{}", player_stats.hp, player_stats.max_hp)), sidebar_layout[0]);
        frame.render_widget(Paragraph::new(format!("ATK: {}  DEF: {}", player_stats.power, player_stats.defense)), sidebar_layout[1]);
        frame.render_widget(Paragraph::new(format!("Depth: Level {}", self.dungeon_level)), sidebar_layout[2]);
        frame.render_widget(sidebar, sidebar_area);

        let log_block = Block::default().borders(Borders::ALL).title(" Message Log ");
        let log_items: Vec<ListItem> = self.log.iter().rev().take(5).map(|s| ListItem::new(s.clone())).collect();
        frame.render_widget(List::new(log_items).block(log_block), log_area);

        if self.state == RunState::ShowInventory { self.render_inventory(frame); }
        else if self.state == RunState::ShowHelp { self.render_help(frame); }
        else if self.state == RunState::Dead { self.render_death_screen(frame); }
    }

    fn render_inventory(&self, frame: &mut Frame) {
        let area = centered_rect(60, 60, frame.size());
        frame.render_widget(Clear, area);
        let block = Block::default().borders(Borders::ALL).title(" Inventory ");
        let items: Vec<ListItem> = self.world.query::<(&Item, &InBackpack, &Name)>().iter().map(|(_, (_, _, name))| ListItem::new(name.0.clone())).collect();
        if items.is_empty() { frame.render_widget(Paragraph::new("Your backpack is empty.").block(block), area); }
        else {
            let mut state = ListState::default(); state.select(Some(self.inventory_cursor));
            frame.render_stateful_widget(List::new(items).block(block).highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)).highlight_symbol(">> "), area, &mut state);
        }
    }

    fn render_help(&self, frame: &mut Frame) {
        let area = centered_rect(50, 50, frame.size()); frame.render_widget(Clear, area);
        let text = vec![
            Line::from(vec![Span::styled("Move:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" Arrows/HJKL")]),
            Line::from(vec![Span::styled("Pick Up:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" G")]),
            Line::from(vec![Span::styled("Inventory:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" I")]),
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
