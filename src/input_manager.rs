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
        use event::Key::*;
        match input {
            Char('q') => return Outcome::Quit,
            Char('j') => app.scroll_down(),
            Char('k') => app.scroll_up(),
            Char('b') => app.create_fake_message(),
            Char('B') => app.add_loading_message(),
            Ctrl('k') => app.enter_mode(Mode::SelectChannel),
            Esc => app.enter_mode(Mode::History),
            _ => {}
        }
        Outcome::Continue
    }
}
