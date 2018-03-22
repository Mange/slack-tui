extern crate slack;
use slack::api::Channel as SlackChannel;

use std::hash::{Hash, Hasher};
use std::collections::BTreeMap;
use std::iter::FromIterator;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChannelID(String);

#[derive(Debug, Clone)]
pub struct Channel {
    id: ChannelID,
    name: String,
    is_member: bool,
    is_starred: bool,
    has_unreads: bool,
}

#[derive(Debug, Clone)]
pub struct ChannelList {
    channels: BTreeMap<ChannelID, Channel>,
}

type Iter<'a> = ::std::collections::btree_map::Iter<'a, ChannelID, Channel>;

impl Channel {
    pub fn from_slack(channel: &SlackChannel) -> Option<Self> {
        let id = match channel.id {
            Some(ref id) => ChannelID::from(id),
            None => return None,
        };

        let name = match channel.name {
            Some(ref name) => name.clone(),
            None => return None,
        };

        Some(Channel {
            id,
            name,
            is_starred: false, // TODO. Needs to be read using Slack API stars.list
            has_unreads: channel.unread_count.unwrap_or(0) > 0,
            is_member: channel.is_member.unwrap_or(false),
        })
    }

    pub fn id(&self) -> &ChannelID {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_member(&self) -> bool {
        self.is_member
    }

    pub fn is_starred(&self) -> bool {
        // TODO: This is just a quick ugly hack to test the "starred" feature
        self.is_starred || self.name == "team-core" || self.name == "development"
    }

    pub fn has_unreads(&self) -> bool {
        self.has_unreads
    }
}

impl ChannelList {
    pub fn new() -> Self {
        ChannelList {
            channels: BTreeMap::new(),
        }
    }

    pub fn iter(&self) -> Iter {
        self.channels.iter()
    }
}

impl<'a> From<&'a str> for ChannelID {
    fn from(s: &'a str) -> Self {
        ChannelID(String::from(s))
    }
}

impl<'a> From<&'a String> for ChannelID {
    fn from(s: &'a String) -> Self {
        ChannelID(s.clone())
    }
}

impl From<String> for ChannelID {
    fn from(s: String) -> Self {
        ChannelID(s)
    }
}

impl Hash for Channel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl PartialEq for Channel {
    fn eq(&self, rhs: &Channel) -> bool {
        self.id.eq(&rhs.id)
    }
}

impl Eq for Channel {}

impl FromIterator<Channel> for ChannelList {
    fn from_iter<I: IntoIterator<Item = Channel>>(iter: I) -> Self {
        ChannelList {
            channels: iter.into_iter().map(|c| (c.id.clone(), c)).collect(),
        }
    }
}
