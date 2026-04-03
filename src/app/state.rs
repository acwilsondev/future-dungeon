use serde::{Deserialize, Serialize};
use ratatui::prelude::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunState {
    AwaitingInput,
    MonsterTurn,
    ShowInventory,
    ShowHelp,
    ShowTargeting,
    LevelUp,
    ShowShop,
    ShowLogHistory,
    ShowBestiary,
    ShowIdentify,
    ShowAlchemy,
    Dead,
    Victory,
}

pub enum MonsterAction {
    Move(i16, i16),
    Attack(hecs::Entity),
    RangedAttack(hecs::Entity),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VisualEffect {
    Flash { x: u16, y: u16, glyph: char, fg: Color, bg: Option<Color>, duration: u32 },
    Projectile { path: Vec<(u16, u16)>, glyph: char, fg: Color, frame: u32, speed: u32 },
}

pub fn default_runstate() -> RunState { RunState::AwaitingInput }
