use ratatui::prelude::*;
use ratatui::widgets::*;
use serde::{Deserialize, Serialize};
use crate::map_builder::MapBuilder;
use crate::components::*;
use hecs::World;
use bracket_pathfinding::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
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
    pub pos: Position,
    pub render: Renderable,
    pub name: Option<Name>,
    pub stats: Option<CombatStats>,
    pub is_player: bool,
    pub is_monster: bool,
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
}

impl App {
    pub fn new() -> Self {
        Self::new_random()
    }

    pub fn new_random() -> Self {
        let mut mb = MapBuilder::new(80, 50);
        mb.build();
        let mut world = World::new();
        
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

        let mut app = Self {
            exit: false,
            death: false,
            world,
            map: mb.map,
            entities: Vec::new(),
            log: vec!["Welcome to RustLike!".to_string()],
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
        for (id, (pos, render)) in self.world.query::<(&Position, &Renderable)>().iter() {
            let is_player = self.world.get::<&Player>(id).is_ok();
            let is_monster = self.world.get::<&Monster>(id).is_ok();
            let name = self.world.get::<&Name>(id).ok().map(|n| (*n).clone());
            let stats = self.world.get::<&CombatStats>(id).ok().map(|s| *s);
            
            self.entities.push(EntitySnapshot {
                pos: *pos,
                render: *render,
                name,
                stats,
                is_player,
                is_monster,
            });
        }
    }

    pub fn unpack_entities(&mut self) {
        self.world = World::new();
        for e in &self.entities {
            let mut cb = hecs::EntityBuilder::new();
            cb.add(e.pos);
            cb.add(e.render);
            if let Some(ref name) = e.name {
                cb.add(name.clone());
            }
            if let Some(stats) = e.stats {
                cb.add(stats);
            }
            if e.is_player { cb.add(Player); }
            if e.is_monster { cb.add(Monster); }
            self.world.spawn(cb.build());
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

        // Check for monster at target position (Bump Combat)
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
                if monster_stats.hp <= 0 {
                    dead = true;
                }
            }
            if dead {
                self.log.push(format!("{} dies!", monster_name));
                self.world.despawn(monster_id).unwrap();
            }
            return;
        }

        if self.map.get_tile(new_x, new_y) == TileType::Floor {
            let mut player_query = self.world.query::<(&mut Position, &Player)>();
            let (_, (pos, _)) = player_query.iter().next().expect("Player not found");
            pos.x = new_x;
            pos.y = new_y;
            drop(player_query);
            self.update_fov();
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

        // Collect all current monster positions to avoid nested world queries
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
                    // Update the set and the component
                    let mut pos = self.world.get::<&mut Position>(id).unwrap();
                    occupied_positions.remove(&(pos.x, pos.y));
                    pos.x = new_x;
                    pos.y = new_y;
                    occupied_positions.insert((new_x, new_y));
                }
            } else {
                // Attack Player
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
                    self.death = true;
                }
            }
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(5),
            ])
            .split(frame.size());

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(25),
            ])
            .split(chunks[0]);

        let map_area = top_chunks[0];
        let sidebar_area = top_chunks[1];
        let log_area = chunks[1];

        // Map
        let map_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" Map ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
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
        let sidebar = Block::default().borders(Borders::ALL).title(" Status ");
        let stats_text = vec![
            Line::from(vec![Span::raw("HP: "), Span::styled(format!("{}/{}", player_stats.hp, player_stats.max_hp), Style::default().fg(Color::Red))]),
            Line::from(format!("ATK: {}", player_stats.power)),
            Line::from(format!("DEF: {}", player_stats.defense)),
        ];
        frame.render_widget(Paragraph::new(stats_text).block(sidebar), sidebar_area);

        // Log
        let log_block = Block::default().borders(Borders::ALL).title(" Log ");
        let log_items: Vec<ListItem> = self.log.iter().rev().take(4).map(|s| ListItem::new(s.clone())).collect();
        frame.render_widget(List::new(log_items).block(log_block), log_area);
    }
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
