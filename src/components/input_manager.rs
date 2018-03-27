use termion::event::Key;

use components::App;
use models::Mode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Quit,
}

#[derive(Debug)]
pub struct KeyManager {}

impl KeyManager {
    pub fn new() -> Self {
        KeyManager {}
    }

    pub fn handle_key(&mut self, app: &mut App, input: Key) -> Outcome {
        match app.state().current_mode() {
            &Mode::History => self.handle_history_key(app, input),
            &Mode::SelectChannel => self.handle_select_channel_key(app, input),
        }
    }

    fn handle_history_key(&mut self, app: &mut App, input: Key) -> Outcome {
        match input {
            Key::Char('q') => return Outcome::Quit,
            Key::Char('j') => app.state_mut().scroll_down(1),
            Key::Char('k') => app.state_mut().scroll_up(1),
            Key::Ctrl('f') => {
                // Leave 2 lines from last page visible
                let page_size = app.state().chat_height().saturating_sub(2);
                app.state_mut().scroll_down(page_size as usize);
            }
            Key::Ctrl('b') => {
                // Leave 2 lines from last page visible
                let page_size = app.state().chat_height().saturating_sub(2);
                app.state_mut().scroll_up(page_size as usize)
            }
            Key::Char('G') => {
                let distance = app.state().current_history_scroll();
                app.state_mut().scroll_down(distance)
            }
            Key::Char('b') => app.state_mut().add_fake_message(None),
            Key::Char('B') => app.state_mut().toggle_loading_state(),
            Key::Ctrl('k') => app.state_mut().enter_mode(Mode::SelectChannel),
            _ => {}
        }
        Outcome::Continue
    }

    fn handle_select_channel_key(&mut self, app: &mut App, input: Key) -> Outcome {
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
                app.state_mut().enter_mode(Mode::History);
            }
            Key::Esc => {
                app.channel_selector.reset();
                app.state_mut().enter_mode(Mode::History);
            }
            Key::Char(chr) => app.channel_selector.add_character(chr),
            _ => {}
        }
        Outcome::Continue
    }
}
