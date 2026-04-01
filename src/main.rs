mod app;
mod engine;
mod persistence;
mod map_builder;
mod components;

use anyhow::Result;
use app::{App, RunState};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use engine::Engine;
use std::time::Duration;

fn main() -> Result<()> {
    let mut engine = Engine::new()?;
    
    // Try to load an existing game, otherwise start a new one
    let mut app = match persistence::load_game() {
        Ok(Some(app)) => app,
        _ => App::new(),
    };

    while !app.exit {
        engine.draw(|f| app.render(f))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.state {
                        RunState::AwaitingInput => {
                            match key.code {
                                KeyCode::Char('q') => app.exit = true,
                                KeyCode::Left | KeyCode::Char('h') => app.move_player(-1, 0),
                                KeyCode::Right | KeyCode::Char('l') => app.move_player(1, 0),
                                KeyCode::Up | KeyCode::Char('k') => app.move_player(0, -1),
                                KeyCode::Down | KeyCode::Char('j') => app.move_player(0, 1),
                                KeyCode::Char('g') => app.pick_up_item(),
                                KeyCode::Char('i') => app.state = RunState::ShowInventory,
                                KeyCode::Char('?') | KeyCode::Char('/') => app.state = RunState::ShowHelp,
                                KeyCode::Enter => app.try_level_transition(),
                                _ => {}
                            }
                        }
                        RunState::ShowInventory => {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char('i') => app.state = RunState::AwaitingInput,
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if app.inventory_cursor > 0 { app.inventory_cursor -= 1; }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    let count = app.world.query::<(&crate::components::InBackpack,)>().iter().count();
                                    if count > 0 && app.inventory_cursor < count - 1 {
                                        app.inventory_cursor += 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    let item_to_use = app.world.query::<(&crate::components::Item, &crate::components::InBackpack)>()
                                        .iter()
                                        .nth(app.inventory_cursor)
                                        .map(|(id, _)| id);
                                    
                                    if let Some(id) = item_to_use {
                                        app.use_item(id);
                                        app.inventory_cursor = 0;
                                    }
                                }
                                _ => {}
                            }
                        }
                        RunState::ShowTargeting => {
                            match key.code {
                                KeyCode::Esc => app.state = RunState::AwaitingInput,
                                KeyCode::Left | KeyCode::Char('h') => { if app.targeting_cursor.0 > 0 { app.targeting_cursor.0 -= 1; } }
                                KeyCode::Right | KeyCode::Char('l') => { if app.targeting_cursor.0 < app.map.width - 1 { app.targeting_cursor.0 += 1; } }
                                KeyCode::Up | KeyCode::Char('k') => { if app.targeting_cursor.1 > 0 { app.targeting_cursor.1 -= 1; } }
                                KeyCode::Down | KeyCode::Char('j') => { if app.targeting_cursor.1 < app.map.height - 1 { app.targeting_cursor.1 += 1; } }
                                KeyCode::Enter => app.fire_targeting_item(),
                                _ => {}
                            }
                        }
                        RunState::ShowHelp => {
                            if let KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('/') = key.code {
                                app.state = RunState::AwaitingInput;
                            }
                        }
                        RunState::LevelUp => {
                            match key.code {
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if app.level_up_cursor > 0 { app.level_up_cursor -= 1; }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    if app.level_up_cursor < 3 { app.level_up_cursor += 1; }
                                }
                                KeyCode::Enter => {
                                    let player_id = app.world.query::<(&crate::components::Player,)>().iter().next().unwrap().0;
                                    
                                    match app.level_up_cursor {
                                        0 => {
                                            if let Ok(mut stats) = app.world.get::<&mut crate::components::CombatStats>(player_id) {
                                                stats.max_hp += 10;
                                                stats.hp += 10;
                                            }
                                            if let Ok(mut perks) = app.world.get::<&mut crate::components::Perks>(player_id) {
                                                perks.traits.push(crate::components::Perk::Toughness);
                                            }
                                            app.log.push("You chose Toughness! Max HP increased.".to_string());
                                        }
                                        1 => {
                                            if let Ok(mut viewshed) = app.world.get::<&mut crate::components::Viewshed>(player_id) {
                                                viewshed.visible_tiles += 2;
                                            }
                                            if let Ok(mut perks) = app.world.get::<&mut crate::components::Perks>(player_id) {
                                                perks.traits.push(crate::components::Perk::EagleEye);
                                            }
                                            app.log.push("You chose Eagle Eye! FOV increased.".to_string());
                                        }
                                        2 => {
                                            if let Ok(mut stats) = app.world.get::<&mut crate::components::CombatStats>(player_id) {
                                                stats.power += 2;
                                            }
                                            if let Ok(mut perks) = app.world.get::<&mut crate::components::Perks>(player_id) {
                                                perks.traits.push(crate::components::Perk::Strong);
                                            }
                                            app.log.push("You chose Strong! Power increased.".to_string());
                                        }
                                        3 => {
                                            if let Ok(mut stats) = app.world.get::<&mut crate::components::CombatStats>(player_id) {
                                                stats.defense += 1;
                                            }
                                            if let Ok(mut perks) = app.world.get::<&mut crate::components::Perks>(player_id) {
                                                perks.traits.push(crate::components::Perk::ThickSkin);
                                            }
                                            app.log.push("You chose Thick Skin! Defense increased.".to_string());
                                        }
                                        _ => {}
                                    }
                                    
                                    app.state = RunState::MonsterTurn;
                                }
                                _ => {}
                            }
                        }
                        RunState::ShowShop => {
                            match key.code {
                                KeyCode::Esc => app.state = RunState::AwaitingInput,
                                KeyCode::Tab => {
                                    app.shop_mode = (app.shop_mode + 1) % 2;
                                    app.shop_cursor = 0;
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    if app.shop_cursor > 0 { app.shop_cursor -= 1; }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    let player_id = app.world.query::<(&crate::components::Player,)>().iter().next().unwrap().0;
                                    let count = if app.shop_mode == 0 {
                                        if let Some(m_id) = app.active_merchant {
                                            app.world.query::<(&crate::components::InBackpack,)>().iter()
                                                .filter(|(_, (backpack,))| backpack.owner == m_id)
                                                .count()
                                        } else { 0 }
                                    } else {
                                        app.world.query::<(&crate::components::InBackpack,)>().iter()
                                            .filter(|(_, (backpack,))| backpack.owner == player_id)
                                            .count()
                                    };
                                    if count > 0 && app.shop_cursor < count - 1 {
                                        app.shop_cursor += 1;
                                    }
                                }
                                KeyCode::Enter => {
                                    let player_id = app.world.query::<(&crate::components::Player,)>().iter().next().unwrap().0;
                                    let item_to_trade = if app.shop_mode == 0 {
                                        if let Some(m_id) = app.active_merchant {
                                            app.world.query::<(&crate::components::InBackpack,)>().iter()
                                                .filter(|(_, (backpack,))| backpack.owner == m_id)
                                                .nth(app.shop_cursor)
                                                .map(|(id, _)| id)
                                        } else { None }
                                    } else {
                                        app.world.query::<(&crate::components::InBackpack,)>().iter()
                                            .filter(|(_, (backpack,))| backpack.owner == player_id)
                                            .nth(app.shop_cursor)
                                            .map(|(id, _)| id)
                                    };
                                    
                                    if let Some(id) = item_to_trade {
                                        if app.shop_mode == 0 {
                                            app.buy_item(id);
                                        } else {
                                            app.sell_item(id);
                                        }
                                        app.shop_cursor = 0;
                                    }
                                }
                                _ => {}
                            }
                        }
                        RunState::Dead => {
                            if let KeyCode::Char('q') | KeyCode::Esc = key.code {
                                app.exit = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.state == RunState::MonsterTurn {
            app.monster_turn();
        }

        if app.death && app.state != RunState::Dead {
            app.state = RunState::Dead;
        }
    }

    // Save game on exit if the player is still alive
    if !app.death && app.exit {
        persistence::save_game(app)?;
    } else if app.death {
        persistence::delete_save();
    }

    Ok(())
}
