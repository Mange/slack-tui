use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::cmp::{Ord, Ordering, PartialOrd};
use canvas::Canvas;

#[derive(Clone, Debug, Deserialize)]
pub struct Message {
    pub timestamp: String,
    pub from: String,
    pub body: String,
}

impl Hash for Message {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.timestamp.hash(state)
    }
}

impl PartialEq for Message {
    fn eq(&self, rhs: &Message) -> bool {
        self.timestamp.eq(&rhs.timestamp)
    }
}

impl Eq for Message {}

impl PartialOrd for Message {
    fn partial_cmp(&self, rhs: &Message) -> Option<Ordering> {
        self.timestamp.partial_cmp(&rhs.timestamp)
    }
}

impl Ord for Message {
    fn cmp(&self, rhs: &Message) -> Ordering {
        self.timestamp.cmp(&rhs.timestamp)
    }
}

impl Message {
    fn render_as_canvas(&self, width: u16) -> Canvas {
        use tui::style::*;

        let underlined = Style::default().modifier(Modifier::Underline);
        let mut canvas = Canvas::new(width);
        canvas.add_string_truncated(&format!("{}\n", self.from), underlined);
        canvas.add_string_wrapped(&format!("{}\n", self.body), Style::default());

        canvas
    }
}

pub struct MessageBuffer {
    messages: BTreeSet<Message>,
}

impl Into<MessageBuffer> for Vec<Message> {
    fn into(self) -> MessageBuffer {
        MessageBuffer {
            messages: self.into_iter().collect(),
        }
    }
}

impl MessageBuffer {
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

    mod message {
        use super::*;

        #[test]
        fn it_renders_as_canvas() {
            let message = Message {
                from: "Bear Grylls".into(),
                body: "I'm lost. I guess I have to drink my own urine. :)".into(),
                timestamp: "1110000.0000".into(),
            };

            let big_canvas = message.render_as_canvas(50);
            assert_eq!(
                &big_canvas.render_to_string(Some("|")),
                "Bear Grylls                                       |
I'm lost. I guess I have to drink my own urine. :)|
                                                  |",
            );

            let small_canvas = message.render_as_canvas(20);
            assert_eq!(
                &small_canvas.render_to_string(Some("|")),
                "Bear Grylls         |
I'm lost. I guess I |
have to drink my own|
 urine. :)          |",
            );
        }
    }

    mod message_buffer {
        use super::*;

        #[test]
        fn it_renders_messages_as_canvas() {
            let mut message_buffer = MessageBuffer {
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
}
