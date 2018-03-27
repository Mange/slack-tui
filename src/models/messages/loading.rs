use models::Canvas;

#[derive(Clone, Debug)]
pub struct LoadingMessage {}

impl LoadingMessage {
    pub fn new() -> Self {
        LoadingMessage {}
    }

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
        let message = LoadingMessage::new();

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
