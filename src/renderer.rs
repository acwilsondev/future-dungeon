use crate::app::{App, RunState, VisualEffect};
use crate::components::*;
use crate::map::TileType;
use bracket_pathfinding::prelude::*;
use ratatui::layout::Rect as RatatuiRect;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn apply_lighting(color: Color, intensity: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * intensity).clamp(0.0, 255.0) as u8,
            (g as f32 * intensity).clamp(0.0, 255.0) as u8,
            (b as f32 * intensity).clamp(0.0, 255.0) as u8,
        ),
        Color::Indexed(i) => {
            if intensity < 0.2 {
                Color::Indexed(232)
            } else if intensity < 0.4 {
                Color::Indexed(236)
            } else if intensity < 0.6 {
                Color::Indexed(240)
            } else if intensity < 0.8 {
                Color::Indexed(244)
            } else if intensity < 1.0 {
                Color::Indexed(248)
            } else {
                Color::Indexed(i)
            }
        }
        _ => color,
    }
}

fn get_tile_render_data(app: &App, idx: usize, is_visible: bool) -> (&'static str, Color) {
    match app.current_branch {
        Branch::Main => match app.map.tiles[idx] {
            TileType::Wall => (
                "#",
                if is_visible {
                    Color::Indexed(252)
                } else {
                    Color::Indexed(238)
                },
            ),
            TileType::Floor => (
                ".",
                if is_visible {
                    Color::Indexed(242)
                } else {
                    Color::Indexed(234)
                },
            ),
        },
        Branch::Gardens => match app.map.tiles[idx] {
            TileType::Wall => (
                "♣",
                if is_visible {
                    Color::Rgb(34, 139, 34)
                } else {
                    Color::Rgb(0, 100, 0)
                },
            ),
            TileType::Floor => (
                ",",
                if is_visible {
                    Color::Rgb(144, 238, 144)
                } else {
                    Color::Rgb(50, 150, 50)
                },
            ),
        },
        Branch::Vaults => match app.map.tiles[idx] {
            TileType::Wall => (
                "#",
                if is_visible {
                    Color::Rgb(173, 216, 230)
                } else {
                    Color::Rgb(70, 130, 180)
                },
            ),
            TileType::Floor => (
                "-",
                if is_visible {
                    Color::Rgb(224, 255, 255)
                } else {
                    Color::Rgb(100, 149, 237)
                },
            ),
        },
    }
}

fn draw_tiles(app: &App, buffer: &mut ratatui::buffer::Buffer, area: RatatuiRect, camera: (i32, i32)) {
    let (camera_x, camera_y) = camera;
    let view_w = area.width as i32;
    let view_h = area.height as i32;

    for y in 0..view_h {
        let map_y = y + camera_y;
        if map_y < 0 || map_y >= app.map.height as i32 {
            continue;
        }
        for x in 0..view_w {
            let map_x = x + camera_x;
            if map_x < 0 || map_x >= app.map.width as i32 {
                continue;
            }
            let idx = map_y as usize * app.map.width as usize + map_x as usize;
            if !app.map.revealed[idx] {
                continue;
            }

            let light = app.map.light[idx];
            let is_visible = app.map.visible[idx];

            let (char, mut color) = get_tile_render_data(app, idx, is_visible);

            if is_visible {
                color = apply_lighting(color, light.max(0.2));
            } else {
                color = apply_lighting(color, 0.1);
            }

            buffer
                .get_mut(area.x + x as u16, area.y + y as u16)
                .set_symbol(char)
                .set_fg(color);
        }
    }
}

fn draw_entities(app: &App, buffer: &mut ratatui::buffer::Buffer, area: RatatuiRect, camera: (i32, i32)) {
    let (camera_x, camera_y) = camera;
    let view_w = area.width as i32;
    let view_h = area.height as i32;

    let mut data: Vec<(hecs::Entity, Position, Renderable, RenderOrder)> = Vec::new();
    for (id, (pos, render, order)) in app
        .world
        .query::<(&Position, &Renderable, &RenderOrder)>()
        .iter()
    {
        data.push((id, *pos, *render, *order));
    }
    data.sort_by(|a, b| a.3.cmp(&b.3));

    for (id, pos, render, _) in data {
        let idx = pos.y as usize * app.map.width as usize + pos.x as usize;
        if !app.map.visible[idx] {
            continue;
        }

        let light = app.map.light[idx];
        if light < 0.1 && app.world.get::<&Player>(id).is_err() {
            continue;
        }

        if let Ok(trap) = app.world.get::<&Trap>(id) {
            if !trap.revealed {
                continue;
            }
        }
        let screen_x = pos.x as i32 - camera_x;
        let screen_y = pos.y as i32 - camera_y;
        if screen_x >= 0 && screen_x < view_w && screen_y >= 0 && screen_y < view_h {
            let color = apply_lighting(render.fg, light.max(0.3));
            let mut style = Style::default().fg(color);
            if app.world.get::<&Player>(id).is_ok() {
                style = style.add_modifier(Modifier::BOLD);
            }
            buffer
                .get_mut(area.x + screen_x as u16, area.y + screen_y as u16)
                .set_symbol(&render.glyph.to_string())
                .set_style(style);
        }
    }
}

