use std::fmt;
use std::fmt::Debug;

use tui::buffer::{Buffer, Cell};
use tui::style::Style;

#[derive(Clone)]
pub struct Canvas {
    width: u16,
    cells: Vec<Cell>,
}

impl Canvas {
    pub fn new(width: u16) -> Canvas {
        Canvas {
            width: width,
            cells: Vec::new(),
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.cells.len() as u16 / self.width
    }

    pub fn get_pos(&self, x: u16, y: u16) -> Option<&Cell> {
        self.get_index(x, y).and_then(|i| self.cells.get(i))
    }

    pub fn add_string_wrapped(&mut self, string: &str, style: Style) {
        let mut chars_on_line = 0;
        for chr in string.chars() {
            match chr {
                '\n' => {
                    let remaining = self.width - chars_on_line;
                    for _ in 0..remaining {
                        self.add_char(' ', style);
                    }
                    chars_on_line = 0;
                }
                '\r' => {}
                // TODO: Treat \t and \b differently.
                _ => {
                    self.add_char(chr, style);
                    chars_on_line = (chars_on_line + 1) % self.width;
                }
            }
        }
    }

    pub fn add_string_truncated(&mut self, string: &str, style: Style) {
        let mut chars_on_line = 0;
        for chr in string.chars() {
            match chr {
                '\n' => {
                    let remaining = self.width - chars_on_line;
                    for _ in 0..remaining {
                        self.add_char(' ', style);
                    }
                    chars_on_line = 0;
                }
                '\r' => {}
                // TODO: Treat \t and \b differently.
                _ => {
                    if chars_on_line < self.width {
                        self.add_char(chr, style);
                    }
                    chars_on_line += 1;
                }
            }
        }
    }

    pub fn render_viewport(&self, viewport_options: ViewportOptions) -> Buffer {
        use tui::layout::Rect;
        let rect = Rect::new(
            viewport_options.rect_x,
            viewport_options.rect_y,
            self.width,
            viewport_options.height,
        );
        let mut cells: Vec<Cell> = self.cells
            .iter()
            .skip(viewport_options.offset as usize * self.width as usize)
            .take(viewport_options.height as usize * self.width as usize)
            .map(Cell::clone)
            .collect();
        cells.resize(rect.area() as usize, Cell::default());

        Buffer {
            area: rect,
            content: cells,
        }
    }

    #[cfg(test)]
    pub fn render_to_string<'a>(&self, eol_marker: Option<&'a str>) -> String {
        let mut s = String::with_capacity(self.cells.len() + self.height() as usize);
        let mut total_chars = 0;
        let width = self.width as usize;

        for (i, chr) in self.cells
            .iter()
            .flat_map(|c| c.symbol.chars().next())
            .enumerate()
        {
            if i > 0 && i % width == 0 {
                if let &Some(marker) = &eol_marker {
                    s.push_str(marker);
                }
                s.push('\n');
            }
            s.push(chr);
            total_chars += 1;
        }

        // File the last line with spaces. If a full line is needed, then we don't need to add
        // anything.
        let characters_left = width - (total_chars % width);
        if characters_left < width {
            for _ in 0..characters_left {
                s.push(' ');
            }
        }
        if let &Some(marker) = &eol_marker {
            s.push_str(marker);
        }
        s
    }

    fn get_index(&self, x: u16, y: u16) -> Option<usize> {
        if x < self.width {
            let index = y as usize * self.width as usize + x as usize;
            if index < self.cells.len() {
                return Some(index);
            }
        }
        None
    }

    fn add_char(&mut self, chr: char, style: Style) {
        let mut cell = Cell::default();
        cell.set_char(chr).set_style(style);
        self.cells.push(cell);
    }
}

impl ::std::ops::AddAssign<Canvas> for Canvas {
    fn add_assign(&mut self, rhs: Canvas) {
        assert!(
            self.width == rhs.width,
            "Tried to add_assign (+=) two canvases with different width! LHS={}, RHS={}",
            self.width,
            rhs.width
        );
        let mut rhs = rhs;
        self.cells.append(&mut rhs.cells);
    }
}

