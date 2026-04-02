use crossterm::event::{KeyCode, KeyEvent};
use crate::actions::Action;
use crate::app::RunState;

pub fn map_key_to_action(key: KeyEvent, state: RunState) -> Option<Action> {
    match state {
        RunState::AwaitingInput => {
            match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Left | KeyCode::Char('h') => Some(Action::MovePlayer(-1, 0)),
                KeyCode::Right | KeyCode::Char('l') => Some(Action::MovePlayer(1, 0)),
                KeyCode::Up | KeyCode::Char('k') => Some(Action::MovePlayer(0, -1)),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::MovePlayer(0, 1)),
                KeyCode::Char('g') => Some(Action::PickUpItem),
                KeyCode::Char('i') => Some(Action::OpenInventory),
                KeyCode::Char('?') | KeyCode::Char('/') => Some(Action::OpenHelp),
                KeyCode::Char('m') => Some(Action::OpenLogHistory),
                KeyCode::Char('b') => Some(Action::OpenBestiary),
                KeyCode::Enter => Some(Action::TryLevelTransition),
                KeyCode::Char(' ') | KeyCode::Char('.') => Some(Action::Wait),
                _ => None,
            }
        }
        RunState::ShowLogHistory => {
            match key.code {
                KeyCode::Esc | KeyCode::Char('m') => Some(Action::CloseMenu),
                KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
                _ => None,
            }
        }
        RunState::ShowBestiary => {
            match key.code {
                KeyCode::Esc | KeyCode::Char('b') => Some(Action::CloseMenu),
                KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
                _ => None,
            }
        }
        RunState::ShowInventory => {
            match key.code {
                KeyCode::Esc | KeyCode::Char('i') => Some(Action::CloseMenu),
                KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
                KeyCode::Enter => Some(Action::MenuSelect),
                _ => None,
            }
        }
        RunState::ShowTargeting => {
            match key.code {
                KeyCode::Esc => Some(Action::CloseMenu),
                KeyCode::Left | KeyCode::Char('h') => Some(Action::MovePlayer(-1, 0)),
                KeyCode::Right | KeyCode::Char('l') => Some(Action::MovePlayer(1, 0)),
                KeyCode::Up | KeyCode::Char('k') => Some(Action::MovePlayer(0, -1)),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::MovePlayer(0, 1)),
                KeyCode::Enter => Some(Action::MenuSelect),
                _ => None,
            }
        }
        RunState::ShowHelp => {
            match key.code {
                KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('/') => Some(Action::CloseMenu),
                _ => None,
            }
        }
        RunState::LevelUp => {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
                KeyCode::Enter => Some(Action::MenuSelect),
                _ => None,
            }
        }
        RunState::ShowShop => {
            match key.code {
                KeyCode::Esc => Some(Action::CloseMenu),
                KeyCode::Tab => Some(Action::ToggleShopMode),
                KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
                KeyCode::Enter => Some(Action::MenuSelect),
                _ => None,
            }
        }
        RunState::Dead => {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
                _ => None,
            }
        }
        _ => None,
    }
}