fn draw_effects(app: &App, buffer: &mut ratatui::buffer::Buffer, area: RatatuiRect, camera: (i32, i32)) {
    let (camera_x, camera_y) = camera;
    let view_w = area.width as i32;
    let view_h = area.height as i32;

    for effect in &app.effects {
        match effect {
            VisualEffect::Flash {
                x,
                y,
                glyph,
                fg,
                bg,
                ..
            } => {
                let idx = *y as usize * app.map.width as usize + *x as usize;
                if !app.map.visible[idx] {
                    continue;
                }
                let sx = *x as i32 - camera_x;
                let sy = *y as i32 - camera_y;
                if sx >= 0 && sx < view_w && sy >= 0 && sy < view_h {
                    let cell = buffer.get_mut(area.x + sx as u16, area.y + sy as u16);
                    cell.set_symbol(&glyph.to_string()).set_fg(*fg);
                    if let Some(bg_color) = bg {
                        cell.set_bg(*bg_color);
                    }
                }
            }
            VisualEffect::Projectile {
                path,
                glyph,
                fg,
                frame,
                speed,
            } => {
                let path_idx = (*frame / *speed) as usize;
                if let Some(pos) = path.get(path_idx) {
                    let idx = pos.1 as usize * app.map.width as usize + pos.0 as usize;
                    if !app.map.visible[idx] {
                        return;
                    }
                    let sx = pos.0 as i32 - camera_x;
                    let sy = pos.1 as i32 - camera_y;
                    if sx >= 0 && sx < view_w && sy >= 0 && sy < view_h {
                        buffer
                            .get_mut(area.x + sx as u16, area.y + sy as u16)
                            .set_symbol(&glyph.to_string())
                            .set_fg(*fg);
                    }
                }
            }
        }
    }
}

fn draw_targeting_line(app: &App, buffer: &mut ratatui::buffer::Buffer, area: RatatuiRect, camera: (i32, i32), player_pos: &Position) {
    let (camera_x, camera_y) = camera;
    let view_w = area.width as i32;
    let view_h = area.height as i32;

    let line = line2d(
        LineAlg::Bresenham,
        Point::new(player_pos.x, player_pos.y),
        Point::new(app.targeting_cursor.0, app.targeting_cursor.1),
    );

    for (i, p) in line.iter().enumerate() {
        let sx = p.x - camera_x;
        let sy = p.y - camera_y;
        if sx >= 0 && sx < view_w && sy >= 0 && sy < view_h {
            let cell = buffer.get_mut(area.x + sx as u16, area.y + sy as u16);
            if i > 0 {
                if i == line.len() - 1 {
                    cell.set_bg(Color::Cyan).set_fg(Color::Black);
                } else {
                    cell.set_bg(Color::Indexed(236));
                }
            }
        }
    }
}

fn draw_map(
    app: &App,
    frame: &mut Frame,
    area: RatatuiRect,
    camera: (i32, i32),
    player_pos: &Position,
) {
    let buffer = frame.buffer_mut();
    draw_tiles(app, buffer, area, camera);
    draw_entities(app, buffer, area, camera);
    draw_effects(app, buffer, area, camera);

    if app.state == RunState::ShowTargeting {
        draw_targeting_line(app, buffer, area, camera, player_pos);
    }
}

