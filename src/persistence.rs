use crate::app::App;
use anyhow::Result;
use std::fs;
use std::path::Path;

const SAVE_FILE: &str = "savegame.json";

pub fn save_game(app: &App) -> Result<()> {
    let json = serde_json::to_string(app)?;
    fs::write(SAVE_FILE, json)?;
    Ok(())
}

pub fn load_game() -> Result<Option<App>> {
    let path = Path::new(SAVE_FILE);
    if !path.exists() {
        return Ok(None);
    }

    let json = fs::read_to_string(path)?;
    let app: App = serde_json::from_str(&json)?;
    
    // Enforce permadeath: delete the save file as soon as it's loaded
    delete_save();
    
    Ok(Some(app))
}

pub fn delete_save() {
    let path = Path::new(SAVE_FILE);
    if path.exists() {
        let _ = fs::remove_file(path);
    }
}
