use tui::widgets::Widget;
use tui::layout::Rect;
use tui::buffer::Buffer;

use widgets::Scrollbar;
use models::Canvas;
use models::canvas::ViewportOptions;

pub struct ChatHistory<'a> {
    scroll: usize,
    canvas: &'a Canvas,
}

impl<'a> ChatHistory<'a> {
    pub fn with_canvas(canvas: &'a Canvas) -> ChatHistory<'a> {
        ChatHistory { canvas, scroll: 0 }
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

        let canvas_height = self.canvas.height() as usize;
        let viewport_height = area.height as usize;
        let scroll_from_top = (canvas_height - self.scroll).saturating_sub(viewport_height);

        let text_width = self.canvas.width();

        // Canvas should fit, with a single scrollbar
        assert!(area.width == text_width + 1);

        // Draw viewport
        let viewport = self.canvas.render_viewport(
            ViewportOptions::new(area.height)
                .with_offset((scroll_from_top) as u16)
                .with_rect_position(area.x, area.y),
        );
        for i in 0..viewport.content.len() {
            let (x, y) = viewport.pos_of(i);
            let global_index = buf.index_of(x, y);
            buf.content[global_index] = viewport.content[i].clone();
        }

        // Draw scrollbar.
        if area.width > 10 {
            let scrollbar_bottom = canvas_height - self.scroll;
            let scrollbar_top = scrollbar_bottom.saturating_sub(area.height as usize);
            assert!(scrollbar_top <= scrollbar_bottom);

            let scrollbar_area = Rect::new(area.right() - 1, area.top(), 1, area.height);
            Scrollbar::default()
                .set_total(canvas_height)
                .set_shown_range(scrollbar_top..scrollbar_bottom)
                .draw(&scrollbar_area, buf)
        }
    }
}
