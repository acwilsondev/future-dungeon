use crate::app::App;
use anyhow::Result;
use std::fs;
use std::path::Path;

const SAVE_FILE: &str = "savegame.json";

pub fn save_game(mut app: App) -> Result<()> {
    app.pack_entities();
    let json = serde_json::to_string(&app)?;
    fs::write(SAVE_FILE, json)?;
    Ok(())
}

pub fn load_game() -> Result<Option<App>> {
    let path = Path::new(SAVE_FILE);
    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(path)?;
    let mut app: App = serde_json::from_str(&json)?;
    app.unpack_entities()?;

    delete_save();

    Ok(Some(app))
}

pub fn delete_save() {
    let path = Path::new(SAVE_FILE);
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;

    #[test]
    fn test_save_load_game() {
        let mut app = App::new_random();
        app.world = hecs::World::new();
        let player = app.world.spawn((
            Player,
            Position { x: 10, y: 10 },
            CombatStats { hp: 20, max_hp: 20, defense: 0, power: 5 },
            Renderable { glyph: '@', fg: ratatui::prelude::Color::Yellow },
            RenderOrder::Player,
        ));
        app.world.spawn((
            Item,
            Name("Sword".to_string()),
            InBackpack { owner: player },
            Renderable { glyph: '/', fg: ratatui::prelude::Color::White },
            RenderOrder::Item,
        ));

        save_game(app).unwrap();
        assert!(Path::new(SAVE_FILE).exists());

        let loaded_app = load_game().unwrap().expect("Failed to load game");
        assert_eq!(loaded_app.world.query::<&Player>().iter().count(), 1);
        assert_eq!(loaded_app.world.query::<&Item>().iter().count(), 1);
        assert!(!Path::new(SAVE_FILE).exists()); // load_game deletes save
    }
}
