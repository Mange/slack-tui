use std::cmp::{Ord, Ordering, PartialOrd};
use models::{Channel, ChannelID, ChannelList};

#[derive(Debug)]
pub struct ChannelSelector {
    text: String,
    cursor_pos: usize,
    selected_index: usize,
}

#[derive(Debug)]
pub struct ChannelMatch<'a> {
    pub score: f32,
    pub channel: &'a Channel,
}

impl ChannelSelector {
    pub fn new() -> Self {
        ChannelSelector {
            text: String::new(),
            cursor_pos: 0,
            selected_index: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    pub fn selected_index(&self, max: usize) -> usize {
        self.selected_index.min(max)
    }

    pub fn reset(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
        self.selected_index = 0;
    }

    pub fn add_character(&mut self, chr: char) {
        self.text.insert(self.cursor_pos, chr);
        self.cursor_pos += 1;
    }

    pub fn delete_character(&mut self) {
        if self.cursor_pos > 0 {
            if self.cursor_pos < self.text.len() {
                self.text.remove(self.cursor_pos - 1);
            } else {
                self.text.pop();
            }
            self.cursor_pos -= 1;
        }
    }

    pub fn delete_word(&mut self) {
        if self.cursor_pos > 0 {
            let index = self.text[..self.cursor_pos - 1].rfind(' ').unwrap_or(0);
            let chars_removed = self.cursor_pos - index;
            let rest = self.text.split_off(index);
            self.text.push_str(&rest[self.cursor_pos - index..]);
            self.cursor_pos -= chars_removed;
        }
    }

    pub fn move_cursor_left(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_sub(1);
    }

    pub fn move_cursor_right(&mut self) {
        self.cursor_pos = self.cursor_pos.saturating_add(1).min(self.text.len());
    }

    pub fn move_to_end(&mut self) {
        self.cursor_pos = self.text().len();
    }

    pub fn move_to_beginning(&mut self) {
        self.cursor_pos = 0;
    }

    pub fn select_next_match(&mut self) {
        self.selected_index = self.selected_index.saturating_add(1);
    }

    pub fn select_previous_match(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
    }

    pub fn top_matches<'channels>(
        &self,
        channels: &'channels ChannelList,
        max: usize,
    ) -> Vec<ChannelMatch<'channels>> {
        let mut matches: Vec<_> = channels
            .iter()
            .map(|(_, channel)| ChannelMatch {
                score: calculate_score(&channel, &self.text),
                channel,
            })
            .filter(|m| m.score > 0.0)
            .collect();
        matches.sort();
        if matches.len() > max {
            matches.split_off(max);
        }
        matches
    }

    pub fn select(&self, channels: &ChannelList) -> Option<ChannelID> {
        self.top_matches(channels, self.selected_index + 1)
            .get(self.selected_index)
            .map(|channel_match| channel_match.channel.id().clone())
    }
}

fn calculate_score(channel: &Channel, text: &str) -> f32 {
    // Find first character from text in channel name, then search for second text character to the
    // right, and so on. If a character is not found, return a negative score.
    let mut name = channel.name();
    for chr in text.chars() {
        match name.find(chr) {
            Some(pos) => {
                name = &name[pos..];
            }
            None => return -1.0,
        }
    }
    1.0
}

impl<'a> PartialEq for ChannelMatch<'a> {
    fn eq(&self, rhs: &ChannelMatch) -> bool {
        self.score == rhs.score
    }
}

impl<'a> Eq for ChannelMatch<'a> {}

impl<'a> PartialOrd for ChannelMatch<'a> {
    fn partial_cmp(&self, rhs: &ChannelMatch) -> Option<Ordering> {
        let score_order = self.score.partial_cmp(&rhs.score);
        if let Some(Ordering::Equal) = score_order {
            self.channel.name().partial_cmp(rhs.channel.name())
        } else {
            score_order
        }
    }
}

