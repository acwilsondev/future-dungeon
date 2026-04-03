use crate::app::{App, RunState};
use crate::actions::Action;

impl App {
    pub fn process_action(&mut self, action: Action) {
        match self.state {
            RunState::AwaitingInput => self.handle_awaiting_input(action),
            RunState::ShowLogHistory => {
                match action {
                    Action::CloseMenu | Action::OpenLogHistory => self.state = RunState::AwaitingInput,
                    Action::MenuUp => if self.log_cursor > 0 { self.log_cursor -= 1; },
                    Action::MenuDown => if self.log_cursor < self.log.len().saturating_sub(1) { self.log_cursor += 1; },
                    _ => {}
                }
            }
            RunState::ShowBestiary => {
                match action {
                    Action::CloseMenu | Action::OpenBestiary => self.state = RunState::AwaitingInput,
                    Action::MenuUp => if self.bestiary_cursor > 0 { self.bestiary_cursor -= 1; },
                    Action::MenuDown => {
                        let count = self.encountered_monsters.len();
                        if count > 0 && self.bestiary_cursor < count - 1 { self.bestiary_cursor += 1; }
                    },
                    _ => {}
                }
            }
            RunState::ShowInventory => self.handle_inventory_input(action),
            RunState::ShowTargeting => self.handle_targeting_input(action),
            RunState::ShowHelp => {
                if let Action::CloseMenu | Action::OpenHelp = action {
                    self.state = RunState::AwaitingInput;
                }
            }
            RunState::LevelUp => self.handle_level_up_input(action),
            RunState::ShowShop => self.handle_shop_input(action),
            RunState::ShowIdentify => self.handle_identify_input(action),
            RunState::ShowAlchemy => self.handle_alchemy_input(action),
            RunState::Dead | RunState::Victory => {
                if let Action::Quit | Action::CloseMenu = action { self.exit = true; }
            }
            _ => {}
        }
    }

    fn handle_awaiting_input(&mut self, action: Action) {
        match action {
            Action::Quit => self.exit = true,
            Action::MovePlayer(dx, dy) => self.move_player(dx, dy),
            Action::PickUpItem => self.pick_up_item(),
            Action::OpenInventory => self.state = RunState::ShowInventory,
            Action::OpenHelp => self.state = RunState::ShowHelp,
            Action::OpenLogHistory => { self.state = RunState::ShowLogHistory; self.log_cursor = self.log.len().saturating_sub(1); }
            Action::OpenBestiary => { self.state = RunState::ShowBestiary; self.bestiary_cursor = 0; }
            Action::TryLevelTransition => self.try_level_transition(),
            Action::Wait => self.state = RunState::MonsterTurn,
            _ => {}
        }
    }
}