fn draw_sidebar(
    app: &App,
    frame: &mut Frame,
    area: RatatuiRect,
    player_pos: &Position,
    player_stats: &CombatStats,
) {
    let sidebar_block = Block::default().borders(Borders::ALL).title(" Character ");
    frame.render_widget(sidebar_block, area);

    let sidebar_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // HP
            Constraint::Length(1), // Stats
            Constraint::Length(7), // Attributes
            Constraint::Length(1), // XP
            Constraint::Length(3), // Noise
            Constraint::Min(0),    // Status/Perks
        ])
        .split(area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        }));

    let hp_percent = (player_stats.hp as f32 / player_stats.max_hp as f32 * 100.0) as u16;
    let hp_color = if hp_percent > 50 {
        Color::Green
    } else if hp_percent > 25 {
        Color::Yellow
    } else {
        Color::Red
    };
    frame.render_widget(
        Gauge::default()
            .block(Block::default().title("HP"))
            .gauge_style(Style::default().fg(hp_color).bg(Color::Indexed(233)))
            .percent(hp_percent)
            .label(format!("{}/{}", player_stats.hp, player_stats.max_hp)),
        sidebar_layout[0],
    );
    let (player_power, player_av, dodge_dc) = app.get_player_stats();

    frame.render_widget(
        Paragraph::new(format!(
            "POW: {} AV: {} DC: {}",
            player_power, player_av, dodge_dc
        )),
        sidebar_layout[1],
    );

    let Some(player_id) = app.get_player_id() else { return; };

    let attr_text = if let Ok(attr) = app.world.get::<&Attributes>(player_id) {
        format!(
            "STR: {:2} ({:2})\nDEX: {:2} ({:2})\nCON: {:2} ({:2})\nINT: {:2} ({:2})\nWIS: {:2} ({:2})\nCHA: {:2} ({:2})",
            attr.strength, Attributes::get_modifier(attr.strength),
            attr.dexterity, Attributes::get_modifier(attr.dexterity),
            attr.constitution, Attributes::get_modifier(attr.constitution),
            attr.intelligence, Attributes::get_modifier(attr.intelligence),
            attr.wisdom, Attributes::get_modifier(attr.wisdom),
            attr.charisma, Attributes::get_modifier(attr.charisma),
        )
    } else {
        "Attributes missing".to_string()
    };
    frame.render_widget(
        Paragraph::new(attr_text).block(Block::default().title("Attributes")),
        sidebar_layout[2],
    );
    let (level, xp, next_xp) = if let Ok(exp) = app.world.get::<&Experience>(player_id) {
        (exp.level, exp.xp, exp.next_level_xp)
    } else {
        (1, 0, 50)
    };

    frame.render_widget(
        Paragraph::new(format!("Level: {}  XP: {}/{}", level, xp, next_xp)),
        sidebar_layout[3],
    );

    let player_idx = (player_pos.y * app.map.width + player_pos.x) as usize;
    let player_noise = app.map.sound[player_idx];
    let noise_percent = (player_noise * 10.0).clamp(0.0, 100.0) as u16;
    let noise_color = if noise_percent < 30 {
        Color::Cyan
    } else if noise_percent < 70 {
        Color::Yellow
    } else {
        Color::Red
    };
    let noise_label = if noise_percent < 30 {
        "Quiet"
    } else if noise_percent < 70 {
        "Noisy"
    } else {
        "LOUD!"
    };

    frame.render_widget(
        Gauge::default()
            .block(Block::default().title("Noise"))
            .gauge_style(Style::default().fg(noise_color).bg(Color::Indexed(233)))
            .percent(noise_percent)
            .label(noise_label),
        sidebar_layout[4],
    );

    let mut status_lines = Vec::new();
    let gold_amount = app
        .world
        .get::<&Gold>(player_id)
        .map(|g| g.amount)
        .unwrap_or(0);
    status_lines.push(Line::from(Span::styled(
        format!("Gold: {}", gold_amount),
        Style::default().fg(Color::Yellow),
    )));

    if let Ok(poison) = app.world.get::<&Poison>(player_id) {
        status_lines.push(Line::from(Span::styled(
            format!("Poisoned ({})", poison.turns),
            Style::default().fg(Color::Green),
        )));
    }
    if let Ok(strength) = app.world.get::<&Strength>(player_id) {
        status_lines.push(Line::from(Span::styled(
            format!("Strong ({})", strength.turns),
            Style::default().fg(Color::Yellow),
        )));
    }
    if let Ok(speed) = app.world.get::<&Speed>(player_id) {
        status_lines.push(Line::from(Span::styled(
            format!("Fast ({})", speed.turns),
            Style::default().fg(Color::Cyan),
        )));
    }
    if let Ok(confusion) = app.world.get::<&Confusion>(player_id) {
        status_lines.push(Line::from(Span::styled(
            format!("Confused ({})", confusion.turns),
            Style::default().fg(Color::Magenta),
        )));
    }

    if let Ok(perks) = app.world.get::<&Perks>(player_id) {
        for perk in &perks.traits {
            let name = match perk {
                Perk::Toughness => "Toughness",
                Perk::EagleEye => "Eagle Eye",
                Perk::Strong => "Strong",
                Perk::ThickSkin => "Thick Skin",
            };
            status_lines.push(Line::from(Span::styled(
                format!("* {}", name),
                Style::default().fg(Color::LightBlue),
            )));
        }
    }

    if !status_lines.is_empty() {
        frame.render_widget(
            Paragraph::new(status_lines).block(Block::default().title(" Status/Perks ")),
            sidebar_layout[4],
        );
    }
}

