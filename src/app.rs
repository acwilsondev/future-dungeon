use ratatui::prelude::*;
use ratatui::widgets::*;
use ratatui::layout::Rect as RatatuiRect;
use serde::{Deserialize, Serialize};
use crate::map_builder::MapBuilder;
use crate::components::*;
use hecs::World;
use bracket_pathfinding::prelude::*;
use rand::Rng;

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
    Dead,
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<TileType>,
    pub revealed: Vec<bool>,
    #[serde(skip)]
    pub visible: Vec<bool>,
}

impl Map {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            tiles: vec![TileType::Wall; (width * height) as usize],
            revealed: vec![false; (width * height) as usize],
            visible: vec![false; (width * height) as usize],
        }
    }

    pub fn get_tile(&self, x: u16, y: u16) -> TileType {
        if x >= self.width || y >= self.height {
            return TileType::Wall;
        }
        self.tiles[(y * self.width + x) as usize]
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }
}

/// A snapshot of an entity for serialization
#[derive(Serialize, Deserialize)]
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
    pub in_backpack: bool,
    pub is_player: bool,
    pub is_monster: bool,
    #[serde(default)]
    pub is_item: bool,
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
    pub log: Vec<String>,
    pub dungeon_level: u32,
    #[serde(skip, default = "default_runstate")]
    pub state: RunState,
    #[serde(skip)]
    pub inventory_cursor: usize,
}

fn default_runstate() -> RunState { RunState::AwaitingInput }

impl App {
    pub fn new() -> Self {
        Self::new_random()
    }

