use std::cmp::{Eq, Ord, Ordering, PartialEq};

use tui::style::*;
use tui::widgets::Widget;
use tui::layout::Rect;
use tui::buffer::Buffer;

use chat::{self, ChannelID};

pub struct ChannelList<'a> {
    channels: &'a chat::ChannelList,
    selected_id: Option<&'a ChannelID>,
}

struct ChannelEntry<'a> {
    id: &'a ChannelID,
    name: &'a str,
    has_unreads: bool,
    is_selected: bool,
}

impl<'a> ChannelList<'a> {
    pub fn new(channels: &'a chat::ChannelList, selected_id: Option<&'a ChannelID>) -> Self {
        ChannelList {
            channels,
            selected_id,
        }
    }
}

impl<'a> Widget for ChannelList<'a> {
    fn draw(&mut self, area: &Rect, buf: &mut Buffer) {
        if area.width < 3 {
            return;
        }

        let mut starred = vec![];
        let mut others = vec![];

        for (_, channel) in self.channels.iter() {
            let entry = ChannelEntry {
                id: channel.id(),
                name: channel.name(),
                has_unreads: false,
                is_selected: self.selected_id
                    .map(|id| id == channel.id())
                    .unwrap_or(false),
            };
            if channel.is_starred() {
                starred.push(entry);
            } else if channel.is_member() || entry.is_selected {
                others.push(entry);
            }
        }

        starred.sort();
        others.sort();

        let mut y = area.top();
        y = draw_entries("Starred", &starred, area, buf, y);
        draw_entries("Channels", &others, area, buf, y);
    }
}

fn draw_entries(
    title: &str,
    entries: &Vec<ChannelEntry>,
    area: &Rect,
    buf: &mut Buffer,
    y: u16,
) -> u16 {
    let mut y = y;
    if y < area.bottom() {
        buf.set_string(
            area.left(),
            y,
            &format!("{:1$}", title, area.width as usize),
            &Style::default().modifier(Modifier::Bold).bg(Color::Gray),
        );
        y += 1;
    }

    // Subtracted one character for icon
    let name_width = area.width as usize - 1;

    for entry in entries {
        if y >= area.bottom() {
            return y;
        }

        let mut style = Style::default();

        if entry.has_unreads {
            style.modifier(Modifier::Bold);
        }

        if entry.is_selected {
            style = style.bg(Color::White).fg(Color::Black);
        }

        buf.set_stringn(area.x, y, "#", 1, &style);
        buf.set_stringn(
            area.x + 1,
            y,
            &format!("{:1$}", entry.name, name_width),
            name_width,
            &style,
        );
        y += 1;
    }

    y
}

impl<'a> PartialEq for ChannelEntry<'a> {
    fn eq(&self, other: &ChannelEntry) -> bool {
        self.id.eq(other.id)
    }
}

impl<'a> Eq for ChannelEntry<'a> {}

impl<'a> PartialOrd for ChannelEntry<'a> {
    fn partial_cmp(&self, rhs: &ChannelEntry) -> Option<Ordering> {
        (self.has_unreads, self.name).partial_cmp(&(rhs.has_unreads, &rhs.name))
    }
}

impl<'a> Ord for ChannelEntry<'a> {
    fn cmp(&self, rhs: &ChannelEntry) -> Ordering {
        (self.has_unreads, self.name).cmp(&(rhs.has_unreads, &rhs.name))
    }
}
