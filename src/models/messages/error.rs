use std::cmp::{Ord, Ordering, PartialOrd};
use std::hash::{Hash, Hasher};

use chrono::prelude::*;
use failure::Fail;

use super::prelude::*;
use util::format_error_with_causes;

#[derive(Clone, Debug)]
pub struct ErrorMessage {
    pub id: MessageID,
    pub channel_id: ChannelID,
    pub text: String,
}

impl Hash for ErrorMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl PartialEq for ErrorMessage {
    fn eq(&self, rhs: &ErrorMessage) -> bool {
        self.id.eq(&rhs.id)
    }
}

impl Eq for ErrorMessage {}

impl PartialOrd for ErrorMessage {
    fn partial_cmp(&self, rhs: &ErrorMessage) -> Option<Ordering> {
        self.id.partial_cmp(&rhs.id)
    }
}

impl Ord for ErrorMessage {
    fn cmp(&self, rhs: &ErrorMessage) -> Ordering {
        self.id.cmp(&rhs.id)
    }
}

impl ErrorMessage {
    pub fn from_error<E: Fail>(channel_id: &ChannelID, error: E) -> ErrorMessage {
        ErrorMessage {
            id: Local::now().into(),
            channel_id: channel_id.clone(),
            text: format_error_with_causes(error),
        }
    }
}

impl HistoryEntry for ErrorMessage {
    fn id(&self) -> &MessageID {
        &self.id
    }

    fn channel_id(&self) -> &ChannelID {
        &self.channel_id
    }

    fn render_as_canvas(&self, _state: &AppState, width: u16) -> Canvas {
        use tui::style::*;

        let red = Style::default().fg(Color::Red);
        let mut canvas = Canvas::new(width);
        canvas.add_string_truncated("Error\n", red);
        canvas.add_string_wrapped(&self.text, red);

        canvas
    }

    fn into_message(self) -> Message {
        Message::Error(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_renders_as_canvas() {
        let state = AppState::fixture();
        let message = ErrorMessage {
            id: "1110000.000000".into(),
            channel_id: "C1".into(),
            text: "Thing happened".into(),
        };

        let big_canvas = message.render_as_canvas(&state, 50);
        assert_eq!(
            &big_canvas.render_to_string(Some("|")),
            "Error                                             |
Thing happened                                    |"
        );
    }
}
