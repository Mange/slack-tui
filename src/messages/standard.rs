use std::hash::{Hash, Hasher};
use std::cmp::{Ord, Ordering, PartialOrd};

use super::MessageID;
use canvas::Canvas;

#[derive(Clone, Debug)]
pub struct StandardMessage {
    pub message_id: MessageID,
    pub thread_id: MessageID,
    pub from: String,
    pub body: String,
}

impl Hash for StandardMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.message_id.hash(state)
    }
}

impl PartialEq for StandardMessage {
    fn eq(&self, rhs: &StandardMessage) -> bool {
        self.message_id.eq(&rhs.message_id)
    }
}

impl Eq for StandardMessage {}

impl PartialOrd for StandardMessage {
    fn partial_cmp(&self, rhs: &StandardMessage) -> Option<Ordering> {
        self.message_id.partial_cmp(&rhs.message_id)
    }
}

impl Ord for StandardMessage {
    fn cmp(&self, rhs: &StandardMessage) -> Ordering {
        self.message_id.cmp(&rhs.message_id)
    }
}

impl StandardMessage {
    pub fn id(&self) -> &MessageID {
        &self.message_id
    }

    pub fn render_as_canvas(&self, width: u16) -> Canvas {
        use tui::style::*;

        let underlined = Style::default().modifier(Modifier::Underline);
        let mut canvas = Canvas::new(width);
        canvas.add_string_truncated(&self.from, underlined);
        if self.from.len() < width as usize {
            canvas.add_string_truncated("\n", Style::default());
        }
        canvas.add_string_wrapped(&format!("{}\n", self.body), Style::default());

        canvas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_renders_as_canvas() {
        let message = StandardMessage {
            from: "Bear Grylls".into(),
            body: "I'm lost. I guess I have to drink my own urine. :)".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
        };

        let big_canvas = message.render_as_canvas(50);
        assert_eq!(
            &big_canvas.render_to_string(Some("|")),
            "Bear Grylls                                       |
I'm lost. I guess I have to drink my own urine. :)|",
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
