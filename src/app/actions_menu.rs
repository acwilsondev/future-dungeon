use crate::actions::Action;
use crate::app::{App, RunState};

impl App {
    pub fn handle_main_menu_input(&mut self, action: Action) {
        let has_save = crate::persistence::has_save_game();
        let max_options = if has_save { 3 } else { 2 };

        match action {
            Action::MenuUp if self.main_menu_cursor > 0 => {
                self.main_menu_cursor -= 1;
            }
            Action::MenuDown if self.main_menu_cursor < max_options - 1 => {
                self.main_menu_cursor += 1;
            }
            Action::MenuSelect => {
                let selection = if !has_save && self.main_menu_cursor == 1 {
                    2 // Exit if no save and selected 2nd option
                } else {
                    self.main_menu_cursor
                };

                match selection {
                    0 => {
                        // New Game
                        self.generate_level(Vec::new());
                        self.state = RunState::ShowClassSelection;
                    }
                    1 => {
                        // Load Game
                        if let Ok(Some(loaded_app)) = crate::persistence::load_game() {
                            *self = loaded_app;
                            self.state = RunState::AwaitingInput;
                        }
                    }
                    2 => {
                        // Exit
                        self.exit = true;
                    }
                    _ => {}
                }
            }
            Action::Quit => self.exit = true,
            _ => {}
        }
    }
}
