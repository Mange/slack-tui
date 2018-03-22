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
}

#[derive(Debug, Clone)]
pub struct ChannelList {
    channels: BTreeMap<ChannelID, Channel>,
}

#[derive(Debug, Clone)]
pub struct ChannelEntry<'list> {
    pub id: &'list ChannelID,
    pub name: &'list str,
}

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

        Some(Channel { id, name })
    }
}

impl ChannelList {
    pub fn new() -> Self {
        ChannelList {
            channels: BTreeMap::new(),
        }
    }

    pub fn entries(&self) -> Vec<ChannelEntry> {
        self.channels
            .iter()
            .map(|(_, channel)| ChannelEntry {
                id: &channel.id,
                name: &channel.name,
            })
            .collect()
    }

    pub fn entries_with_selected(
        &self,
        selected_id: Option<&ChannelID>,
    ) -> (usize, Vec<ChannelEntry>) {
        let entries = self.entries();
        let index = selected_id.and_then(|id| entries.iter().position(|c| c.id == id));
        (index.unwrap_or(0), entries)
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
