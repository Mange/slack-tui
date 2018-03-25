use std::hash::{Hash, Hasher};
use std::cmp::{Ord, Ordering, PartialOrd};

use failure::Error;

use super::prelude::*;

#[derive(Clone, Debug)]
pub struct UnsupportedMessage {
    pub id: MessageID,
    pub channel_id: ChannelID,
    pub from: String,
    pub text: String,
    pub subtype: Option<String>,
}

impl Hash for UnsupportedMessage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl PartialEq for UnsupportedMessage {
    fn eq(&self, rhs: &UnsupportedMessage) -> bool {
        self.id.eq(&rhs.id)
    }
}

impl Eq for UnsupportedMessage {}

impl PartialOrd for UnsupportedMessage {
    fn partial_cmp(&self, rhs: &UnsupportedMessage) -> Option<Ordering> {
        self.id.partial_cmp(&rhs.id)
    }
}

impl Ord for UnsupportedMessage {
    fn cmp(&self, rhs: &UnsupportedMessage) -> Ordering {
        self.id.cmp(&rhs.id)
    }
}

impl UnsupportedMessage {
    pub fn from_slack_message(
        id: &Option<String>,
        channel_id: &Option<String>,
        from: &Option<String>,
        text: &Option<String>,
        subtype: &Option<String>,
    ) -> Result<UnsupportedMessage, Error> {
        Ok(UnsupportedMessage {
            id: id.clone()
                .map(MessageID::from)
                .ok_or_else(|| format_err!("ID was blank"))?,
            channel_id: channel_id
                .clone()
                .map(ChannelID::from)
                .ok_or_else(|| format_err!("Channel ID was blank"))?,
            from: from.clone().ok_or_else(|| format_err!("from is missing"))?,
            text: text.clone().ok_or_else(|| format_err!("text is missing"))?,
            subtype: subtype.clone(),
        })
    }
}

impl HistoryEntry for UnsupportedMessage {
    fn id(&self) -> &MessageID {
        &self.id
    }

    fn channel_id(&self) -> &ChannelID {
        &self.channel_id
    }

    fn render_as_canvas(&self, width: u16) -> Canvas {
        use tui::style::*;

        let faint = Style::default().modifier(Modifier::Faint);
        let mut canvas = Canvas::new(width);
        canvas.add_string_truncated(&self.from, faint);
        canvas.add_string_truncated("\n", faint);
        canvas.add_string_wrapped(
            &format!(
                "(Unsupported message {})\n{}\n",
                self.subtype.as_ref().map(String::as_ref).unwrap_or("?"),
                self.text
            ),
            faint,
        );

        canvas
    }

    fn into_message(self) -> Message {
        Message::Unsupported(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_renders_as_canvas() {
        let message = UnsupportedMessage {
            id: "1110000.000000".into(),
            channel_id: "C1".into(),
            from: "Mystery".into(),
            text: "Thing happened".into(),
            subtype: Some(String::from("mystery_event")),
        };

        let big_canvas = message.render_as_canvas(50);
        assert_eq!(
            &big_canvas.render_to_string(Some("|")),
            "Mystery                                           |
(Unsupported message mystery_event)               |
Thing happened                                    |"
        );

        let small_canvas = message.render_as_canvas(15);
        assert_eq!(
            &small_canvas.render_to_string(Some("|")),
            "Mystery        |
(Unsupported me|
ssage mystery_e|
vent)          |
Thing happened |"
        );
    }
}
