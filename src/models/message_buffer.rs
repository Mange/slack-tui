use std::collections::BTreeMap;

use models::messages::*;
use models::{Canvas, ChannelID};

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

    pub fn render_as_canvas(
        &self,
        channel_id: &ChannelID,
        width: u16,
        is_loading_more_messages: bool,
    ) -> Canvas {
        use tui::style::Style;

        let mut canvas = Canvas::new(width);
        if is_loading_more_messages {
            canvas += LoadingMessage::new().render_as_canvas(width);
        }

        for (_id, message) in self.messages
            .iter()
            .filter(|&(_, m)| m.channel_id() == channel_id)
        {
            canvas += message.render_as_canvas(width);
            canvas.add_string_truncated("\n", Style::default());
        }
        canvas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_renders_messages_as_canvas() {
        let mut message_buffer = MessageBuffer::new();
        message_buffer.add(StandardMessage {
            from: "Example".into(),
            body: "Hello...".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
            channel_id: "C1".into(),
        });
        message_buffer.add(StandardMessage {
            from: "Example".into(),
            body: "...World!".into(),
            message_id: "1110001.0000".into(),
            thread_id: "1110001.0000".into(),
            channel_id: "C1".into(),
        });

        let canvas = message_buffer.render_as_canvas(&"C1".into(), 10, false);
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
        let mut message_buffer = MessageBuffer::new();
        message_buffer.add(StandardMessage {
            from: "Example".into(),
            body: "Hello World".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
            channel_id: "C1".into(),
        });

        let canvas = message_buffer.render_as_canvas(&"C1".into(), 50, true);
        assert_eq!(
            &canvas.render_to_string(Some("|")),
            "              Loading more messages               |
Example                                           |
Hello World                                       |
                                                  |"
        );
    }

    #[test]
    fn it_skips_messages_in_other_channels() {
        let mut message_buffer = MessageBuffer::new();
        message_buffer.add(StandardMessage {
            from: "Example".into(),
            body: "First channel".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
            channel_id: "C1".into(),
        });
        message_buffer.add(StandardMessage {
            from: "Example".into(),
            body: "Second channel".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
            channel_id: "C2".into(),
        });

        let canvas = message_buffer.render_as_canvas(&"C2".into(), 50, false);
        let rendered = canvas.render_to_string(Some("|"));

        assert!(rendered.contains("Second channel"));
        assert!(!rendered.contains("First channel"));
    }
}
