//! #Project Goal
//! The goal of this project is to bring the rope data structure to a Ratatui widget.
//! This crate wraps the ['ropey'](https://crates.io/crates/ropey) crate. The rope data
//! structure is a binary tree used for efficiently manipulating long strings. The author of
//! the ropey crate has a great paper on ['Ropey's Design'](https://github.com/cessen/ropey/blob/master/design/design.md). his specific implementation of this data structure.
//!
//! #When should I use tui_document?
//! The underlying ropey crate allocates space in kilobytes. Small text documents don't need a rope.
//! This crate is for things like text editors or large log files. Medium to large documents that
//! require frequent edits or efficient search functionality.
//!
//! This is information quoted from the ropey crate documentation.
//! - On a recent mobile i7 Intel CPU, Ropey performed over 1.8 million small
//!   incoherent insertions per second while building up a text roughly 100 MB
//!   large.  Coherent insertions (i.e. all near the same place in the text) are
//!   even faster, doing the same task at over 3.3 million insertions per
//!   second.
//! - Freshly loading a file from disk only incurs about 10% memory overhead.  For
//!   example, a 100 MB text file will occupy about 110 MB of memory when loaded
//!   by Ropey.
//! - Cloning ropes is _extremely_ cheap.  Rope clones share data, so an initial
//!   clone only takes 8 bytes of memory.  After that, memory usage will grow
//!   incrementally as the clones diverge due to edits.
//!
//!```no_run
//!```
//!
//!

extern crate ratatui;
extern crate ropey;

mod document;
pub use crate::document::Document;

use ratatui::{buffer, layout, text, widgets::*};

pub type RopeyResult<T> = std::result::Result<T, ropey::Error>;

impl<'a> Document<'_> {
    fn render_document(&self, text_area: layout::Rect, buf: &mut buffer::Buffer) {
        let mut visible_slice = match self.visible_slice() {
            Some(slice) => slice,
            None => self.rope.slice(..),
        };
        match visible_slice.to_line() {
            Ok(lines) => {
                let text = text::Text::from(lines);
                Paragraph::new(text)
                    .block(Block::default())
                    .wrap(self.wrap.unwrap_or(Wrap { trim: true }))
                    .render(text_area, buf);
            }
            // I think the only real error case I can expect here is an out of bounds error.
            // IDEA: Write a function that gets visible size by Rect area and produces a paragraph by rope len?
            // IDEA: Do more bounds checking on document.visible_range?
            Err(err) => panic!("{}", err),
        };
    }
}

impl<'a> Widget for Document<'_> {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer) {
        self.render_document(area, buf);
    }
}
impl<'a> Widget for &Document<'_> {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer) {
        let _ = &self.render_document(area, buf);
    }
}
pub trait CustomConvertions {
    fn to_line<'a>(&'a mut self) -> RopeyResult<text::Line<'a>>;
}
impl CustomConvertions for ropey::RopeSlice<'_> {
    fn to_line<'a>(&'a mut self) -> RopeyResult<text::Line<'a>> {
        let spans = self
            .chunks()
            .map(|chunk| text::Span::raw(chunk))
            .collect::<Vec<text::Span<'_>>>();
        Ok(text::Line::from(spans))
    }
}

#[cfg(test)]
mod tests {
    use super::Document;

    use insta::assert_snapshot;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    fn render_widget_integration_test() {
        let app = Document::from_str(TEXT);
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();
        //        This throws an error with my LSP so I keep it commented out unless I'm testing.
        //        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_booyer_moore() {}

    static TEXT: &str = "
A widget to display some text. It is used to display a block of text. The text can be styled and aligned. It can also be wrapped to the next line if it is too long to fit in the given area.
    ";
}
