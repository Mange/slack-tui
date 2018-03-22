use termion::event;

use {App, Mode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Quit,
}

pub struct KeyManager {}

impl KeyManager {
    pub fn new() -> Self {
        KeyManager {}
    }

    pub fn handle_key(&mut self, app: &mut App, input: event::Key) -> Outcome {
        match app.current_mode {
            Mode::History => self.handle_history_key(app, input),
            Mode::SelectChannel => self.handle_select_channel_key(app, input),
        }
    }

    pub fn handle_history_key(&mut self, app: &mut App, input: event::Key) -> Outcome {
        use event::Key::*;
        match input {
            Char('q') => return Outcome::Quit,
            Char('j') => app.scroll_down(),
            Char('k') => app.scroll_up(),
            Char('b') => app.create_fake_message(),
            Char('B') => app.add_loading_message(),
            Ctrl('k') => app.enter_mode(Mode::SelectChannel),
            _ => {}
        }
        Outcome::Continue
    }

    pub fn handle_select_channel_key(&mut self, app: &mut App, input: event::Key) -> Outcome {
        use event::Key::*;
        match input {
            Char(chr) => app.channel_selector.add_character(chr),
            Backspace => app.channel_selector.delete_character(),
            Ctrl('w') => app.channel_selector.delete_word(),
            Ctrl('a') => app.channel_selector.move_to_beginning(),
            Ctrl('e') => app.channel_selector.move_to_end(),
            Ctrl('k') => app.channel_selector.reset(),
            Left => app.channel_selector.move_cursor_left(),
            Right => app.channel_selector.move_cursor_right(),
            Esc => {
                app.channel_selector.reset();
                app.enter_mode(Mode::History);
            }
            _ => {}
        }
        Outcome::Continue
    }
}
