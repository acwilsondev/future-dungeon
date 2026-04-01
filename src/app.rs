use ratatui::prelude::*;
use ratatui::widgets::*;
use serde::{Deserialize, Serialize};
use crate::map_builder::MapBuilder;

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
        Self::new_random()
    }

    pub fn new_random() -> Self {
        let mut mb = MapBuilder::new(80, 50);
        mb.build();
        Self {
            exit: false,
            death: false,
            player_pos: mb.player_start,
            map: mb.map,
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

        // Camera logic: center the view on the player
        let view_w = inner_map.width as i32;
        let view_h = inner_map.height as i32;
        
        let mut camera_x = self.player_pos.0 as i32 - view_w / 2;
        let mut camera_y = self.player_pos.1 as i32 - view_h / 2;

        // Clamp camera to map boundaries
        camera_x = camera_x.clamp(0, (self.map.width as i32 - view_w).max(0));
        camera_y = camera_y.clamp(0, (self.map.height as i32 - view_h).max(0));

        // Render the map tiles relative to the camera
        for y in 0..view_h {
            let map_y = y + camera_y;
            if map_y >= self.map.height as i32 { break; }
            
            for x in 0..view_w {
                let map_x = x + camera_x;
                if map_x >= self.map.width as i32 { break; }
                
                let (char, color) = match self.map.get_tile(map_x as u16, map_y as u16) {
                    TileType::Wall => ("#", Color::Indexed(242)), // Grayish
                    TileType::Floor => (".", Color::Indexed(237)), // Dark Gray
                };

                let pos_x = inner_map.x + x as u16;
                let pos_y = inner_map.y + y as u16;
                buffer.get_mut(pos_x, pos_y).set_symbol(char).set_fg(color);
            }
        }

        // Render player character '@' relative to the camera
        let player_screen_x = self.player_pos.0 as i32 - camera_x;
        let player_screen_y = self.player_pos.1 as i32 - camera_y;

        if player_screen_x >= 0 && player_screen_x < view_w && player_screen_y >= 0 && player_screen_y < view_h {
            let x = inner_map.x + player_screen_x as u16;
            let y = inner_map.y + player_screen_y as u16;
            buffer.get_mut(x, y)
                .set_symbol("@")
                .set_fg(Color::Yellow)
                .set_style(Style::default().add_modifier(Modifier::BOLD));
        }

        // Draw status bar
        let status = Paragraph::new(format!(
            " HP: 10/10 | Pos: ({}, {}) | Camera: ({}, {}) | Press 'q' to quit",
            self.player_pos.0, self.player_pos.1, camera_x, camera_y
        ))
        .style(Style::default().bg(Color::Indexed(235)).fg(Color::White));
        frame.render_widget(status, status_area);
    }
}
