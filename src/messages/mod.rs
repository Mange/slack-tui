mod buffer;
mod loading;
mod standard;

use std::cmp::{Ord, Ordering, PartialOrd};

use canvas::Canvas;

pub use self::buffer::Buffer;
pub use self::standard::StandardMessage;
pub use self::loading::LoadingMessage;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Message {
    Standard(StandardMessage),
    Loading(LoadingMessage),
}

impl Message {
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
        use self::Message::*;
        match (self, rhs) {
            (&Loading(ref a), &Loading(ref b)) => a.partial_cmp(b),
            (&Standard(ref a), &Standard(ref b)) => a.partial_cmp(b),
            (&Loading(_), _) => Some(Ordering::Less),
            (_, &Loading(_)) => Some(Ordering::Greater),
        }
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
