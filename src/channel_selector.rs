#[derive(Debug)]
pub struct ChannelSelector {
    text: String,
    cursor_pos: usize,
}

impl ChannelSelector {
    pub fn new() -> Self {
        ChannelSelector {
            text: String::new(),
            cursor_pos: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor_pos
    }

    pub fn reset(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
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
}
