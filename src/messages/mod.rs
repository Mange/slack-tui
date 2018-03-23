mod buffer;
mod loading;
mod standard;

use std::cmp::{Ord, Ordering, PartialOrd};

use chrono::{DateTime, TimeZone};
use slack::api;
use failure::Error;

use canvas::Canvas;

pub use self::buffer::Buffer;
pub use self::standard::StandardMessage;
pub use self::loading::LoadingMessage;

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct MessageID(String);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Message {
    // TODO: This is a purely render-concern; don't have it in the buffer anymore!
    Loading(LoadingMessage),

    Standard(StandardMessage),
}

impl Message {
    pub fn from_slack_message(msg: &api::Message) -> Result<Option<Self>, Error> {
        match *msg {
            api::Message::Standard(ref msg) => Ok(Some(StandardMessage::from(msg).into())),
            _ => Ok(None),
        }
    }

    pub fn id(&self) -> &MessageID {
        use self::Message::*;
        match *self {
            Standard(ref msg) => msg.id(),
            Loading(ref msg) => msg.id(),
        }
    }

    pub fn render_as_canvas(&self, width: u16) -> Canvas {
        use self::Message::*;
        match *self {
            Standard(ref msg) => msg.render_as_canvas(width),
            Loading(ref msg) => msg.render_as_canvas(width),
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

impl From<LoadingMessage> for Message {
    fn from(message: LoadingMessage) -> Message {
        Message::Loading(message)
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
