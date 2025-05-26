extern crate ratatui;
extern crate ropey;

mod document;

pub use crate::document::Document;

use ratatui::{
    buffer, layout,
    widgets::{self, Paragraph},
};
// Pass out the underlying Ropey Error messages.
pub type RopeyResult<T> = std::result::Result<T, ropey::Error>;

/// # Disclaimer:
/// Most of the good code and documentation in this crate has been taken directly from Ropey source code.
/// My goal in this project is to produce a widget that wraps a great crate and bring it to the Ratitui community. Rather than
/// mark every function that has taken influence or been copied from the Ropey source code or mark each piece of
/// documentation that was copied, modified, editied, etc after copying from Ropey source code.
///
/// All credit goes to cessen and the Ropey crate.

impl<'a> widgets::Widget for &Document<'_> {
    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer) {
        let text = if let Some(txt) = self.slice(..) {
            if let Some(txt) = txt.as_str() {
                txt
            } else {
                ""
            }
        } else {
            ""
        };
        Paragraph::new(text)
            .block(widgets::Block::default())
            .wrap(widgets::Wrap { trim: true })
            .render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::Document;

    use insta::assert_snapshot;
    use ratatui::{Terminal, backend::TestBackend};

    #[test]
    // Note to self: use cargo insta test
    fn render_widget_integration_test() {
        let app = Document::from_str(
            "TestDoc.txt",
            "This is a line of text.\nThis is another line of text.",
        );
        let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
        terminal
            .draw(|frame| frame.render_widget(&app, frame.area()))
            .unwrap();
        assert_snapshot!(terminal.backend());
    }

    #[test]
    fn test_booyer_moore() {}
}
