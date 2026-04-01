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
    let mut app = persistence::load_game()?.unwrap_or_else(App::new);

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
                        RunState::ShowHelp => {
                            if let KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('/') = key.code {
                                app.state = RunState::AwaitingInput;
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