impl Debug for Canvas {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::fmt::Write;

        write!(
            f,
            "Canvas (width={}, height={})\n",
            self.width(),
            self.height()
        )?;
        for (i, cell) in self.cells.iter().enumerate() {
            if i > 0 && i as u16 % self.width == 0 {
                f.write_char('\n')?;
            }
            f.write_char(cell.symbol.chars().next().unwrap())?;
        }
        Ok(())
    }
}

#[derive(Default, Debug, PartialEq, Clone, Copy, Eq)]
pub struct ViewportOptions {
    height: u16,
    offset: u16,
    rect_x: u16,
    rect_y: u16,
}

#[allow(dead_code)]
impl ViewportOptions {
    pub fn new(height: u16) -> Self {
        ViewportOptions {
            height,
            ..Default::default()
        }
    }

    pub fn with_height(mut self, height: u16) -> Self {
        self.height = height;
        self
    }

    pub fn with_offset(mut self, offset: u16) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_rect_position(mut self, x: u16, y: u16) -> Self {
        self.rect_x = x;
        self.rect_y = y;
        self
    }
}

#[cfg(test)]
mod tests {
    use tui::layout::Rect;
    use tui::style::Color;
    use super::*;

    fn cell(chr: char, style: Style) -> Cell {
        let mut cell = Cell::default();
        cell.set_char(chr).set_style(style);
        cell
    }

