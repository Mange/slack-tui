use std::collections::BTreeMap;

use models::messages::*;
use models::{AppState, Canvas, ChannelID};

#[derive(Debug, Default)]
pub struct MessageBuffer {
    messages: BTreeMap<MessageID, Message>,
}

impl MessageBuffer {
    pub fn new() -> Self {
        MessageBuffer {
            messages: BTreeMap::new(),
        }
    }

    pub fn add<E: HistoryEntry>(&mut self, entry: E) {
        let message = entry.into_message();
        self.messages.insert(message.id().clone(), message);
    }

    pub fn render_as_canvas(&self, state: &AppState, width: u16) -> Canvas {
        use tui::style::Style;

        let mut canvas = Canvas::new(width);
        if state.is_loading_more_messages {
            canvas += LoadingMessage::new().render_as_canvas(state, width);
        }

        for (_id, message) in self.messages
            .iter()
            .filter(|&(_, m)| m.channel_id() == state.selected_channel_id())
        {
            canvas += message.render_as_canvas(state, width);
            canvas.add_string_truncated("\n", Style::default());
        }
        canvas
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::User;

    #[test]
    fn it_renders_messages_as_canvas() {
        let mut state = AppState::fixture();
        state.selected_channel_id = ChannelID::from("C1");
        state.users.add_user(User::fixture("U55", "Example"));

        let mut message_buffer = MessageBuffer::new();
        message_buffer.add(StandardMessage {
            user_id: "U55".into(),
            body: "Hello...".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
            channel_id: "C1".into(),
        });
        message_buffer.add(StandardMessage {
            user_id: "U55".into(),
            body: "...World!".into(),
            message_id: "1110001.0000".into(),
            thread_id: "1110001.0000".into(),
            channel_id: "C1".into(),
        });

        let canvas = message_buffer.render_as_canvas(&state, 10);
        assert_eq!(
            &canvas.render_to_string(Some("|")),
            "Example   |
Hello...  |
          |
Example   |
...World! |
          |"
        );
    }

    #[test]
    fn it_adds_loading_message_when_loading() {
        let mut state = AppState::fixture();
        let mut message_buffer = MessageBuffer::new();
        message_buffer.add(StandardMessage {
            user_id: "U55".into(),
            body: "Hello World".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
            channel_id: "C1".into(),
        });

        state.selected_channel_id = ChannelID::from("C1");
        state.is_loading_more_messages = true;
        let canvas = message_buffer.render_as_canvas(&state, 50);
        assert_eq!(
            &canvas.render_to_string(Some("|")),
            "              Loading more messages               |
U55                                               |
Hello World                                       |
                                                  |"
        );
    }

    #[test]
    fn it_skips_messages_in_other_channels() {
        let mut state = AppState::fixture();
        state.selected_channel_id = ChannelID::from("C2");

        let mut message_buffer = MessageBuffer::new();
        message_buffer.add(StandardMessage {
            user_id: "Example".into(),
            body: "First channel".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
            channel_id: "C1".into(),
        });
        message_buffer.add(StandardMessage {
            user_id: "Example".into(),
            body: "Second channel".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
            channel_id: "C2".into(),
        });

        let canvas = message_buffer.render_as_canvas(&state, 50);
        let rendered = canvas.render_to_string(Some("|"));

        assert!(rendered.contains("Second channel"));
        assert!(!rendered.contains("First channel"));
    }
}
