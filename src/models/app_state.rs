use chrono::prelude::*;
use failure::{Error, Fail};
use std::cell::{Cell, Ref, RefCell};

use models::*;

#[derive(Debug)]
pub struct AppState {
    pub current_mode: Mode,

    pub chat_canvas: RefCell<Option<Canvas>>,
    pub history_scroll: usize,
    pub last_chat_height: Cell<u16>,

    pub selected_channel_id: ChannelID,
    pub channels: ChannelList,

    pub is_loading_more_messages: bool,
    pub messages: MessageBuffer,

    pub team_name: String,
    pub users: UserList,
}

impl AppState {
    pub fn rendered_chat_canvas(&self, width: u16, height: u16) -> Ref<Canvas> {
        // Populate RefCell inside this scope when not present.
        {
            let mut cache = self.chat_canvas.borrow_mut();
            if cache.is_none() {
                let canvas = self.messages.render_as_canvas(
                    &self.selected_channel_id,
                    width,
                    self.is_loading_more_messages,
                );
                *cache = Some(canvas);
            }
        }

        self.last_chat_height.replace(height);

        // By now we know that chat_canvas is_some(), so unwrap should be safe. Option::as_ref
        // returns a new option of a reference to inner value, so it's fine to consume that Option.
        Ref::map(self.chat_canvas.borrow(), |option| option.as_ref().unwrap())
    }

    pub fn selected_channel(&self) -> Option<&Channel> {
        self.channels.get(&self.selected_channel_id)
    }

    pub fn selected_channel_id(&self) -> &ChannelID {
        &self.selected_channel_id
    }

    pub fn max_history_scroll(&self) -> usize {
        // NOTE: Scroll value is distance from bottom
        let chat_canvas_height = {
            match *self.chat_canvas.borrow() {
                Some(ref canvas) => canvas.height(),
                None => return 0,
            }
        };
        let chat_viewport_height = self.chat_height();

        // If the canvas is smaller than the viewport, lock to bottom.
        if chat_canvas_height <= chat_viewport_height {
            0
        } else {
            chat_canvas_height as usize - chat_viewport_height as usize
        }
    }

    pub fn current_history_scroll(&self) -> usize {
        self.history_scroll.min(self.max_history_scroll())
    }

    pub fn chat_height(&self) -> u16 {
        self.last_chat_height.get()
    }

    pub fn clear_chat_canvas_cache(&self) {
        self.chat_canvas.replace(None);
    }

    pub fn current_mode(&self) -> &Mode {
        &self.current_mode
    }

    pub fn enter_mode(&mut self, new_mode: Mode) {
        self.current_mode = new_mode;
    }

    pub fn scroll_down(&mut self, amount: usize) {
        // NOTE: Scroll value is distance from bottom
        self.history_scroll = self.current_history_scroll().saturating_sub(amount);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.history_scroll =
            (self.history_scroll.saturating_add(amount)).min(self.max_history_scroll());
    }

    pub fn select_channel(&mut self, id: ChannelID) -> Result<(), Error> {
        self.selected_channel_id = id;
        self.history_scroll = 0;
        self.clear_chat_canvas_cache();
        Ok(())
    }

    pub fn toggle_loading_state(&mut self) {
        let new_state = !self.is_loading_more_messages;
        self.set_loading_state(new_state);
    }

    pub fn set_loading_state(&mut self, state: bool) {
        self.is_loading_more_messages = state;
        self.clear_chat_canvas_cache();
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.add(message);
        self.clear_chat_canvas_cache();
    }

    pub fn add_fake_message(&mut self, msg: Option<&str>) {
        let time = Local::now();

        let message = match msg {
            Some(msg) => String::from(msg),
            None => format!("This is a fake message generated at: {}", time),
        };

        self.messages.add(messages::StandardMessage {
            from: "Fake Message".into(),
            body: message,
            message_id: time.into(),
            thread_id: time.into(),
            channel_id: self.selected_channel_id.clone(),
        });
        self.clear_chat_canvas_cache();
    }

    pub fn add_error_message<E: Fail>(&mut self, error: E) {
        self.messages.add(messages::ErrorMessage::from_error(
            &self.selected_channel_id,
            error,
        ));
        self.clear_chat_canvas_cache();
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Mode {
    History,
    SelectChannel,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::History
    }
}