fn draw_message_log(app: &App, frame: &mut Frame, area: RatatuiRect) {
    let log_block = Block::default()
        .borders(Borders::ALL)
        .title(" Message Log ");
    let log_items: Vec<ListItem> = app
        .log
        .iter()
        .rev()
        .take(5)
        .enumerate()
        .map(|(i, s)| {
            let mut style = Style::default();
            if i == 0 {
                style = style.add_modifier(Modifier::BOLD).fg(Color::White);
            } else {
                style = style.fg(Color::Indexed(245));
            }

            let mut fg = style.fg.unwrap_or(Color::White);
            if s.contains("damage") || s.contains("dies") || s.contains("dead") {
                fg = Color::Red;
            } else if s.contains("gold") || s.contains("buy") {
                fg = Color::Yellow;
            } else if s.contains("level") {
                fg = Color::Magenta;
            } else if s.contains("health") || s.contains("heal") {
                fg = Color::Green;
            }

            ListItem::new(Span::styled(
                s.clone(),
                Style::default().fg(fg).add_modifier(if i == 0 {
                    Modifier::BOLD
                } else {
                    Modifier::empty()
                }),
            ))
        })
        .collect();
    frame.render_widget(List::new(log_items).block(log_block), area);
}

pub fn render(app: &App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(6)])
        .split(frame.size());
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(30)])
        .split(chunks[0]);
    let map_area = top_chunks[0];
    let sidebar_area = top_chunks[1];
    let log_area = chunks[1];

    let map_title = format!(" RustLike Dungeon - FPS: {:.1} ", app.fps);
    let map_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Indexed(240)))
        .title(Span::styled(
            map_title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    frame.render_widget(map_block, map_area);
    let inner_map = map_area.inner(&Margin {
        vertical: 1,
        horizontal: 1,
    });

    let mut player_query = app.world.query::<(&Position, &Player, &CombatStats)>();
    let Some((_, (player_pos, _, player_stats))) = player_query.iter().next() else { return; };

    let view_w = inner_map.width as i32;
    let view_h = inner_map.height as i32;
    let mut camera_x = player_pos.x as i32 - view_w / 2;
    let mut camera_y = player_pos.y as i32 - view_h / 2;
    camera_x = camera_x.clamp(0, (app.map.width as i32 - view_w).max(0));
    camera_y = camera_y.clamp(0, (app.map.height as i32 - view_h).max(0));

    // 1. Draw Map & Entities
    draw_map(app, frame, inner_map, (camera_x, camera_y), player_pos);

    // 2. Draw Sidebar (Character Stats)
    draw_sidebar(app, frame, sidebar_area, player_pos, player_stats);

    // 3. Draw Message Log
    draw_message_log(app, frame, log_area);

    if app.state == RunState::ShowInventory
        || app.state == RunState::ShowIdentify
        || app.state == RunState::ShowAlchemy
    {
        render_inventory(app, frame);
    } else if app.state == RunState::ShowHelp {
        render_help(app, frame);
    } else if app.state == RunState::Dead {
        render_death_screen(app, frame);
    } else if app.state == RunState::ShowClassSelection {
        render_class_selection(app, frame);
    } else if app.state == RunState::LevelUp {
        render_level_up(app, frame);
    } else if app.state == RunState::ShowShop {
        render_shop(app, frame);
    } else if app.state == RunState::ShowLogHistory {
        render_log_history(app, frame);
    } else if app.state == RunState::ShowBestiary {
        render_bestiary(app, frame);
    } else if app.state == RunState::Victory {
        render_victory_screen(app, frame);
    }
}

fn render_log_history(app: &App, frame: &mut Frame) {
    let area = centered_rect(80, 80, frame.size());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Message History ");

    let log_items: Vec<ListItem> = app
        .log
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let mut fg = Color::Indexed(245);
            if s.contains("damage") || s.contains("dies") || s.contains("dead") {
                fg = Color::Red;
            } else if s.contains("gold") || s.contains("buy") {
                fg = Color::Yellow;
            } else if s.contains("level") {
                fg = Color::Magenta;
            } else if s.contains("health") || s.contains("heal") {
                fg = Color::Green;
            }

            ListItem::new(Span::styled(
                format!("{}: {}", i + 1, s),
                Style::default().fg(fg),
            ))
        })
        .collect();

    let mut state = ListState::default();
    let scroll_pos = if app.log.len() > area.height as usize - 2 {
        app.log_cursor
    } else {
        0
    };
    state.select(Some(scroll_pos));

    frame.render_stateful_widget(
        List::new(log_items)
            .block(block.title_bottom(
                Line::from(" [UP/DOWN] Scroll, [ESC] Close ").alignment(Alignment::Right),
            ))
            .highlight_style(Style::default().bg(Color::Indexed(236)))
            .highlight_symbol("> "),
        area,
        &mut state,
    );
}