impl<'a> Ord for ChannelMatch<'a> {
    fn cmp(&self, rhs: &ChannelMatch) -> Ordering {
        self.score
            .partial_cmp(&rhs.score)
            .unwrap_or(Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_adds_characters() {
        let mut channel_selector = ChannelSelector::new();
        channel_selector.add_character('f');
        channel_selector.add_character('o');
        channel_selector.add_character('o');

        assert_eq!(channel_selector.text(), "foo");
    }

    #[test]
    fn it_deletes_character() {
        let mut channel_selector = ChannelSelector::new();
        channel_selector.add_character('f');
        channel_selector.add_character('o');
        channel_selector.add_character('o');
        channel_selector.delete_character();

        assert_eq!(channel_selector.text(), "fo");
    }

    #[test]
    fn it_deletes_word() {
        let mut channel_selector = ChannelSelector::new();
        channel_selector.add_character('f');
        channel_selector.add_character('o');
        channel_selector.add_character('o');
        channel_selector.add_character(' ');
        channel_selector.add_character('b');
        channel_selector.add_character('a');
        channel_selector.add_character('r');
        channel_selector.delete_word();

        assert_eq!(channel_selector.text(), "foo");
    }

    #[test]
    fn it_deletes_character_at_cursor() {
        let mut channel_selector = ChannelSelector::new();
        channel_selector.add_character('f');
        channel_selector.add_character('o');
        channel_selector.add_character('o');
        channel_selector.move_cursor_left();
        channel_selector.move_cursor_left();
        channel_selector.delete_character();
        channel_selector.add_character('b');
        channel_selector.move_to_end();
        channel_selector.add_character('!');

        assert_eq!(channel_selector.text(), "boo!");
    }

    #[test]
    fn it_deletes_word_from_cursor() {
        let mut channel_selector = ChannelSelector::new();
        channel_selector.add_character('f');
        channel_selector.add_character('o');
        channel_selector.add_character('o');
        channel_selector.add_character(' ');
        channel_selector.add_character('b');
        channel_selector.move_cursor_left();
        channel_selector.delete_word();
        channel_selector.add_character('a');

        assert_eq!(channel_selector.text(), "ab");
    }

    #[test]
    fn it_moves_to_beginning_and_end() {
        let mut channel_selector = ChannelSelector::new();
        channel_selector.add_character('o');
        channel_selector.move_to_beginning();
        channel_selector.add_character('f');
        channel_selector.move_to_end();
        channel_selector.add_character('o');

        assert_eq!(channel_selector.text(), "foo");
    }

    #[test]
    fn it_does_not_panic_when_deleting_at_beginning() {
        let mut channel_selector = ChannelSelector::new();
        channel_selector.delete_character();
        channel_selector.delete_word();

        channel_selector.add_character('a');
        channel_selector.move_to_beginning();
        channel_selector.delete_character();
        channel_selector.delete_word();

        assert_eq!(channel_selector.text(), "a");
    }

    #[test]
    fn it_matches_channels_containing_text() {
        let mut channel_list = ChannelList::new();
        channel_list.add_channel(Channel::fixture("1", "foobar"));
        channel_list.add_channel(Channel::fixture("2", "bar-jumping"));
        channel_list.add_channel(Channel::fixture("3", "foosball"));
        channel_list.add_channel(Channel::fixture("4", "unrelated"));

        let mut channel_selector = ChannelSelector::new();
        channel_selector.add_character('f');
        channel_selector.add_character('o');
        channel_selector.add_character('o');

        let three_top_channels: Vec<&str> = channel_selector
            .top_matches(&channel_list, 3)
            .iter()
            .map(|m| m.channel.name())
            .collect();

        let one_top_channels: Vec<&str> = channel_selector
            .top_matches(&channel_list, 1)
            .iter()
            .map(|m| m.channel.name())
            .collect();

        assert_eq!(&three_top_channels, &["foobar", "foosball"]);
        assert_eq!(&one_top_channels, &["foobar"]);
    }
}
