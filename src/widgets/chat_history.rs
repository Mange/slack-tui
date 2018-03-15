use tui::style::Style;
use tui::widgets::Widget;
use tui::layout::Rect;
use tui::buffer::Buffer;

use widgets::Scrollbar;

const EMPTY_CANVAS: &'static [String] = &[];

pub struct ChatHistory<'a> {
    scroll: usize,
    canvas: &'a [String],
}

impl<'a> Default for ChatHistory<'a> {
    fn default() -> ChatHistory<'a> {
        ChatHistory {
            scroll: 0,
            canvas: &EMPTY_CANVAS,
        }
    }
}

impl<'a> ChatHistory<'a> {
    pub fn canvas(&mut self, canvas: &'a Vec<String>) -> &mut ChatHistory<'a> {
        if canvas.len() <= self.scroll {
            self.scroll = canvas.len();
        }
        self.canvas = canvas;
        self
    }

    pub fn scroll(&mut self, position: usize) -> &mut ChatHistory<'a> {
        self.scroll = position;
        self
    }
}

impl<'a> Widget for ChatHistory<'a> {
    fn draw(&mut self, area: &Rect, buf: &mut Buffer) {
        if area.width <= 1 {
            return;
        }
        // Render canvas backwards by "scrolling up" to the nth last line, rendering it at the
        // bottom of the rect and then moving upwards until we are outside of the rect.
        // NOTE: scroll will always be <= canvas.len()
        let last_row_index = self.canvas.len() - self.scroll;
        let width = area.width - 1; // Leave 1 space for scrollbar

        let mut rows = 0;
        let mut iterator = self.canvas[0..last_row_index].iter();

        while let Some(line) = iterator.next_back() {
            if area.bottom() - rows < area.top() {
                break;
            }

            assert!(line.len() <= width as usize);
            assert!(!line.contains('\n'));
            buf.set_string(area.left(), area.bottom() - rows, line, &Style::default());
            rows += 1;
        }

        // Draw scrollbar.
        if area.width > 10 {
            let scrollbar_bottom = self.canvas.len() - self.scroll;
            let scrollbar_top = scrollbar_bottom.saturating_sub(area.height as usize);
            assert!(scrollbar_top <= scrollbar_bottom);

            let scrollbar_area = Rect::new(area.right() - 1, area.top(), 1, area.height);
            Scrollbar::default()
                .set_total(self.canvas.len())
                .set_shown_range(scrollbar_top..scrollbar_bottom)
                .draw(&scrollbar_area, buf)
        }
    }
}
