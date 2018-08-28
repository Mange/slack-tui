use std::cmp::{Ord, Ordering, PartialOrd};
use std::hash::{Hash, Hasher};

use failure::Error;
use slack::api;

use super::prelude::*;
use models::UserID;

#[derive(Clone, Debug)]
pub struct StandardMessage {
    pub message_id: MessageID,
    pub thread_id: MessageID,
    pub channel_id: ChannelID,
    pub user_id: UserID,
    pub body: String,
}

impl StandardMessage {
    pub fn from_slack(
        msg: &api::MessageStandard,
        side_channel: &MessageSideChannel,
    ) -> Result<Self, Error> {
        let ts = match msg.ts.clone() {
            Some(val) => val,
            None => return Err(format_err!("Message had no ts:\n{:#?}", msg)),
        };

        let channel_id = match msg
            .channel
            .clone()
            .map(ChannelID::from)
            .or_else(|| side_channel.channel_id.clone())
        {
            Some(val) => val,
            None => return Err(format_err!("Message had no channel:\n{:#?}", msg)),
        };

        let message_id = MessageID::from(ts);
        let thread_id = msg
            .ts
            .clone()
            .map(MessageID::from)
            .unwrap_or_else(|| message_id.clone());

        Ok(StandardMessage {
            message_id,
            thread_id,
            channel_id,
            body: msg.text.clone().unwrap_or_else(|| String::new()),
            user_id: msg.user.clone().map(UserID::from).unwrap(),
        })
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

    fn render_as_canvas(&self, state: &AppState, width: u16) -> Canvas {
        use tui::style::*;

        let user = state.users.get(&self.user_id);

        let underlined = Style::default().modifier(Modifier::Underline);
        let mut canvas = Canvas::new(width);
        match user {
            Some(user) => {
                let color_style = underlined.clone().fg(user.color());
                canvas.add_string_truncated(user.display_name(), color_style)
            }
            None => {
                canvas.add_string_truncated(self.user_id.as_str(), Style::default().fg(Color::Red))
            }
        }
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

    fn fake_slack_message() -> api::MessageStandard {
        api::MessageStandard {
            attachments: None,
            bot_id: None,
            channel: Some(String::from("C1")),
            edited: None,
            event_ts: None,
            reply_broadcast: None,
            source_team: None,
            team: None,
            text: None,
            thread_ts: None,
            ts: Some(String::from("1111")),
            ty: None,
            user: Some(String::from("U1")),
        }
    }

    #[test]
    fn it_uses_side_channel_to_fill_in_channel_when_missing() {
        let mut slack_message_with_channel = fake_slack_message();
        let mut slack_message_without_channel = fake_slack_message();

        slack_message_with_channel.channel = Some(String::from("C555"));
        slack_message_without_channel.channel = None;

        let side_channel = MessageSideChannel {
            channel_id: Some(ChannelID::from("C123")),
        };

        let message_not_using_side_channel =
            StandardMessage::from_slack(&slack_message_with_channel, &side_channel).unwrap();

        let message_using_side_channel =
            StandardMessage::from_slack(&slack_message_without_channel, &side_channel).unwrap();

        assert_eq!(
            message_not_using_side_channel.channel_id,
            ChannelID::from("C555")
        );
        assert_eq!(
            message_using_side_channel.channel_id,
            ChannelID::from("C123")
        );
    }

    #[test]
    fn it_renders_as_canvas() {
        use models::User;

        let mut state = AppState::fixture();
        state.users.add_user(User::fixture("U42", "Bear Grylls"));

        let message = StandardMessage {
            user_id: "U42".into(),
            body: "I'm lost. I guess I have to drink my own urine. :)".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
            channel_id: "C1".into(),
        };

        let big_canvas = message.render_as_canvas(&state, 50);
        assert_eq!(
            &big_canvas.render_to_string(Some("|")),
            "Bear Grylls                                       |
I'm lost. I guess I have to drink my own urine. :)|",
        );

        let small_canvas = message.render_as_canvas(&state, 20);
        assert_eq!(
            &small_canvas.render_to_string(Some("|")),
            "Bear Grylls         |
I'm lost. I guess I |
have to drink my own|
 urine. :)          |",
        );
    }

    #[test]
    fn it_renders_messages_with_many_characters() {
        let state = AppState::fixture();
        let message = StandardMessage {
            user_id: "Data Dump".into(),
            body: "Imagine that this is a lot of data:\nHello\nAgain".into(),
            message_id: "1110000.0000".into(),
            thread_id: "1110000.0000".into(),
            channel_id: "C1".into(),
        };

        let big_canvas = message.render_as_canvas(&state, 50);
        assert_eq!(
            &big_canvas.render_to_string(Some("|")),
            "Data Dump                                         |
Imagine that this is a lot of data:               |
Hello                                             |
Again                                             |",
        );
    }
}
