use tui::style::Style;
use tui::widgets::{Block, Paragraph, Widget};
use tui::layout::Rect;
use tui::buffer::Buffer;

pub struct ChatHistory<'a> {
    block: Option<Block<'a>>,
    style: Style,
    scroll: usize,
    text: &'a str,
}

impl<'a> Default for ChatHistory<'a> {
    fn default() -> ChatHistory<'a> {
        ChatHistory {
            block: None,
            style: Style::default(),
            scroll: 0,
            text: "",
        }
    }
}

impl<'a> ChatHistory<'a> {
    pub fn text(&mut self, text: &'a str) -> &mut ChatHistory<'a> {
        self.text = text;
        self
    }

    pub fn block<B>(&mut self, block: B) -> &mut ChatHistory<'a>
    where
        B: Into<Option<Block<'a>>>,
    {
        self.block = block.into();
        self
    }

    pub fn scroll(&mut self, position: usize) -> &mut ChatHistory<'a> {
        self.scroll = position;
        self
    }
}

impl<'a> Widget for ChatHistory<'a> {
    fn draw(&mut self, area: &Rect, buf: &mut Buffer) {
        let mut paragraph = Paragraph::default();

        paragraph
            .style(self.style)
            .text(&self.text)
            .wrap(true)
            .scroll(self.scroll as u16);

        if let Some(block) = self.block.clone() {
            paragraph.block(block).draw(area, buf);
        } else {
            paragraph.draw(area, buf);
        }
    }
}
