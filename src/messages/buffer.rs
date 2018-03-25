use std::collections::BTreeMap;

use messages::*;
use canvas::Canvas;

pub struct Buffer {
    messages: BTreeMap<MessageID, Message>,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            messages: BTreeMap::new(),
        }
    }

    pub fn add<E: HistoryEntry>(&mut self, entry: E) {
        let message = entry.into_message();
        self.messages.insert(message.id().clone(), message);
    }

    pub fn render_as_canvas(&self, width: u16, is_loading_more_messages: bool) -> Canvas {
        use tui::style::Style;

        let mut canvas = Canvas::new(width);
        if is_loading_more_messages {
            canvas += LoadingMessage::new().render_as_canvas(width);
        }

        for (_id, message) in &self.messages {
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
        let mut message_buffer = Buffer::new();
        message_buffer.add(StandardMessage {
            from: "Example".into(),
            body: "Hello...".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
        });
        message_buffer.add(StandardMessage {
            from: "Example".into(),
            body: "...World!".into(),
            message_id: "1110001.0000".into(),
            thread_id: "1110001.0000".into(),
        });

        let canvas = message_buffer.render_as_canvas(10, false);
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
        let mut message_buffer = Buffer::new();
        message_buffer.add(StandardMessage {
            from: "Example".into(),
            body: "Hello World".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
        });

        let canvas = message_buffer.render_as_canvas(50, true);
        assert_eq!(
            &canvas.render_to_string(Some("|")),
            "              Loading more messages               |
Example                                           |
Hello World                                       |
                                                  |"
        );
    }
}
