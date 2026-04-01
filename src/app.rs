use ratatui::prelude::*;
use ratatui::widgets::*;
use serde::{Deserialize, Serialize};

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
    pub fn new(width: u16, height: u16) -> Self {
        let mut tiles = vec![TileType::Floor; (width * height) as usize];
        
        // Add some walls around the edges
        for x in 0..width {
            tiles[x as usize] = TileType::Wall;
            tiles[((height - 1) * width + x) as usize] = TileType::Wall;
        }
        for y in 0..height {
            tiles[(y * width) as usize] = TileType::Wall;
            tiles[(y * width + (width - 1)) as usize] = TileType::Wall;
        }

        // Add some random walls for flavor
        tiles[(15 * width + 15) as usize] = TileType::Wall;
        tiles[(15 * width + 16) as usize] = TileType::Wall;
        tiles[(16 * width + 15) as usize] = TileType::Wall;

        Self { width, height, tiles }
    }

    pub fn get_tile(&self, x: u16, y: u16) -> TileType {
        if x >= self.width || y >= self.height {
            return TileType::Wall;
        }
        self.tiles[(y * self.width + x) as usize]
    }
}

#[derive(Serialize, Deserialize)]
pub struct App {
    #[serde(skip)]
    pub exit: bool,
    #[serde(skip)]
    pub death: bool,
    pub player_pos: (u16, u16),
    pub map: Map,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            death: false,
            player_pos: (5, 5),
            map: Map::new(40, 20),
        }
    }

    pub fn move_player(&mut self, dx: i16, dy: i16) {
        let new_x = (self.player_pos.0 as i16 + dx).max(0) as u16;
        let new_y = (self.player_pos.1 as i16 + dy).max(0) as u16;

        if self.map.get_tile(new_x, new_y) == TileType::Floor {
            self.player_pos = (new_x, new_y);
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

        // Draw map block
        let map_block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(" RustLike Dungeon ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
        frame.render_widget(map_block, map_area);

        let inner_map = map_area.inner(&Margin { vertical: 1, horizontal: 1 });
        let buffer = frame.buffer_mut();

        // Render the map tiles efficiently by setting buffer cells directly
        for y in 0..self.map.height {
            if y >= inner_map.height { break; }
            for x in 0..self.map.width {
                if x >= inner_map.width { break; }
                
                let (char, color) = match self.map.get_tile(x, y) {
                    TileType::Wall => ("#", Color::Indexed(242)), // Grayish
                    TileType::Floor => (".", Color::Indexed(237)), // Dark Gray
                };

                let pos_x = inner_map.x + x;
                let pos_y = inner_map.y + y;
                buffer.get_mut(pos_x, pos_y).set_symbol(char).set_fg(color);
            }
        }

        // Render player character '@'
        if self.player_pos.0 < inner_map.width && self.player_pos.1 < inner_map.height {
            let x = inner_map.x + self.player_pos.0;
            let y = inner_map.y + self.player_pos.1;
            buffer.get_mut(x, y)
                .set_symbol("@")
                .set_fg(Color::Yellow)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        // Draw status bar
        let status = Paragraph::new(format!(
            " HP: 10/10 | Pos: ({}, {}) | Press 'q' to quit",
            self.player_pos.0, self.player_pos.1
        ))
        .style(Style::default().bg(Color::Indexed(235)).fg(Color::White));
        frame.render_widget(status, status_area);
    }
}
