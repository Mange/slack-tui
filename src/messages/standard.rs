use std::hash::{Hash, Hasher};
use std::cmp::{Ord, Ordering, PartialOrd};

use canvas::Canvas;

#[derive(Clone, Debug, Deserialize)]
pub struct StandardMessage {
    pub timestamp: String,
    pub from: String,
    pub body: String,
}

impl Hash for StandardMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.timestamp.hash(state)
    }
}

impl PartialEq for StandardMessage {
    fn eq(&self, rhs: &StandardMessage) -> bool {
        self.timestamp.eq(&rhs.timestamp)
    }
}

impl Eq for StandardMessage {}

impl PartialOrd for StandardMessage {
    fn partial_cmp(&self, rhs: &StandardMessage) -> Option<Ordering> {
        self.timestamp.partial_cmp(&rhs.timestamp)
    }
}

impl Ord for StandardMessage {
    fn cmp(&self, rhs: &StandardMessage) -> Ordering {
        self.timestamp.cmp(&rhs.timestamp)
    }
}

impl StandardMessage {
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
            timestamp: "1110000.0000".into(),
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
