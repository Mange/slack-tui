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
            Char('b') => app.add_fake_message(None),
            Char('B') => app.add_loading_message(),
            Ctrl('k') => app.enter_mode(Mode::SelectChannel),
            _ => {}
        }
        Outcome::Continue
    }

    pub fn handle_select_channel_key(&mut self, app: &mut App, input: event::Key) -> Outcome {
        use event::Key;
        match input {
            Key::Backspace => app.channel_selector.delete_character(),
            Key::Ctrl('w') => app.channel_selector.delete_word(),
            Key::Ctrl('a') => app.channel_selector.move_to_beginning(),
            Key::Ctrl('e') => app.channel_selector.move_to_end(),
            Key::Ctrl('k') => app.channel_selector.reset(),
            Key::Left => app.channel_selector.move_cursor_left(),
            Key::Right => app.channel_selector.move_cursor_right(),
            Key::Up => app.channel_selector.select_previous_match(),
            Key::Down => app.channel_selector.select_next_match(),
            Key::Char('\n') => {
                app.select_channel_from_selector();
                app.channel_selector.reset();
                app.enter_mode(Mode::History);
            }
            Key::Esc => {
                app.channel_selector.reset();
                app.enter_mode(Mode::History);
            }
            Key::Char(chr) => app.channel_selector.add_character(chr),
            _ => {}
        }
        Outcome::Continue
    }
}
