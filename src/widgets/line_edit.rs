use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::*;
use tui::widgets::Widget;

pub struct LineEdit<'a> {
    text: &'a str,
    cursor_pos: usize,
    style: Style,
}

impl<'a> Default for LineEdit<'a> {
    fn default() -> Self {
        LineEdit {
            text: "",
            cursor_pos: 0,
            style: Style::default(),
        }
    }
}

impl<'a> LineEdit<'a> {
    pub fn text(&mut self, text: &'a str) -> &mut Self {
        self.text = text;
        self
    }

    pub fn cursor_pos(&mut self, pos: usize) -> &mut Self {
        self.cursor_pos = pos;
        self
    }

    pub fn style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for LineEdit<'a> {
    fn draw(&mut self, area: &Rect, buf: &mut Buffer) {
        // Leave one extra cell for cursor
        let offset = self
            .text
            .len()
            .saturating_sub(area.width as usize)
            .saturating_sub(1)
            .min(self.cursor_pos); // Keep cursor inside viewport

        let text_width = (self.text.len() - offset).min(area.width as usize);

        // Pick the right side of the text, offset by the offset specified.
        // Left pad with spaces so entire input box is rendered.
        let drawn_text = format!(
            "{:<1$}",
            &self.text[offset..offset + text_width],
            area.width as usize
        );

        let cursor_style = Style::default().fg(self.style.bg).bg(self.style.fg);

        for (i, chr) in drawn_text.chars().enumerate() {
            let style = if self.cursor_pos >= offset && i == self.cursor_pos - offset {
                cursor_style
            } else {
                self.style
            };

            let buf_index = buf.index_of(area.x + i as u16, area.y);
            buf.content
                .get_mut(buf_index)
                .unwrap()
                .set_char(chr)
                .set_style(style);
        }
    }
}
