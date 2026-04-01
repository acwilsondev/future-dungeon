mod app;
mod engine;
mod persistence;
mod map_builder;

use anyhow::Result;
use app::App;
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
                    match key.code {
                        KeyCode::Char('q') => app.exit = true,
                        KeyCode::Left | KeyCode::Char('h') => app.move_player(-1, 0),
                        KeyCode::Right | KeyCode::Char('l') => app.move_player(1, 0),
                        KeyCode::Up | KeyCode::Char('k') => app.move_player(0, -1),
                        KeyCode::Down | KeyCode::Char('j') => app.move_player(0, 1),
                        _ => {}
                    }
                }
            }
        }

        if app.death {
            persistence::delete_save();
            break;
        }
    }

    // Save game on exit if the player is still alive
    if !app.death && app.exit {
        persistence::save_game(&app)?;
    }

    Ok(())
}
