mod actions;
mod app;
mod components;
mod content;
mod engine;
mod input;
mod map;
mod map_builder;
mod persistence;
mod renderer;
mod spawner;

use anyhow::Result;
use app::{App, RunState};
use crossterm::event::{self, Event, KeyEventKind};
use engine::Engine;
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    let mut engine = Engine::new()?;

    // Try to load an existing game, otherwise start a new one
    let mut app = match persistence::load_game() {
        Ok(Some(app)) => app,
        _ => App::new(),
    };

    let mut last_frame = Instant::now();

    while !app.exit {
        let now = Instant::now();
        let delta = now.duration_since(last_frame).as_secs_f32();
        last_frame = now;

        // Smooth FPS calculation
        if delta > 0.0 {
            let current_fps = 1.0 / delta;
            app.fps = app.fps * 0.9 + current_fps * 0.1;
        }

        app.on_tick();
        engine.draw(|f| crate::renderer::render(&app, f))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(action) = input::map_key_to_action(key, app.state) {
                        app.process_action(action);
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
