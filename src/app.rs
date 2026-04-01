use ratatui::prelude::*;
use ratatui::widgets::*;
use serde::{Deserialize, Serialize};
use crate::map_builder::MapBuilder;
use crate::components::*;
use hecs::World;

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
}

impl Map {
    pub fn get_tile(&self, x: u16, y: u16) -> TileType {
        if x >= self.width || y >= self.height {
            return TileType::Wall;
        }
        self.tiles[(y * self.width + x) as usize]
    }
}

/// A snapshot of an entity for serialization
#[derive(Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub pos: Position,
    pub render: Renderable,
    pub name: Option<Name>,
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
    // Used only for persistence
    pub entities: Vec<EntitySnapshot>,
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
        ));

        Self {
            exit: false,
            death: false,
            world,
            map: mb.map,
            entities: Vec::new(),
        }
    }

    /// Prepares the entities vector for serialization
    pub fn pack_entities(&mut self) {
        self.entities.clear();
        for (id, (pos, render)) in self.world.query::<(&Position, &Renderable)>().iter() {
            let is_player = self.world.get::<&Player>(id).is_ok();
            let is_monster = self.world.get::<&Monster>(id).is_ok();
            let name = self.world.get::<&Name>(id).ok().map(|n| (*n).clone());
            
            self.entities.push(EntitySnapshot {
                pos: *pos,
                render: *render,
                name,
                is_player,
                is_monster,
            });
        }
    }

    /// Rebuilds the world from the entities vector after deserialization
    pub fn unpack_entities(&mut self) {
        self.world = World::new();
        for e in &self.entities {
            let mut cb = hecs::EntityBuilder::new();
            cb.add(e.pos);
            cb.add(e.render);
            if let Some(ref name) = e.name {
                cb.add(name.clone());
            }
            if e.is_player { cb.add(Player); }
            if e.is_monster { cb.add(Monster); }
            self.world.spawn(cb.build());
        }
    }

    pub fn move_player(&mut self, dx: i16, dy: i16) {
        let mut player_query = self.world.query::<(&mut Position, &Player)>();
        let (_, (pos, _)) = player_query.iter().next().expect("Player not found");

        let new_x = (pos.x as i16 + dx).max(0) as u16;
        let new_y = (pos.y as i16 + dy).max(0) as u16;

        if self.map.get_tile(new_x, new_y) == TileType::Floor {
            pos.x = new_x;
            pos.y = new_y;
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(frame.size());

        let map_area = chunks[0];
        let status_area = chunks[1];

        let map_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" RustLike Dungeon ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        frame.render_widget(map_block, map_area);

        let inner_map = map_area.inner(&Margin { vertical: 1, horizontal: 1 });
        let buffer = frame.buffer_mut();

        // Find player for camera
        let mut player_query = self.world.query::<(&Position, &Player)>();
        let (_, (player_pos, _)) = player_query.iter().next().expect("Player not found");

        let view_w = inner_map.width as i32;
        let view_h = inner_map.height as i32;
        
        let mut camera_x = player_pos.x as i32 - view_w / 2;
        let mut camera_y = player_pos.y as i32 - view_h / 2;

        camera_x = camera_x.clamp(0, (self.map.width as i32 - view_w).max(0));
        camera_y = camera_y.clamp(0, (self.map.height as i32 - view_h).max(0));

        // Render map
        for y in 0..view_h {
            let map_y = y + camera_y;
            if map_y >= self.map.height as i32 { break; }
            for x in 0..view_w {
                let map_x = x + camera_x;
                if map_x >= self.map.width as i32 { break; }
                
                let (char, color) = match self.map.get_tile(map_x as u16, map_y as u16) {
                    TileType::Wall => ("#", Color::Indexed(242)),
                    TileType::Floor => (".", Color::Indexed(237)),
                };

                buffer.get_mut(inner_map.x + x as u16, inner_map.y + y as u16)
                    .set_symbol(char)
                    .set_fg(color);
            }
        }

        // Render entities from ECS
        for (_id, (pos, render)) in self.world.query::<(&Position, &Renderable)>().iter() {
            let screen_x = pos.x as i32 - camera_x;
            let screen_y = pos.y as i32 - camera_y;

            if screen_x >= 0 && screen_x < view_w && screen_y >= 0 && screen_y < view_h {
                let x = inner_map.x + screen_x as u16;
                let y = inner_map.y + screen_y as u16;
                let mut style = Style::default().fg(render.fg);
                if self.world.get::<&Player>(_id).is_ok() {
                    style = style.add_modifier(Modifier::BOLD);
                }
                buffer.get_mut(x, y)
                    .set_symbol(&render.glyph.to_string())
                    .set_style(style);
            }
        }

        let status = Paragraph::new(format!(
            " HP: 10/10 | Pos: ({}, {}) | Camera: ({}, {}) | Press 'q' to quit",
            player_pos.x, player_pos.y, camera_x, camera_y
        ))
        .style(Style::default().bg(Color::Indexed(235)).fg(Color::White));
        frame.render_widget(status, status_area);
    }
}
