extern crate ratatui;
extern crate ropey;

mod document;
mod search;
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
            // I think the only error case I can expect here is an out of bounds error?
            // IDEA: Write a function that gets visible size by Rect area and produces a paragraph by rope len?
            // IDEA: Do more bounds checking on document.visible_range and handle error in to_line. 
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
pub(crate) trait CustomConvertions {
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
