mod buffer;
mod loading;
mod standard;
mod unsupported;

use std::cmp::{Ord, Ordering, PartialOrd};

use chrono::{DateTime, TimeZone};
use slack::api;
use failure::Error;

use canvas::Canvas;

pub use self::buffer::Buffer;
pub use self::standard::StandardMessage;
pub use self::loading::LoadingMessage;
pub use self::unsupported::UnsupportedMessage;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MessageID(String);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Message {
    Standard(StandardMessage),
    Unsupported(UnsupportedMessage),
}

fn unsupported(
    id: &Option<String>,
    from: &Option<String>,
    text: &Option<String>,
    subtype: &Option<String>,
) -> Result<Option<Message>, Error> {
    match UnsupportedMessage::from_slack_message(id, from, text, subtype) {
        Ok(message) => Ok(Some(message.into())),
        Err(error) => Err(error),
    }
}

impl Message {
    pub fn from_slack_message(msg: &api::Message) -> Result<Option<Self>, Error> {
        use self::api::Message as S;
        match *msg {
            S::Standard(ref msg) => Ok(Some(StandardMessage::from(msg).into())),
            S::BotMessage(ref msg) => unsupported(&msg.ts, &msg.username, &msg.text, &msg.subtype),
            S::ChannelArchive(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::ChannelJoin(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::ChannelLeave(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::ChannelName(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::ChannelPurpose(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::ChannelTopic(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::ChannelUnarchive(ref msg) => {
                unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype)
            }
            S::FileComment(ref msg) => unsupported(
                &msg.ts,
                &msg.comment.as_ref().and_then(|c| c.user.clone()),
                &msg.text,
                &msg.subtype,
            ),
            S::FileMention(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::FileShare(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::GroupArchive(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::GroupJoin(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::GroupLeave(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::GroupName(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::GroupPurpose(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::GroupTopic(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::GroupUnarchive(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::MeMessage(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::MessageChanged(ref msg) => unsupported(
                &msg.ts,
                &msg.message.as_ref().and_then(|m| m.user.clone()),
                &msg.message.as_ref().and_then(|c| c.text.clone()),
                &msg.subtype,
            ),
            S::MessageDeleted(ref msg) => unsupported(
                &msg.ts,
                &Some(String::from("Message deleted")),
                &Some(String::from("Message was deleted")),
                &msg.subtype,
            ),
            S::MessageReplied(ref msg) => unsupported(
                &msg.ts,
                &msg.message.as_ref().and_then(|m| m.user.clone()),
                &msg.message.as_ref().and_then(|c| c.text.clone()),
                &msg.subtype,
            ),
            S::PinnedItem(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
            S::ReplyBroadcast(ref msg) => unsupported(
                &msg.ts,
                &msg.user,
                &Some(String::from("Message got a broadcasted reply")),
                &msg.subtype,
            ),
            S::UnpinnedItem(ref msg) => unsupported(&msg.ts, &msg.user, &msg.text, &msg.subtype),
        }
    }

    pub fn id(&self) -> &MessageID {
        use self::Message::*;
        match *self {
            Standard(ref msg) => msg.id(),
            Unsupported(ref msg) => msg.id(),
        }
    }

    pub fn render_as_canvas(&self, width: u16) -> Canvas {
        use self::Message::*;
        match *self {
            Standard(ref msg) => msg.render_as_canvas(width),
            Unsupported(ref msg) => msg.render_as_canvas(width),
        }
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

impl From<UnsupportedMessage> for Message {
    fn from(message: UnsupportedMessage) -> Message {
        Message::Unsupported(message)
    }
}

impl From<StandardMessage> for Message {
    fn from(message: StandardMessage) -> Message {
        Message::Standard(message)
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