fn render_bestiary(app: &App, frame: &mut Frame) {
    let area = centered_rect(80, 80, frame.size());
    frame.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).title(" Bestiary ");

    let mut encountered: Vec<String> = app.encountered_monsters.iter().cloned().collect();
    encountered.sort();

    if encountered.is_empty() {
        frame.render_widget(
            Paragraph::new("You haven't encountered any monsters yet.").block(block),
            area,
        );
        return;
    }

    let list_items: Vec<ListItem> = encountered
        .iter()
        .map(|name| ListItem::new(name.clone()))
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.bestiary_cursor));

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        }));

    frame.render_stateful_widget(
        List::new(list_items)
            .block(Block::default().borders(Borders::RIGHT))
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)),
        layout[0],
        &mut state,
    );

    // Details side
    if let Some(selected_name) = encountered.get(app.bestiary_cursor) {
        let details = match selected_name.as_str() {
            "Orc" => "A common dungeon dweller. Fierce and aggressive. They tend to charge directly at you.",
            "Goblin" => "Small, weak, and cowardly. They often flee when their health is low.",
            "Goblin Archer" => "Keeps their distance and fires arrows. Try to corner them!",
            "Spider" => "Fast and dangerous. Their bites can be painful.",
            _ => "A mysterious inhabitant of the deep."
        };
        frame.render_widget(
            Paragraph::new(details)
                .wrap(Wrap { trim: true })
                .block(Block::default().title(format!(" {} ", selected_name))),
            layout[1],
        );
    }

    frame.render_widget(block, area);
}

fn render_shop(app: &App, frame: &mut Frame) {
    let area = centered_rect(70, 70, frame.size());
    frame.render_widget(Clear, area);

    let Some(player_id) = app.get_player_id() else { return; };
    let player_gold = app
        .world
        .get::<&Gold>(player_id)
        .map(|g| g.amount)
        .unwrap_or(0);

    let title = if app.shop_mode == 0 {
        format!(" Merchant Shop (Buy) - Your Gold: {} ", player_gold)
    } else {
        format!(" Merchant Shop (Sell) - Your Gold: {} ", player_gold)
    };
    let block = Block::default().borders(Borders::ALL).title(title);

    let items: Vec<(hecs::Entity, String, i32)> = if app.shop_mode == 0 {
        // Buy: Merchant's backpack
        if let Some(merchant_id) = app.active_merchant {
            app.world
                .query::<(&Item, &InBackpack, &Name, &ItemValue)>()
                .iter()
                .filter(|(_, (_, backpack, _, _))| backpack.owner == merchant_id)
                .map(|(id, (_, _, name, value))| (id, name.0.clone(), value.price))
                .collect()
        } else {
            Vec::new()
        }
    } else {
        // Sell: Player's backpack (Filter out equipped items)
        app.world
            .query::<(&Item, &InBackpack, &Name, &ItemValue)>()
            .iter()
            .filter(|(id, (_, backpack, _, _))| {
                backpack.owner == player_id && app.world.get::<&Equipped>(*id).is_err()
            })
            .map(|(id, (_, _, name, value))| (id, name.0.clone(), value.price / 2))
            .collect()
    };

    if items.is_empty() {
        frame.render_widget(
            Paragraph::new("Nothing here. (TAB to switch mode)").block(block),
            area,
        );
    } else {
        let list_items: Vec<ListItem> = items
            .iter()
            .map(|(_, name, price)| ListItem::new(format!("{}: {}g", name, price)))
            .collect();

        let mut state = ListState::default();
        state.select(Some(app.shop_cursor));

        let footer = " [UP/DOWN] Select, [ENTER] Confirm, [TAB] Buy/Sell, [ESC] Leave ";
        frame.render_stateful_widget(
            List::new(list_items)
                .block(block.title_bottom(Line::from(footer).alignment(Alignment::Right)))
                .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
                .highlight_symbol(">> "),
            area,
            &mut state,
        );
    }
}