    pub fn new_random() -> Self {
        let mut mb = MapBuilder::new(80, 50);
        mb.build();
        let mut world = World::new();
        let mut rng = rand::thread_rng();
        
        // Spawn player
        world.spawn((
            Position { x: mb.player_start.0, y: mb.player_start.1 },
            Renderable { glyph: '@', fg: Color::Yellow },
            Player,
            Name("Player".to_string()),
            CombatStats { max_hp: 30, hp: 30, defense: 2, power: 5 },
        ));

        // Spawn monsters
        for spawn in &mb.monster_spawns {
            world.spawn((
                Position { x: spawn.0, y: spawn.1 },
                Renderable { glyph: 'o', fg: Color::Red },
                Monster,
                Name("Orc".to_string()),
                CombatStats { max_hp: 10, hp: 10, defense: 1, power: 3 },
            ));
        }

        // Spawn items
        for spawn in &mb.item_spawns {
            let roll = rng.gen_range(0..3);
            match roll {
                0 => {
                    world.spawn((
                        Position { x: spawn.0, y: spawn.1 },
                        Renderable { glyph: '!', fg: Color::Magenta },
                        Item,
                        Name("Health Potion".to_string()),
                        Potion { heal_amount: 8 },
                    ));
                }
                1 => {
                    world.spawn((
                        Position { x: spawn.0, y: spawn.1 },
                        Renderable { glyph: '/', fg: Color::Cyan },
                        Item,
                        Name("Dagger".to_string()),
                        Weapon { power_bonus: 2 },
                    ));
                }
                _ => {
                    world.spawn((
                        Position { x: spawn.0, y: spawn.1 },
                        Renderable { glyph: '[', fg: Color::Green },
                        Item,
                        Name("Leather Armor".to_string()),
                        Armor { defense_bonus: 1 },
                    ));
                }
            }
        }

        let mut app = Self {
            exit: false,
            death: false,
            world,
            map: mb.map,
            entities: Vec::new(),
            log: vec!["Welcome to RustLike!".to_string()],
            dungeon_level: 1,
            state: RunState::AwaitingInput,
            inventory_cursor: 0,
        };
        app.update_fov();
        app
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
            let is_player = self.world.get::<&Player>(id).is_ok();
            let is_monster = self.world.get::<&Monster>(id).is_ok();
            let is_item = self.world.get::<&Item>(id).is_ok();
            let name = self.world.get::<&Name>(id).ok().map(|n| (*n).clone());
            let stats = self.world.get::<&CombatStats>(id).ok().map(|s| *s);
            let potion = self.world.get::<&Potion>(id).ok().map(|p| *p);
            let weapon = self.world.get::<&Weapon>(id).ok().map(|w| *w);
            let armor = self.world.get::<&Armor>(id).ok().map(|a| *a);
            
            self.entities.push(EntitySnapshot {
                pos,
                render: *render,
                name,
                stats,
                potion,
                weapon,
                armor,
                in_backpack: self.world.get::<&InBackpack>(id).is_ok(),
                is_player,
                is_monster,
                is_item,
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
            if e.is_player { cb.add(Player); }
            if e.is_monster { cb.add(Monster); }
            if e.is_item { cb.add(Item); }
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
        self.update_fov();
    }

    pub fn move_player(&mut self, dx: i16, dy: i16) {
        let (new_x, new_y, player_power) = {
            let mut player_query = self.world.query::<(&Position, &Player, &CombatStats)>();
            let (_, (pos, _, player_stats)) = player_query.iter().next().expect("Player not found");
            (
                (pos.x as i16 + dx).max(0) as u16,
                (pos.y as i16 + dy).max(0) as u16,
                player_stats.power,
            )
        };

        let mut target_monster = None;
        for (id, (m_pos, _, _)) in self.world.query::<(&Position, &Monster, &CombatStats)>().iter() {
            if m_pos.x == new_x && m_pos.y == new_y {
                target_monster = Some(id);
                break;
            }
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

        if self.map.get_tile(new_x, new_y) == TileType::Floor {
            let mut player_query = self.world.query::<(&mut Position, &Player)>();
            let (_, (pos, _)) = player_query.iter().next().expect("Player not found");
            pos.x = new_x;
            pos.y = new_y;
            drop(player_query);
            self.update_fov();
            self.state = RunState::MonsterTurn;
        }
    }

    pub fn pick_up_item(&mut self) {
        let (player_pos, player_id) = {
            let mut player_query = self.world.query::<(&Position, &Player)>();
            let (id, (pos, _)) = player_query.iter().next().expect("Player not found");
            (*pos, id)
        };

        let mut item_to_pick = None;
        for (id, (pos, _)) in self.world.query::<(&Position, &Item)>().iter() {
            if pos.x == player_pos.x && pos.y == player_pos.y {
                item_to_pick = Some(id);
                break;
            }
        }

        if let Some(item_id) = item_to_pick {
            let item_name = self.world.get::<&Name>(item_id).unwrap().0.clone();
            self.world.remove_one::<Position>(item_id).unwrap();
            self.world.insert_one(item_id, InBackpack { owner: player_id }).unwrap();
            self.log.push(format!("You pick up the {}.", item_name));
            self.state = RunState::MonsterTurn;
        } else {
            self.log.push("There is nothing here to pick up.".to_string());
        }
    }

    pub fn use_item(&mut self, item_id: hecs::Entity) {
        let player_id = {
            let mut player_query = self.world.query::<(&Player,)>();
            let (id, (_,)) = player_query.iter().next().expect("Player not found");
            id
        };

        let item_name = self.world.get::<&Name>(item_id).unwrap().0.clone();
        let mut despawn_item = false;

        if let Ok(potion) = self.world.get::<&Potion>(item_id) {
            let mut stats = self.world.get::<&mut CombatStats>(player_id).unwrap();
            stats.hp = (stats.hp + potion.heal_amount).min(stats.max_hp);
            self.log.push(format!("You drink the {}, healing for {} HP.", item_name, potion.heal_amount));
            despawn_item = true;
        } else if let Ok(weapon) = self.world.get::<&Weapon>(item_id) {
             self.log.push(format!("You equip the {}. Your power increases!", item_name));
             let mut stats = self.world.get::<&mut CombatStats>(player_id).unwrap();
             stats.power += weapon.power_bonus;
             despawn_item = true;
        } else if let Ok(armor) = self.world.get::<&Armor>(item_id) {
             self.log.push(format!("You equip the {}. Your defense increases!", item_name));
             let mut stats = self.world.get::<&mut CombatStats>(player_id).unwrap();
             stats.defense += armor.defense_bonus;
             despawn_item = true;
        }

        if despawn_item {
            self.world.despawn(item_id).unwrap();
            self.state = RunState::MonsterTurn;
        }
    }

    pub fn monster_turn(&mut self) {
        let (player_pos, player_id) = {
            let mut player_query = self.world.query::<(&Position, &Player)>();
            let (id, (pos, _)) = player_query.iter().next().expect("Player not found");
            (*pos, id)
        };

        let mut actions = Vec::new();

        for (id, (pos, _)) in self.world.query::<(&Position, &Monster)>().iter() {
            let idx = (pos.y * self.map.width + pos.x) as usize;
            if !self.map.visible[idx] { continue; }

            let distance = (((pos.x as i32 - player_pos.x as i32).pow(2) + (pos.y as i32 - player_pos.y as i32).pow(2)) as f32).sqrt();

            if distance < 1.5 {
                actions.push((id, None));
            } else {
                let mut dx = 0;
                let mut dy = 0;
                if pos.x < player_pos.x { dx = 1; } else if pos.x > player_pos.x { dx = -1; }
                if pos.y < player_pos.y { dy = 1; } else if pos.y > player_pos.y { dy = -1; }
                actions.push((id, Some((dx, dy))));
            }
        }

        let mut occupied_positions: std::collections::HashSet<(u16, u16)> = self.world
            .query::<(&Position, &Monster)>()
            .iter()
            .map(|(_, (p, _))| (p.x, p.y))
            .collect();
        occupied_positions.insert((player_pos.x, player_pos.y));

        for (id, action) in actions {
            if let Some((dx, dy)) = action {
                let (new_x, new_y) = {
                    let pos = self.world.get::<&Position>(id).unwrap();
                    ((pos.x as i16 + dx).max(0) as u16, (pos.y as i16 + dy).max(0) as u16)
                };

                if !occupied_positions.contains(&(new_x, new_y)) && self.map.get_tile(new_x, new_y) == TileType::Floor {
                    let mut pos = self.world.get::<&mut Position>(id).unwrap();
                    occupied_positions.remove(&(pos.x, pos.y));
                    pos.x = new_x;
                    pos.y = new_y;
                    occupied_positions.insert((new_x, new_y));
                }
            } else {
                let (monster_name, monster_power) = {
                    let stats = self.world.get::<&CombatStats>(id).unwrap();
                    let name = self.world.get::<&Name>(id).unwrap();
                    (name.0.clone(), stats.power)
                };

                let player_defense = self.world.get::<&CombatStats>(player_id).unwrap().defense;
                let damage = (monster_power - player_defense).max(0);
                
                let mut player_stats = self.world.get::<&mut CombatStats>(player_id).unwrap();
                player_stats.hp -= damage;
                self.log.push(format!("{} hits you for {} damage!", monster_name, damage));

                if player_stats.hp <= 0 {
                    self.log.push("You are dead!".to_string());
                    self.state = RunState::Dead;
                    self.death = true;
                }
            }
        }
        if self.state != RunState::Dead {
            self.state = RunState::AwaitingInput;
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(6),
            ])
            .split(frame.size());

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(30),
            ])
            .split(chunks[0]);

        let map_area = top_chunks[0];
        let sidebar_area = top_chunks[1];
        let log_area = chunks[1];

        // Map
        let map_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Indexed(240)))
            .title(Span::styled(" RustLike Dungeon ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        frame.render_widget(map_block, map_area);

        let inner_map = map_area.inner(&Margin { vertical: 1, horizontal: 1 });
        let buffer = frame.buffer_mut();

        let mut player_query = self.world.query::<(&Position, &Player, &CombatStats)>();
        let (_, (player_pos, _, player_stats)) = player_query.iter().next().expect("Player not found");

        let view_w = inner_map.width as i32;
        let view_h = inner_map.height as i32;
        let mut camera_x = player_pos.x as i32 - view_w / 2;
        let mut camera_y = player_pos.y as i32 - view_h / 2;
        camera_x = camera_x.clamp(0, (self.map.width as i32 - view_w).max(0));
        camera_y = camera_y.clamp(0, (self.map.height as i32 - view_h).max(0));

        for y in 0..view_h {
            let map_y = y + camera_y;
            if map_y >= self.map.height as i32 { break; }
            for x in 0..view_w {
                let map_x = x + camera_x;
                if map_x >= self.map.width as i32 { break; }
                let idx = (map_y as u16 * self.map.width + map_x as u16) as usize;
                if !self.map.revealed[idx] { continue; }
                let (char, color) = match self.map.tiles[idx] {
                    TileType::Wall => ("#", if self.map.visible[idx] { Color::Indexed(252) } else { Color::Indexed(238) }),
                    TileType::Floor => (".", if self.map.visible[idx] { Color::Indexed(242) } else { Color::Indexed(234) }),
                };
                buffer.get_mut(inner_map.x + x as u16, inner_map.y + y as u16).set_symbol(char).set_fg(color);
            }
        }

        // Render entities
        for (id, (pos, render)) in self.world.query::<(&Position, &Renderable)>().iter() {
            let idx = (pos.y * self.map.width + pos.x) as usize;
            if !self.map.visible[idx] { continue; }
            let screen_x = pos.x as i32 - camera_x;
            let screen_y = pos.y as i32 - camera_y;
            if screen_x >= 0 && screen_x < view_w && screen_y >= 0 && screen_y < view_h {
                let x = inner_map.x + screen_x as u16;
                let y = inner_map.y + screen_y as u16;
                let mut style = Style::default().fg(render.fg);
                if self.world.get::<&Player>(id).is_ok() { style = style.add_modifier(Modifier::BOLD); }
                buffer.get_mut(x, y).set_symbol(&render.glyph.to_string()).set_style(style);
            }
        }

        // Sidebar
        let sidebar = Block::default().borders(Borders::ALL).title(" Character ");
        let hp_percent = (player_stats.hp as f32 / player_stats.max_hp as f32 * 100.0) as u16;
        let hp_color = if hp_percent > 50 { Color::Green } else if hp_percent > 25 { Color::Yellow } else { Color::Red };
        
        let sidebar_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // HP bar
                Constraint::Length(1), // ATK/DEF
                Constraint::Length(1), // Level
                Constraint::Min(0),
            ])
            .split(sidebar_area.inner(&Margin { vertical: 1, horizontal: 1 }));

        let hp_gauge = Gauge::default()
            .block(Block::default().title("HP"))
            .gauge_style(Style::default().fg(hp_color).bg(Color::Indexed(233)))
            .percent(hp_percent)
            .label(format!("{}/{}", player_stats.hp, player_stats.max_hp));
        frame.render_widget(hp_gauge, sidebar_layout[0]);

        frame.render_widget(Paragraph::new(format!("ATK: {}  DEF: {}", player_stats.power, player_stats.defense)), sidebar_layout[1]);
        frame.render_widget(Paragraph::new(format!("Depth: Level {}", self.dungeon_level)), sidebar_layout[2]);
        frame.render_widget(sidebar, sidebar_area);

        // Log
        let log_block = Block::default().borders(Borders::ALL).title(" Message Log ");
        let log_items: Vec<ListItem> = self.log.iter().rev().take(5).map(|s| ListItem::new(s.clone())).collect();
        frame.render_widget(List::new(log_items).block(log_block), log_area);

        // Overlays
        if self.state == RunState::ShowInventory {
            self.render_inventory(frame);
        } else if self.state == RunState::ShowHelp {
            self.render_help(frame);
        } else if self.state == RunState::Dead {
            self.render_death_screen(frame);
        }
    }

    fn render_inventory(&self, frame: &mut Frame) {
        let area = centered_rect(60, 60, frame.size());
        frame.render_widget(Clear, area);
        let block = Block::default().borders(Borders::ALL).title(" Inventory (Enter to Use, Esc to Exit) ");
        
        let items: Vec<ListItem> = self.world.query::<(&Item, &InBackpack, &Name)>()
            .iter()
            .map(|(_, (_, _, name))| ListItem::new(name.0.clone()))
            .collect();

        if items.is_empty() {
            frame.render_widget(Paragraph::new("Your backpack is empty.").block(block), area);
        } else {
            let mut state = ListState::default();
            state.select(Some(self.inventory_cursor));
            frame.render_stateful_widget(
                List::new(items)
                    .block(block)
                    .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
                    .highlight_symbol(">> "),
                area,
                &mut state
            );
        }
    }

    fn render_help(&self, frame: &mut Frame) {
        let area = centered_rect(50, 50, frame.size());
        frame.render_widget(Clear, area);
        let block = Block::default().borders(Borders::ALL).title(" Controls ");
        let text = vec![
            Line::from(vec![Span::styled("Move:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" Arrow Keys or HJKL")]),
            Line::from(vec![Span::styled("Pick Up Item:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" G")]),
            Line::from(vec![Span::styled("Inventory:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" I")]),
            Line::from(vec![Span::styled("Help:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" ? or /")]),
            Line::from(vec![Span::styled("Quit:", Style::default().add_modifier(Modifier::BOLD)), Span::raw(" Q")]),
            Line::from(""),
            Line::from("Bump into monsters to attack them."),
        ];
        frame.render_widget(Paragraph::new(text).block(block).alignment(Alignment::Center), area);
    }

    fn render_death_screen(&self, frame: &mut Frame) {
        let area = centered_rect(40, 20, frame.size());
        frame.render_widget(Clear, area);
        let block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::Red));
        let text = vec![
            Line::from(Span::styled("YOU HAVE PERISHED", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("Press Q or Esc to exit."),
        ];
        frame.render_widget(Paragraph::new(text).block(block).alignment(Alignment::Center), area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: RatatuiRect) -> RatatuiRect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_get_tile() {
        let mut map = Map::new(10, 10);
        map.tiles[0] = TileType::Wall;
        map.tiles[1] = TileType::Floor;
        assert_eq!(map.get_tile(0, 0), TileType::Wall);
        assert_eq!(map.get_tile(1, 0), TileType::Floor);
        assert_eq!(map.get_tile(10, 10), TileType::Wall);
    }

    #[test]
    fn test_app_move_player() {
        let mut app = App::new_random();
        let (x, y) = {
            let mut player_query = app.world.query::<(&Position, &Player, &CombatStats)>();
            let (_, (pos, _, _)) = player_query.iter().next().unwrap();
            (pos.x, pos.y)
        };
        
        let mut dx = 0;
        let mut dy = 0;
        if app.map.get_tile(x + 1, y) == TileType::Floor { dx = 1; }
        else if app.map.get_tile(x - 1, y) == TileType::Floor { dx = -1; }
        else if app.map.get_tile(x, y + 1) == TileType::Floor { dy = 1; }
        else if app.map.get_tile(x, y - 1) == TileType::Floor { dy = -1; }

        if dx != 0 || dy != 0 {
            let target_x = x + dx as u16;
            let target_y = y + dy as u16;
            app.move_player(dx, dy);
            let mut player_query = app.world.query::<(&Position, &Player, &CombatStats)>();
            let (_, (pos, _, _)) = player_query.iter().next().unwrap();
            assert_eq!(pos.x, target_x);
            assert_eq!(pos.y, target_y);
        }
    }

    #[test]
    fn test_fov() {
        let mut app = App::new_random();
        let (x, y) = {
            let mut player_query = app.world.query::<(&Position, &Player, &CombatStats)>();
            let (_, (pos, _, _)) = player_query.iter().next().unwrap();
            (pos.x, pos.y)
        };
        let idx = (y * app.map.width + x) as usize;
        assert!(app.map.visible[idx]);
        assert!(app.map.revealed[idx]);
    }
}
