use std::collections::BTreeSet;

use messages::*;
use canvas::Canvas;

pub struct Buffer {
    messages: BTreeSet<Message>,
}

impl Into<Buffer> for Vec<Message> {
    fn into(self) -> Buffer {
        Buffer {
            messages: self.into_iter().collect(),
        }
    }
}

impl Buffer {
    pub fn render_as_canvas(&self, width: u16) -> Canvas {
        use tui::style::Style;

        let mut canvas = Canvas::new(width);
        for message in &self.messages {
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
        let mut message_buffer = Buffer {
            messages: BTreeSet::new(),
        };
        message_buffer.messages.insert(Message {
            from: "Example".into(),
            body: "Hello...".into(),
            timestamp: "1110000.0000".into(),
        });
        message_buffer.messages.insert(Message {
            from: "Example".into(),
            body: "...World!".into(),
            timestamp: "1110001.0000".into(),
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
