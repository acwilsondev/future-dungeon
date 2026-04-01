mod app;
mod engine;

use anyhow::Result;
use app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use engine::Engine;
use std::time::Duration;

fn main() -> Result<()> {
    let mut engine = Engine::new()?;
    let mut app = App::new();

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
    }

    Ok(())
}