    macro_rules! assert_eq_buffer {
        ($a:ident, $b:ident) => {
            match (&$a, &$b) {
                (ref a, ref b) => {
                    use render_buffer;
                    assert_eq!(a.area, b.area, "Expected buffer areas to be equal");
                    assert_eq!(a.content.len(), b.content.len(), "Expected buffer content sizes to be equal");
                    if a.content != b.content {
                        panic!(
                            "Expected cells to be equal:\n{}\n\n-=-=-=-=-=-=-=-=-\n\n{}",
                            render_buffer(a),
                            render_buffer(b)
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn it_writes_characters_to_grow() {
        let red = Style::default().bg(Color::Red);
        let green = Style::default().bg(Color::Green);

        let mut canvas = Canvas::new(3);
        assert_eq!(canvas.height(), 0);

        canvas.add_string_wrapped("Foobar", red);
        assert_eq!(canvas.height(), 2);
        assert_eq!(canvas.cells.len(), 6);

        canvas.add_string_truncated("Goodbye!", green);
        assert_eq!(canvas.height(), 3);
        assert_eq!(canvas.cells.len(), 9);

        // Adding a newline means "fill rest of line with spaces"
        canvas.add_string_truncated("\n!\n", green);
        assert_eq!(canvas.height(), 5);
        assert_eq!(canvas.cells.len(), 9 + 3 + 1 + 2);

        assert_eq!(
            format!("{:?}", canvas),
            "Canvas (width=3, height=5)\nFoo\nbar\nGoo\n   \n!  ",
        );

        assert_eq!(canvas.get_pos(0, 0), Some(&cell('F', red)));

        assert_eq!(canvas.get_pos(1, 0), Some(&cell('o', red)));
        assert_eq!(canvas.get_pos(0, 1), Some(&cell('b', red)));
        assert_eq!(canvas.get_pos(0, 2), Some(&cell('G', green)));

        assert_eq!(canvas.get_pos(3, 0), None);
        assert_eq!(canvas.get_pos(0, 8), None);
        assert_eq!(canvas.get_pos(3, 8), None);
    }

    #[test]
    fn it_appends_canvases() {
        let mut top = Canvas::new(3);
        let mut bottom = Canvas::new(3);
        let style = Style::default();

        top.add_string_truncated("Foo", style);
        bottom.add_string_truncated("bar", style);

        top += bottom;
        assert_eq!(top.width(), 3);
        assert_eq!(top.height(), 2);
    }

    #[test]
    #[should_panic]
    fn it_panics_when_adding_different_widths() {
        let mut top = Canvas::new(3);
        let bottom = Canvas::new(5);
        top += bottom;
    }

    #[test]
    fn it_renders_into_tui_buffer_viewports() {
        let red = Style::default().fg(Color::Red);
        let green = Style::default().fg(Color::Green);

        let mut canvas = Canvas::new(6);
        canvas.add_string_wrapped("Foobar------", green);
        canvas.add_string_wrapped("Here I am", red);

        // Render top
        let render_options = ViewportOptions::new(2);
        let expected_rect = Rect::new(0, 0, 6, 2);
        let mut expected_buffer = Buffer::empty(expected_rect);
        expected_buffer.set_string(0, 0, "Foobar", &green);
        expected_buffer.set_string(0, 1, "------", &green);

        let top_viewport = canvas.render_viewport(render_options);
        assert_eq_buffer!(top_viewport, expected_buffer);

        // Render middle
        let render_options = ViewportOptions::new(2).with_offset(1);
        let expected_rect = Rect::new(0, 0, 6, 2);
        let mut expected_buffer = Buffer::empty(expected_rect);
        expected_buffer.set_string(0, 0, "------", &green);
        expected_buffer.set_string(0, 1, "Here I", &red);

        let middle_viewport = canvas.render_viewport(render_options);
        assert_eq_buffer!(middle_viewport, expected_buffer);

        // Render bottom part
        let render_options = ViewportOptions::new(3).with_offset(1);
        let expected_rect = Rect::new(0, 0, 6, 3);
        let mut expected_buffer = Buffer::empty(expected_rect);
        expected_buffer.set_string(0, 0, "------", &green);
        expected_buffer.set_string(0, 1, "Here I", &red);
        expected_buffer.set_string(0, 2, " am", &red);

        let bottom_viewport = canvas.render_viewport(render_options);
        assert_eq_buffer!(bottom_viewport, expected_buffer);

        // Rendering out-of-bounds is filled with spaces
        let render_options = ViewportOptions::new(5).with_offset(1);
        let expected_rect = Rect::new(0, 0, 6, 5);
        let mut expected_buffer = Buffer::empty(expected_rect);
        expected_buffer.set_string(0, 0, "------", &green);
        expected_buffer.set_string(0, 1, "Here I", &red);
        expected_buffer.set_string(0, 2, " am", &red);
        expected_buffer.set_string(0, 3, "", &Style::default()); // Only for
        expected_buffer.set_string(0, 4, "", &Style::default()); // illustration

        let out_of_bounds_viewport = canvas.render_viewport(render_options);
        assert_eq_buffer!(out_of_bounds_viewport, expected_buffer);

        // Rendering with an Rect x/y offset
        let render_options = ViewportOptions::new(2)
            .with_offset(1)
            .with_rect_position(15, 62);
        let expected_rect = Rect::new(15, 62, 6, 2);
        let mut expected_buffer = Buffer::empty(expected_rect);
        expected_buffer.set_string(15, 62, "------", &green);
        expected_buffer.set_string(15, 63, "Here I", &red);

        let rect_offset_viewport = canvas.render_viewport(render_options);
        assert_eq_buffer!(rect_offset_viewport, expected_buffer);
    }

    #[test]
    fn it_renders_to_string() {
        let mut canvas = Canvas::new(3);
        let style = Style::default();

        canvas.add_string_wrapped("Foobar yay!", style);

        assert_eq!(&canvas.render_to_string(None), "Foo\nbar\n ya\ny! ");
        assert_eq!(
            &canvas.render_to_string(Some("|")),
            "Foo|\nbar|\n ya|\ny! |"
        );

        let mut canvas = Canvas::new(10);
        canvas.add_string_wrapped("\n12345\n1234567890     67890\n", style);
        assert_eq!(
            &canvas.render_to_string(Some("|")),
            "          |
12345     |
1234567890|
     67890|
          |"
        );
    }
}
