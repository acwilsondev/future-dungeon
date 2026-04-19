use crate::actions::Action;
use crate::app::RunState;
use crossterm::event::{KeyCode, KeyEvent};

pub fn map_key_to_action(key: KeyEvent, state: RunState) -> Option<Action> {
    match state {
        RunState::ShowClassSelection => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::MenuSelect),
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
            _ => None,
        },
        RunState::AwaitingInput => match key.code {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Left | KeyCode::Char('h') => Some(Action::MovePlayer(-1, 0)),
            KeyCode::Right | KeyCode::Char('l') => Some(Action::MovePlayer(1, 0)),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MovePlayer(0, -1)),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MovePlayer(0, 1)),
            KeyCode::Char('g') => Some(Action::PickUpItem),
            KeyCode::Char('i') => Some(Action::OpenInventory),
            KeyCode::Char('a') => Some(Action::OpenSpells),
            KeyCode::Char('?') | KeyCode::Char('/') => Some(Action::OpenHelp),
            KeyCode::Char('m') => Some(Action::OpenLogHistory),
            KeyCode::Char('b') => Some(Action::OpenBestiary),
            KeyCode::Enter => Some(Action::TryLevelTransition),
            KeyCode::Char('f') => Some(Action::Target),
            KeyCode::Char('`') | KeyCode::Char('~') => Some(Action::ToggleDebugConsole),
            KeyCode::Char(' ') | KeyCode::Char('.') => Some(Action::Wait),
            _ => None,
        },
        RunState::ShowLogHistory => match key.code {
            KeyCode::Esc | KeyCode::Char('m') => Some(Action::CloseMenu),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            _ => None,
        },
        RunState::ShowBestiary => match key.code {
            KeyCode::Esc | KeyCode::Char('b') => Some(Action::CloseMenu),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            _ => None,
        },
        RunState::ShowInventory => match key.code {
            KeyCode::Esc | KeyCode::Char('i') => Some(Action::CloseMenu),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::MenuSelect),
            _ => None,
        },
        RunState::ShowTargeting => match key.code {
            KeyCode::Esc => Some(Action::CloseMenu),
            KeyCode::Left | KeyCode::Char('h') => Some(Action::MovePlayer(-1, 0)),
            KeyCode::Right | KeyCode::Char('l') => Some(Action::MovePlayer(1, 0)),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MovePlayer(0, -1)),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MovePlayer(0, 1)),
            KeyCode::Enter => Some(Action::MenuSelect),
            _ => None,
        },
        RunState::MainMenu => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::MenuSelect),
            KeyCode::Esc | KeyCode::Char('q') => Some(Action::Quit),
            _ => None,
        },
        RunState::ShowDebugConsole => match key.code {
            KeyCode::Enter => Some(Action::SubmitCommand),
            KeyCode::Backspace => Some(Action::Backspace),
            KeyCode::Char('`') | KeyCode::Char('~') | KeyCode::Esc => {
                Some(Action::ToggleDebugConsole)
            }
            KeyCode::Char(c) => Some(Action::TypeChar(c)),
            _ => None,
        },
        RunState::ShowHelp => match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('/') => Some(Action::CloseMenu),
            _ => None,
        },
        RunState::LevelUp | RunState::ShowResetShrine => match key.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::MenuSelect),
            _ => None,
        },
        RunState::ShowShop => match key.code {
            KeyCode::Esc => Some(Action::CloseMenu),
            KeyCode::Tab => Some(Action::ToggleShopMode),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::MenuSelect),
            _ => None,
        },
        RunState::ShowIdentify | RunState::ShowAlchemy => match key.code {
            KeyCode::Esc => Some(Action::CloseMenu),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::MenuSelect),
            _ => None,
        },
        RunState::ShowSpells => match key.code {
            KeyCode::Esc | KeyCode::Char('a') => Some(Action::CloseMenu),
            KeyCode::Up | KeyCode::Char('k') => Some(Action::MenuUp),
            KeyCode::Down | KeyCode::Char('j') => Some(Action::MenuDown),
            KeyCode::Enter => Some(Action::MenuSelect),
            _ => None,
        },
        RunState::ShowShrine | RunState::ShowStudyTome => match key.code {
            KeyCode::Char('y') | KeyCode::Enter => Some(Action::Confirm),
            KeyCode::Char('n') | KeyCode::Esc => Some(Action::Decline),
            _ => None,
        },
        RunState::Dead | RunState::Victory => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn mock_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn test_map_key_movement() {
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('h')), RunState::AwaitingInput),
            Some(Action::MovePlayer(-1, 0))
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Left), RunState::AwaitingInput),
            Some(Action::MovePlayer(-1, 0))
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('l')), RunState::AwaitingInput),
            Some(Action::MovePlayer(1, 0))
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Right), RunState::AwaitingInput),
            Some(Action::MovePlayer(1, 0))
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('k')), RunState::AwaitingInput),
            Some(Action::MovePlayer(0, -1))
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Up), RunState::AwaitingInput),
            Some(Action::MovePlayer(0, -1))
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('j')), RunState::AwaitingInput),
            Some(Action::MovePlayer(0, 1))
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Down), RunState::AwaitingInput),
            Some(Action::MovePlayer(0, 1))
        );
    }

    #[test]
    fn test_map_key_menus() {
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('i')), RunState::AwaitingInput),
            Some(Action::OpenInventory)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('m')), RunState::AwaitingInput),
            Some(Action::OpenLogHistory)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('b')), RunState::AwaitingInput),
            Some(Action::OpenBestiary)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('?')), RunState::AwaitingInput),
            Some(Action::OpenHelp)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Enter), RunState::AwaitingInput),
            Some(Action::TryLevelTransition)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char(' ')), RunState::AwaitingInput),
            Some(Action::Wait)
        );
    }

    #[test]
    fn test_map_key_inventory() {
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Esc), RunState::ShowInventory),
            Some(Action::CloseMenu)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('i')), RunState::ShowInventory),
            Some(Action::CloseMenu)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Up), RunState::ShowInventory),
            Some(Action::MenuUp)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Down), RunState::ShowInventory),
            Some(Action::MenuDown)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Enter), RunState::ShowInventory),
            Some(Action::MenuSelect)
        );
    }

    #[test]
    fn test_map_key_targeting() {
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('h')), RunState::ShowTargeting),
            Some(Action::MovePlayer(-1, 0))
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Enter), RunState::ShowTargeting),
            Some(Action::MenuSelect)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Esc), RunState::ShowTargeting),
            Some(Action::CloseMenu)
        );
    }

    #[test]
    fn test_map_key_shop() {
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Tab), RunState::ShowShop),
            Some(Action::ToggleShopMode)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Esc), RunState::ShowShop),
            Some(Action::CloseMenu)
        );
    }

    #[test]
    fn test_map_key_end_screens() {
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('q')), RunState::Dead),
            Some(Action::Quit)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Esc), RunState::Victory),
            Some(Action::Quit)
        );
    }

    #[test]
    fn test_map_key_menus_more() {
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('m')), RunState::ShowLogHistory),
            Some(Action::CloseMenu)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Esc), RunState::ShowLogHistory),
            Some(Action::CloseMenu)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Up), RunState::ShowLogHistory),
            Some(Action::MenuUp)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Down), RunState::ShowLogHistory),
            Some(Action::MenuDown)
        );

        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('b')), RunState::ShowBestiary),
            Some(Action::CloseMenu)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Esc), RunState::ShowBestiary),
            Some(Action::CloseMenu)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Up), RunState::ShowBestiary),
            Some(Action::MenuUp)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Down), RunState::ShowBestiary),
            Some(Action::MenuDown)
        );

        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('?')), RunState::ShowHelp),
            Some(Action::CloseMenu)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Esc), RunState::ShowHelp),
            Some(Action::CloseMenu)
        );

        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Enter), RunState::LevelUp),
            Some(Action::MenuSelect)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Up), RunState::LevelUp),
            Some(Action::MenuUp)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Down), RunState::LevelUp),
            Some(Action::MenuDown)
        );

        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Esc), RunState::ShowAlchemy),
            Some(Action::CloseMenu)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Enter), RunState::ShowIdentify),
            Some(Action::MenuSelect)
        );
    }

    #[test]
    fn test_map_key_edge_cases() {
        // AwaitingInput
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('q')), RunState::AwaitingInput),
            Some(Action::Quit)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('g')), RunState::AwaitingInput),
            Some(Action::PickUpItem)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('/')), RunState::AwaitingInput),
            Some(Action::OpenHelp)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('.')), RunState::AwaitingInput),
            Some(Action::Wait)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Tab), RunState::AwaitingInput),
            None
        );

        // ShowShop more
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Up), RunState::ShowShop),
            Some(Action::MenuUp)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Down), RunState::ShowShop),
            Some(Action::MenuDown)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Enter), RunState::ShowShop),
            Some(Action::MenuSelect)
        );

        // ShowIdentify/Alchemy more
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Up), RunState::ShowIdentify),
            Some(Action::MenuUp)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Down), RunState::ShowIdentify),
            Some(Action::MenuDown)
        );

        // Dead/Victory
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Esc), RunState::Dead),
            Some(Action::Quit)
        );
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('x')), RunState::Dead),
            None
        );

        // Default case
        assert_eq!(
            map_key_to_action(mock_key(KeyCode::Char('a')), RunState::MonsterTurn),
            None
        );
    }
}
