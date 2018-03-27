mod error;
mod loading;
mod standard;
mod unsupported;

use std::cmp::{Ord, Ordering, PartialOrd};

use chrono::{DateTime, TimeZone};
use slack::api;
use failure::Error;

use models::{AppState, Canvas, ChannelID};

pub use self::error::ErrorMessage;
pub use self::standard::StandardMessage;
pub use self::loading::LoadingMessage;
pub use self::unsupported::UnsupportedMessage;

mod prelude {
    pub use super::{HistoryEntry, Message, MessageID, MessageSideChannel};
    pub use models::{AppState, Canvas, ChannelID};
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MessageID(String);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Message {
    Standard(StandardMessage),
    Unsupported(UnsupportedMessage),
    Error(ErrorMessage),
}

pub trait HistoryEntry {
    fn id(&self) -> &MessageID;
    fn channel_id(&self) -> &ChannelID;

    fn render_as_canvas(&self, state: &AppState, width: u16) -> Canvas;
    fn into_message(self) -> Message;
}

#[derive(Debug, Default, Clone)]
pub struct MessageSideChannel {
    pub channel_id: Option<ChannelID>,
}

fn unsupported(
    id: &Option<String>,
    channel: &Option<String>,
    from: &Option<String>,
    text: &Option<String>,
    subtype: &Option<String>,
    side_channel: &MessageSideChannel,
) -> Result<Option<Message>, Error> {
    match UnsupportedMessage::from_slack_message(id, channel, from, text, subtype, side_channel) {
        Ok(message) => Ok(Some(message.into_message())),
        Err(error) => Err(error),
    }
}

impl Message {
    pub fn from_slack_message<'a, S>(
        msg: &api::Message,
        side_channel: S,
    ) -> Result<Option<Self>, Error>
    where
        S: Into<Option<&'a MessageSideChannel>>,
    {
        use self::api::Message as S;
        let side_channel = side_channel.into().cloned().unwrap_or_default();
        match *msg {
            S::Standard(ref msg) => {
                StandardMessage::from_slack(msg, &side_channel).map(|m| Some(m.into_message()))
            }
            // TODO: slack_api does not have the "channel" key for a lot of messages.
            // Underlying cause: The https://github.com/slack-rs/slack-api-schemas repo does not
            // know what do wo with `"channel": { ... }` in the samples for these messages.
            S::BotMessage(_) => Ok(None),
            S::ChannelArchive(_) => Ok(None),
            S::ChannelJoin(_) => Ok(None),
            S::ChannelLeave(_) => Ok(None),
            S::ChannelName(_) => Ok(None),
            S::ChannelPurpose(_) => Ok(None),
            S::ChannelTopic(_) => Ok(None),
            S::ChannelUnarchive(_) => Ok(None),
            S::FileComment(_) => Ok(None),
            S::FileMention(_) => Ok(None),
            S::FileShare(_) => Ok(None),
            S::GroupArchive(_) => Ok(None),
            S::GroupJoin(_) => Ok(None),
            S::GroupLeave(_) => Ok(None),
            S::GroupName(_) => Ok(None),
            S::GroupPurpose(_) => Ok(None),
            S::GroupTopic(_) => Ok(None),
            S::GroupUnarchive(_) => Ok(None),
            S::MeMessage(_) => Ok(None),
            S::MessageChanged(ref msg) => unsupported(
                &msg.ts,
                &msg.channel,
                &msg.message.as_ref().and_then(|m| m.user.clone()),
                &msg.message.as_ref().and_then(|c| c.text.clone()),
                &msg.subtype,
                &side_channel,
            ),
            S::MessageDeleted(ref msg) => unsupported(
                &msg.ts,
                &msg.channel,
                &Some(String::from("Message deleted")),
                &Some(String::from("Message was deleted")),
                &msg.subtype,
                &side_channel,
            ),
            S::MessageReplied(ref msg) => unsupported(
                &msg.ts,
                &msg.channel,
                &msg.message.as_ref().and_then(|m| m.user.clone()),
                &msg.message.as_ref().and_then(|c| c.text.clone()),
                &msg.subtype,
                &side_channel,
            ),
            S::PinnedItem(ref msg) => unsupported(
                &msg.ts,
                &msg.channel,
                &msg.user,
                &msg.text,
                &msg.subtype,
                &side_channel,
            ),
            S::ReplyBroadcast(ref msg) => unsupported(
                &msg.ts,
                &msg.channel,
                &msg.user,
                &Some(String::from("Message got a broadcasted reply")),
                &msg.subtype,
                &side_channel,
            ),
            S::UnpinnedItem(ref msg) => unsupported(
                &msg.ts,
                &msg.channel,
                &msg.user,
                &msg.text,
                &msg.subtype,
                &side_channel,
            ),
        }
    }
}

impl HistoryEntry for Message {
    fn id(&self) -> &MessageID {
        use self::Message::*;
        match *self {
            Standard(ref msg) => msg.id(),
            Unsupported(ref msg) => msg.id(),
            Error(ref msg) => msg.id(),
        }
    }

    fn channel_id(&self) -> &ChannelID {
        use self::Message::*;
        match *self {
            Standard(ref msg) => msg.channel_id(),
            Unsupported(ref msg) => msg.channel_id(),
            Error(ref msg) => msg.channel_id(),
        }
    }

    fn render_as_canvas(&self, state: &AppState, width: u16) -> Canvas {
        use self::Message::*;
        match *self {
            Standard(ref msg) => msg.render_as_canvas(state, width),
            Unsupported(ref msg) => msg.render_as_canvas(state, width),
            Error(ref msg) => msg.render_as_canvas(state, width),
        }
    }

    fn into_message(self) -> Message {
        self
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, rhs: &Message) -> Option<Ordering> {
        self.id().partial_cmp(rhs.id())
    }
}

impl Ord for Message {
    fn cmp(&self, rhs: &Message) -> Ordering {
        self.partial_cmp(rhs).unwrap()
    }
}

impl MessageID {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn as_string(&self) -> String {
        self.0.clone()
    }
}

impl From<String> for MessageID {
    fn from(s: String) -> Self {
        MessageID(s)
    }
}

impl<'a> From<&'a str> for MessageID {
    fn from(s: &'a str) -> Self {
        MessageID(s.to_owned())
    }
}

impl<Z: TimeZone> From<DateTime<Z>> for MessageID {
    fn from(time: DateTime<Z>) -> Self {
        MessageID(format!(
            "{}.{:06}",
            time.timestamp(),
            time.timestamp_subsec_micros()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod message_id {
        use super::*;

        #[test]
        fn it_sorts_by_oldest_first() {
            let older = "1403051575.000407".into();
            let newer = "1403051575.000408".into();
            let newest = "1403051575.000409".into();

            let mut ids: Vec<&MessageID> = vec![&newest, &older, &newer];
            ids.sort();

            assert_eq!(&ids, &[&older, &newer, &newest]);
        }

        #[test]
        fn it_is_constructed_from_microsecond_timestamps() {
            use chrono::prelude::*;
            let expected_id = "1403051575.000407";
            let time = Utc.timestamp(1403051575, 407_000);
            let id: MessageID = time.clone().into();

            assert_eq!(&id.0, expected_id);
        }
    }
}
