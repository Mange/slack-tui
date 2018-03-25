use std::hash::{Hash, Hasher};
use std::cmp::{Ord, Ordering, PartialOrd};

use slack::api;

use super::prelude::*;

#[derive(Clone, Debug)]
pub struct StandardMessage {
    pub message_id: MessageID,
    pub thread_id: MessageID,
    pub channel_id: ChannelID,
    pub from: String,
    pub body: String,
}

impl<'a> From<&'a api::MessageStandard> for StandardMessage {
    fn from(msg: &'a api::MessageStandard) -> Self {
        let message_id = MessageID::from(msg.ts.clone().unwrap());
        let thread_id = msg.ts
            .clone()
            .map(MessageID::from)
            .unwrap_or_else(|| message_id.clone());
        let channel_id = ChannelID::from(msg.channel.clone().unwrap());

        StandardMessage {
            message_id,
            thread_id,
            channel_id,
            body: msg.text.clone().unwrap(),
            from: msg.user.clone().unwrap(),
        }
    }
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

impl HistoryEntry for StandardMessage {
    fn id(&self) -> &MessageID {
        &self.message_id
    }

    fn channel_id(&self) -> &ChannelID {
        &self.channel_id
    }

    fn render_as_canvas(&self, width: u16) -> Canvas {
        use tui::style::*;

        let underlined = Style::default().modifier(Modifier::Underline);
        let mut canvas = Canvas::new(width);
        canvas.add_string_truncated(&self.from, underlined);
        canvas.add_string_truncated("\n", Style::default());
        canvas.add_string_wrapped(&format!("{}\n", self.body), Style::default());

        canvas
    }

    fn into_message(self) -> Message {
        Message::Standard(self)
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
            channel_id: "C1".into(),
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