fn render_level_up(app: &App, frame: &mut Frame) {
    let area = centered_rect(50, 50, frame.size());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Level Up! Increase an Attribute ");

    let options = vec![
        ListItem::new("Strength (+1 STR)"),
        ListItem::new("Dexterity (+1 DEX)"),
        ListItem::new("Constitution (+1 CON)"),
        ListItem::new("Intelligence (+1 INT)"),
        ListItem::new("Wisdom (+1 WIS)"),
        ListItem::new("Charisma (+1 CHA)"),
    ];

    let mut state = ListState::default();
    state.select(Some(app.level_up_cursor));
    frame.render_stateful_widget(
        List::new(options)
            .block(block)
            .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black))
            .highlight_symbol(">> "),
        area,
        &mut state,
    );
}

fn render_inventory(app: &App, frame: &mut Frame) {
    let area = centered_rect(70, 70, frame.size());
    frame.render_widget(Clear, area);
    let title = if app.state == RunState::ShowIdentify {
        " Identify Item "
    } else if app.state == RunState::ShowAlchemy {
        " Alchemy Station "
    } else {
        " Inventory "
    };
    let block = Block::default().borders(Borders::ALL).title(title);

    let Some(player_id) = app.get_player_id() else { return; };
    let items: Vec<(hecs::Entity, String)> = app
        .world
        .query::<(&Item, &InBackpack)>()
        .iter()
        .filter(|(_, (_, backpack))| backpack.owner == player_id)
        .map(|(id, _)| (id, app.get_item_name(id)))
        .collect();

    if items.is_empty() {
        frame.render_widget(Paragraph::new("Your backpack is empty.").block(block), area);
    } else {
        let list_items: Vec<ListItem> = items
            .iter()
            .map(|(id, name)| {
                let mut display_name = name.clone();
                if app.state == RunState::ShowAlchemy && app.alchemy_selection.contains(id) {
                    display_name = format!("{} (Selected)", display_name);
                }
                if app.world.get::<&Equipped>(*id).is_ok() {
                    display_name = format!("{} (E)", display_name);
                }
                ListItem::new(display_name)
            })
            .collect();
        let mut state = ListState::default();
        state.select(Some(app.inventory_cursor));

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(35),
                Constraint::Percentage(35),
            ])
            .split(area.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }));

        frame.render_stateful_widget(
            List::new(list_items)
                .block(Block::default().borders(Borders::RIGHT))
                .highlight_style(Style::default().bg(Color::Cyan).fg(Color::Black)),
            layout[0],
            &mut state,
        );

        // Paper Doll / Equipment Slots
        let mut equipment_list = Vec::new();
        let slots = [
            (EquipmentSlot::Head, "Head"),
            (EquipmentSlot::Neck, "Neck"),
            (EquipmentSlot::Torso, "Torso"),
            (EquipmentSlot::Hands, "Hands"),
            (EquipmentSlot::Feet, "Feet"),
            (EquipmentSlot::MainHand, "Main Hand"),
            (EquipmentSlot::OffHand, "Off Hand"),
            (EquipmentSlot::Ammo, "Ammo"),
            (EquipmentSlot::LeftFinger, "L.Finger"),
            (EquipmentSlot::RightFinger, "R.Finger"),
        ];

        for (slot, label) in slots {
            let mut equipped_name = "None".to_string();
            let mut equipped_color = Color::DarkGray;

            for (id, (eq, backpack)) in app.world.query::<(&Equipped, &InBackpack)>().iter() {
                if backpack.owner == player_id && eq.slot == slot {
                    equipped_name = app.get_item_name(id);
                    equipped_color = Color::Cyan;
                    break;
                }
            }

            equipment_list.push(Line::from(vec![
                Span::styled(
                    format!("{:<10}: ", label),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(equipped_name, Style::default().fg(equipped_color)),
            ]));
        }
        frame.render_widget(
            Paragraph::new(equipment_list).block(
                Block::default()
                    .title(" Equipment ")
                    .borders(Borders::RIGHT),
            ),
            layout[1],
        );

        // Item Details / Tooltip
        if let Some((item_id, _)) = items.get(app.inventory_cursor) {
            let mut tooltip = Vec::new();

            let real_name = app
                .world
                .get::<&Name>(*item_id)
                .map(|n| n.0.clone())
                .unwrap_or_default();
            let is_identified = app.identified_items.contains(&real_name)
                || app.world.get::<&ObfuscatedName>(*item_id).is_err();

            if !is_identified {
                tooltip.push(Line::from(vec![Span::styled(
                    "Unknown properties.",
                    Style::default().fg(Color::DarkGray),
                )]));
                tooltip.push(Line::from("Use or identify to reveal."));
            } else {
                if let Ok(potion) = app.world.get::<&Potion>(*item_id) {
                    tooltip.push(Line::from(vec![
                        Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Potion"),
                    ]));
                    tooltip.push(Line::from(vec![
                        Span::styled("Effect: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!("Heals {} HP", potion.heal_amount),
                            Style::default().fg(Color::Green),
                        ),
                    ]));
                }
                if let Ok(weapon) = app.world.get::<&Weapon>(*item_id) {
                    tooltip.push(Line::from(vec![
                        Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{:?} Melee Weapon{}", weapon.weight, if weapon.two_handed { " (2H)" } else { "" })),
                    ]));
                    tooltip.push(Line::from(vec![
                        Span::styled("Damage: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!("{}d{}", weapon.damage_n_dice, weapon.damage_die_type),
                            Style::default().fg(Color::Red),
                        ),
                        Span::raw(format!(" + {} power", weapon.power_bonus)),
                    ]));
                }
                if let Ok(armor) = app.world.get::<&Armor>(*item_id) {
                    tooltip.push(Line::from(vec![
                        Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Armor"),
                    ]));
                    tooltip.push(Line::from(vec![
                        Span::styled("Bonus: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!("+{} Defense", armor.defense_bonus),
                            Style::default().fg(Color::Blue),
                        ),
                    ]));
                    if let Some(max_dex) = armor.max_dex_bonus {
                        tooltip.push(Line::from(vec![
                            Span::styled("Max DEX Bonus: ", Style::default().add_modifier(Modifier::BOLD)),
                            Span::raw(format!("{}", max_dex)),
                        ]));
                    }
                }
                if let Ok(ranged) = app.world.get::<&Ranged>(*item_id) {
                    tooltip.push(Line::from(vec![
                        Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Consumable Ranged"),
                    ]));
                    tooltip.push(Line::from(vec![
                        Span::styled("Range: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{}", ranged.range)),
                    ]));
                }
                if let Ok(rw) = app.world.get::<&RangedWeapon>(*item_id) {
                    tooltip.push(Line::from(vec![
                        Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Ranged Weapon"),
                    ]));
                    tooltip.push(Line::from(vec![
                        Span::styled("Range: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(format!("{}", rw.range)),
                    ]));
                    tooltip.push(Line::from(vec![
                        Span::styled("Bonus: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!("+{} Damage", rw.damage_bonus),
                            Style::default().fg(Color::Red),
                        ),
                    ]));
                }
                if app.world.get::<&Ammunition>(*item_id).is_ok() {
                    tooltip.push(Line::from(vec![
                        Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw("Ammunition"),
                    ]));
                    tooltip.push(Line::from("Required for bows."));
                }
                if let Ok(aoe) = app.world.get::<&AreaOfEffect>(*item_id) {
                    tooltip.push(Line::from(vec![
                        Span::styled(
                            "AoE Radius: ",
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{}", aoe.radius),
                            Style::default().fg(Color::Yellow),
                        ),
                    ]));
                }
                if let Ok(poison) = app.world.get::<&Poison>(*item_id) {
                    tooltip.push(Line::from(vec![
                        Span::styled("Poison: ", Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(
                            format!("{} damage for {} turns", poison.damage, poison.turns),
                            Style::default().fg(Color::Green),
                        ),
                    ]));
                }
            }

            if app.world.get::<&Cursed>(*item_id).is_ok() {
                tooltip.push(Line::from(vec![Span::styled(
                    "CURSED",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]));
            }

            frame.render_widget(
                Paragraph::new(tooltip).block(Block::default().title(" Item Details ")),
                layout[2],
            );
        }

        frame.render_widget(block, area);
    }
}

fn render_help(_app: &App, frame: &mut Frame) {
    let area = centered_rect(50, 60, frame.size());
    frame.render_widget(Clear, area);
    let text = vec![
        Line::from(vec![
            Span::styled("Move:", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Arrows/HJKL"),
        ]),
        Line::from(vec![
            Span::styled("Pick Up:", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" G"),
        ]),
        Line::from(vec![
            Span::styled("Inventory:", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" I"),
        ]),
        Line::from(vec![
            Span::styled(
                "Log History:",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(" M"),
        ]),
        Line::from(vec![
            Span::styled("Bestiary:", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" B"),
        ]),
        Line::from(vec![
            Span::styled("Targeting:", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Arrows/HJKL + Enter"),
        ]),
        Line::from(vec![
            Span::styled("Help:", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" ? or /"),
        ]),
        Line::from(vec![
            Span::styled("Quit:", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Q"),
        ]),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(" Controls "))
            .alignment(Alignment::Center),
        area,
    );
}

fn render_death_screen(app: &App, frame: &mut Frame) {
    let area = centered_rect(50, 40, frame.size());
    frame.render_widget(Clear, area);

    let (gold, level) = if let Some(player_id) = app.get_player_id() {
        let gold = app
            .world
            .get::<&Gold>(player_id)
            .map(|g| g.amount)
            .unwrap_or(0);
        let level = app
            .world
            .get::<&Experience>(player_id)
            .map(|e| e.level)
            .unwrap_or(1);
        (gold, level)
    } else {
        (0, 1)
    };

    let text = vec![
        Line::from(Span::styled(
            "YOU HAVE PERISHED",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Final Level: {}", level)),
        Line::from(format!("Monsters Slain: {}", app.monsters_killed)),
        Line::from(format!("Gold Collected: {}", gold)),
        Line::from(""),
        Line::from("Your journey ends here in the dark."),
        Line::from(""),
        Line::from(Span::styled(
            "Press Q or Esc to exit.",
            Style::default().fg(Color::Indexed(245)),
        )),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Red)),
            )
            .alignment(Alignment::Center),
        area,
    );
}

fn render_victory_screen(app: &App, frame: &mut Frame) {
    let area = centered_rect(50, 45, frame.size());
    frame.render_widget(Clear, area);

    let (gold, level) = if let Some(player_id) = app.get_player_id() {
        let gold = app
            .world
            .get::<&Gold>(player_id)
            .map(|g| g.amount)
            .unwrap_or(0);
        let level = app
            .world
            .get::<&Experience>(player_id)
            .map(|e| e.level)
            .unwrap_or(1);
        (gold, level)
    } else {
        (0, 1)
    };

    let text = vec![
        Line::from(Span::styled(
            "VICTORY!",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("You have emerged from the dungeon with the"),
        Line::from(Span::styled(
            "Amulet of the Ancients!",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "--- SESSION SUMMARY ---",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(format!("Final Level: {}", level)),
        Line::from(format!("Monsters Slain: {}", app.monsters_killed)),
        Line::from(format!("Gold Collected: {}", gold)),
        Line::from(""),
        Line::from("The echoes of the ancients will sing of your name."),
        Line::from(""),
        Line::from(Span::styled(
            "Press Q or Esc to exit.",
            Style::default().fg(Color::Indexed(245)),
        )),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .alignment(Alignment::Center),
        area,
    );
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

fn render_class_selection(app: &App, frame: &mut Frame) {
    let area = centered_rect(50, 40, frame.size());
    frame.render_widget(Clear, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Choose Your Class ");

    let classes = vec!["Fighter"];

    let list_items: Vec<ListItem> = classes
        .iter()
        .enumerate()
        .map(|(i, class_name)| {
            let mut style = Style::default();
            if i == app.class_selection {
                style = style.fg(Color::Yellow).add_modifier(Modifier::BOLD);
            }
            ListItem::new(Span::styled(format!("  {}  ", class_name), style))
        })
        .collect();

    let list = List::new(list_items).block(block);
    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Color;
    use ratatui::layout::Rect as RatatuiRect;

    #[test]
    fn test_apply_lighting() {
        let white = Color::Rgb(255, 255, 255);
        let dark_white = apply_lighting(white, 0.5);
        assert_eq!(dark_white, Color::Rgb(127, 127, 127));

        let indexed = Color::Indexed(10);
        let dark_indexed = apply_lighting(indexed, 0.1);
        assert_eq!(dark_indexed, Color::Indexed(232));
    }

    #[test]
    fn test_centered_rect() {
        let r = RatatuiRect::new(0, 0, 100, 100);
        let centered = centered_rect(50, 50, r);
        assert_eq!(centered.x, 25);
        assert_eq!(centered.y, 25);
        assert_eq!(centered.width, 50);
        assert_eq!(centered.height, 50);
    }

    #[test]
    fn test_render_basic() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut app = App::new_random();
        app.map = crate::map::Map::new(80, 50);
        app.world.spawn((Position { x: 10, y: 10 }, Player, Viewshed { visible_tiles: 8 }));

        let backend = TestBackend::new(80, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                render(&mut app, f);
            })
            .unwrap();
    }

    #[test]
    fn test_render_states() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let mut app = App::new_random();
        app.map = crate::map::Map::new(80, 50);
        app.world.spawn((Position { x: 10, y: 10 }, Player));

        let backend = TestBackend::new(80, 50);
        let mut terminal = Terminal::new(backend).unwrap();

        let states = [
            RunState::ShowInventory,
            RunState::ShowHelp,
            RunState::ShowTargeting,
            RunState::LevelUp,
            RunState::ShowShop,
            RunState::ShowLogHistory,
            RunState::ShowBestiary,
            RunState::ShowIdentify,
            RunState::ShowAlchemy,
            RunState::Dead,
            RunState::Victory,
        ];

        for state in states {
            app.state = state;
            terminal
                .draw(|f| {
                    render(&mut app, f);
                })
                .unwrap();
        }
    }
}
