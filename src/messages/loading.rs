use std::hash::{Hash, Hasher};
use std::cmp::{Ord, Ordering, PartialOrd};

use canvas::Canvas;

#[derive(Clone, Debug, Deserialize)]
pub struct LoadingMessage {
    pub from_timestamp: String,
}

impl Hash for LoadingMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.from_timestamp.hash(state)
    }
}

impl PartialEq for LoadingMessage {
    fn eq(&self, rhs: &LoadingMessage) -> bool {
        self.from_timestamp.eq(&rhs.from_timestamp)
    }
}

impl Eq for LoadingMessage {}

impl PartialOrd for LoadingMessage {
    fn partial_cmp(&self, rhs: &LoadingMessage) -> Option<Ordering> {
        self.from_timestamp.partial_cmp(&rhs.from_timestamp)
    }
}

impl Ord for LoadingMessage {
    fn cmp(&self, rhs: &LoadingMessage) -> Ordering {
        self.from_timestamp.cmp(&rhs.from_timestamp)
    }
}

impl LoadingMessage {
    pub fn render_as_canvas(&self, width: u16) -> Canvas {
        use tui::style::*;

        let mut canvas = Canvas::new(width);
        canvas.add_string_truncated(
            &format!("{:^1$}", "Loading more messages", width as usize),
            Style::default().fg(Color::Red),
        );

        canvas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_renders_as_canvas() {
        let message = LoadingMessage {
            from_timestamp: "1110000.0000".into(),
        };

        let big_canvas = message.render_as_canvas(50);
        assert_eq!(
            &big_canvas.render_to_string(Some("|")),
            "              Loading more messages               |"
        );

        let small_canvas = message.render_as_canvas(20);
        assert_eq!(
            &small_canvas.render_to_string(Some("|")),
            "Loading more message|",
        );
    }
}
