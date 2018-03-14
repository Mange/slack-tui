use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::cmp::{Ord, Ordering, PartialOrd};

#[derive(Clone, Debug)]
pub struct Message {
    pub timestamp: String,
    pub from: &'static str,
    pub body: &'static str,
}

impl Hash for Message {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.timestamp.hash(state)
    }
}

impl PartialEq for Message {
    fn eq(&self, rhs: &Message) -> bool {
        self.timestamp.eq(&rhs.timestamp)
    }
}

impl Eq for Message {}

impl PartialOrd for Message {
    fn partial_cmp(&self, rhs: &Message) -> Option<Ordering> {
        self.timestamp.partial_cmp(&rhs.timestamp)
    }
}

impl Ord for Message {
    fn cmp(&self, rhs: &Message) -> Ordering {
        self.timestamp.cmp(&rhs.timestamp)
    }
}

impl Message {
    fn render_as_string(&self) -> String {
        format!(
            "{{fg=cyan;mod=bold {from}}}\n{text}\n",
            from = self.from,
            text = self.body
        )
    }
}

pub struct MessageBuffer {
    messages: BTreeSet<Message>,
}

impl Into<MessageBuffer> for Vec<Message> {
    fn into(self) -> MessageBuffer {
        MessageBuffer {
            messages: self.into_iter().collect(),
        }
    }
}

impl MessageBuffer {
    pub fn render_into_canvas(&self, width: usize) -> String {
        // This is extremely inefficient and lazy and stupid.
        // But it should be enough to experiment with the UI.
        //
        // Problems:
        //  1. Strings are not word-wrapped, but instead wrapped at characters.
        //  2. Full message is rendered first, *then* it is wrapped. The message renderer should
        //     wrap immediately instead.
        //  3. Allocating intermediary `Vec`s when we could append to a String right away.
        //  4. The Canvas should know about how many lines there are in total to help with
        //     scrolling efforts.
        //  5. Only when size changed since last time should we have to reflow all the messages.
        //     Try to figure outt a buffering scheme where new messages are added at the end and
        //     only cause a re-render at the end. Maybe. Something like that.
        self.messages
            .iter()
            .map(|message| message.render_as_string())
            .flat_map(|string| {
                let mut chars = string.chars();
                (0..)
                    .map(|_| chars.by_ref().take(width).collect::<String>())
                    .take_while(|s| !s.is_empty())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
