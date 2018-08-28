use std::ops::Range;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Style;
use tui::widgets::Widget;

#[derive(Debug)]
pub struct Scrollbar {
    total: usize,
    shown_range: Range<usize>,
}

impl Default for Scrollbar {
    fn default() -> Scrollbar {
        Scrollbar {
            total: 0,
            shown_range: 0..0,
        }
    }
}

impl Scrollbar {
    pub fn set_total(&mut self, total: usize) -> &mut Scrollbar {
        self.total = total;
        if self.shown_range.end > total {
            let diff = self.shown_range.end - total;
            self.shown_range = (self.shown_range.start.saturating_sub(diff))..total;
        }
        self
    }

    pub fn set_shown_range(&mut self, shown: Range<usize>) -> &mut Scrollbar {
        if shown.end > self.total {
            panic!("Scrollbar shown range cannot be outside of scrollbar total!");
        }

        self.shown_range = shown;
        self
    }

    fn items_shown(&self) -> usize {
        if self.shown_range.start == self.shown_range.end {
            0
        } else {
            self.shown_range.end - self.shown_range.start
        }
    }

    fn above_height(&self, height: u16) -> u16 {
        (height as f64 * self.ratio_above()).floor() as u16
    }

    fn shown_height(&self, height: u16) -> u16 {
        // Base value on sizes around it so we always round the correct way
        height - self.below_height(height) - self.above_height(height)
    }

    fn below_height(&self, height: u16) -> u16 {
        (height as f64 * self.ratio_below()).floor() as u16
    }

    fn ratio_above(&self) -> f64 {
        self.shown_range.start as f64 / self.total as f64
    }

    fn ratio_shown(&self) -> f64 {
        self.items_shown() as f64 / self.total as f64
    }

    fn ratio_below(&self) -> f64 {
        // Due to float rounding errors ratio_above + ratio_shown can be >1.0
        (1.0 - self.ratio_above() - self.ratio_shown()).max(0.0)
    }
}

impl Widget for Scrollbar {
    fn draw(&mut self, area: &Rect, buf: &mut Buffer) {
        if self.total == 0 || self.shown_range.start == self.shown_range.end {
            return;
        }

        let above_height = self.above_height(area.height);
        let shown_height = self.shown_height(area.height);

        let background_style = Style::default().bg(Color::Black);
        let shown_style = Style::default().bg(Color::White);

        for y in 0..area.height {
            let projected_y = area.top() + y;

            let inside_scroll = y > above_height && y < above_height + shown_height;
            let style = if inside_scroll {
                &shown_style
            } else {
                &background_style
            };

            for x in 0..area.width {
                let projected_x = area.left() + x;

                buf.set_string(projected_x, projected_y, " ", style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_caps_range_to_total() {
        let mut scrollbar = Scrollbar::default();

        assert_eq!(scrollbar.total, 0);
        assert_eq!(scrollbar.shown_range, 0..0);

        scrollbar.set_total(100).set_shown_range(10..60);

        assert_eq!(scrollbar.total, 100);
        assert_eq!(scrollbar.shown_range, 10..60);

        scrollbar.set_total(50);

        assert_eq!(scrollbar.total, 50);
        assert_eq!(scrollbar.shown_range, 0..50);

        scrollbar.set_total(10);

        assert_eq!(scrollbar.total, 10);
        assert_eq!(scrollbar.shown_range, 0..10);

        scrollbar
            .set_total(100)
            .set_shown_range(50..90)
            .set_total(90);

        assert_eq!(scrollbar.total, 90);
        assert_eq!(scrollbar.shown_range, 50..90);
    }

    #[test]
    #[should_panic]
    fn it_panics_when_range_is_set_outside_of_total() {
        Scrollbar::default().set_total(10).set_shown_range(5..11);
    }

    #[test]
    fn it_allows_ranges_up_until_total() {
        let mut scrollbar = Scrollbar::default();
        scrollbar.set_total(10).set_shown_range(5..10);
        assert_eq!(scrollbar.total, 10);
        assert_eq!(scrollbar.shown_range, 5..10);
    }

    #[test]
    fn it_calculates_ratios() {
        let mut scrollbar = Scrollbar::default();
        scrollbar.set_total(100);

        scrollbar.set_shown_range(0..100);
        assert_eq!(scrollbar.items_shown(), 100);
        assert_eq!(scrollbar.ratio_above(), 0.0);
        assert_eq!(scrollbar.ratio_shown(), 1.0);
        assert_eq!(scrollbar.ratio_below(), 0.0);

        scrollbar.set_shown_range(0..50);
        assert_eq!(scrollbar.items_shown(), 50);
        assert_eq!(scrollbar.ratio_above(), 0.0);
        assert_eq!(scrollbar.ratio_shown(), 0.5);
        assert_eq!(scrollbar.ratio_below(), 0.5);

        scrollbar.set_shown_range(50..100);
        assert_eq!(scrollbar.items_shown(), 50);
        assert_eq!(scrollbar.ratio_above(), 0.5);
        assert_eq!(scrollbar.ratio_shown(), 0.5);
        assert_eq!(scrollbar.ratio_below(), 0.0);

        scrollbar.set_total(10).set_shown_range(1..4); // 10 items (0,1,2,3,4,5,6,7,8,9), 1+2+3 are shown.
        assert_eq!(scrollbar.items_shown(), 3);
        assert_eq!((scrollbar.ratio_above() * 100.0).round(), 10.0); // 1 item above (10%)
        assert_eq!((scrollbar.ratio_shown() * 100.0).round(), 30.0); // 3 items shown (30%)
        assert_eq!((scrollbar.ratio_below() * 100.0).round(), 60.0); // 6 items shown (60%)

        scrollbar.set_shown_range(5..10);
        assert_eq!(scrollbar.items_shown(), 5);
    }

    #[test]
    fn it_calculates_heights() {
        let mut scrollbar = Scrollbar::default();

        scrollbar.set_total(100).set_shown_range(50..80);

        assert_eq!(scrollbar.above_height(100), 50);
        assert_eq!(scrollbar.shown_height(100), 30);
        assert_eq!(scrollbar.below_height(100), 20);

        assert_eq!(scrollbar.above_height(50), 25);
        assert_eq!(scrollbar.shown_height(50), 15);
        assert_eq!(scrollbar.below_height(50), 10);

        for height in 3..500 {
            assert_eq!(
                scrollbar.above_height(height)
                    + scrollbar.shown_height(height)
                    + scrollbar.below_height(height),
                height,
                "Heights do not add up to {}",
                height
            );
        }
    }

    #[test]
    fn it_deals_with_float_rounding_errors() {
        let mut scrollbar = Scrollbar::default();
        scrollbar.set_total(398).set_shown_range(377..398);

        assert!(
            scrollbar.ratio_above() >= 0.0,
            "Ratio above was negative: {}",
            scrollbar.ratio_above()
        );
        assert!(
            scrollbar.ratio_shown() >= 0.0,
            "Ratio shown was negative: {}",
            scrollbar.ratio_shown()
        );
        assert!(
            scrollbar.ratio_below() >= 0.0,
            "Ratio below was negative: {}",
            scrollbar.ratio_below()
        );
    }
}
