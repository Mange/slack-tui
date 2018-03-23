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

    pub fn add<M: Into<Message>>(&mut self, message: M) {
        let message = message.into();
        self.messages.insert(message.id().clone(), message);
    }

    pub fn render_as_canvas(&self, width: u16) -> Canvas {
        use tui::style::Style;

        let mut canvas = Canvas::new(width);
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

        let canvas = message_buffer.render_as_canvas(10);
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
}
