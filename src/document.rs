#![allow(dead_code)]
#![allow(unused_imports)]
use ratatui::{
    layout::{self, Alignment, Position},
    style::{self, Styled},
    text::{self, ToSpan},
    widgets::{self, Block},
};
use ropey::{Error, Rope, RopeBuilder};
use std::{
    cmp, error, fmt, io,
    ops::{self, Bound, RangeBounds},
    path::{self, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::RopeyResult;


#[derive(Debug, PartialEq, Eq)]
pub struct Document<'a> {
    name: Option<path::PathBuf>,
    modified: bool,
    pub(crate) rope: ropey::Rope,
    pub(crate) visible_range: std::ops::RangeInclusive<usize>,

    pub(crate) block: Option<widgets::Block<'a>>,
    pub(crate) style: style::Style,
    pub(crate) wrap: Option<widgets::Wrap>,
    pub(crate) scroll: Position,
    pub(crate) alignment: Alignment,
}

impl<'a> Document<'a> {
    /// Creates an empty 'Document'.
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("Hello\n world!");
    /// let mut lines = doc.lines();

    /// let empty_rope = Document::new();
    /// assert_eq!(empty_rope.len_lines(), 2);
    /// assert_eq!(empty_rope.len_chars(), 0);
    /// assert_eq!(empty_rope.len_bytes(), 0);
    /// assert!(empty_rope.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            name: None,
            modified: false,
            rope: Rope::new(),
            visible_range: ops::RangeInclusive::new(0, 0),
            block: None,
            style: style::Style::default(),
            wrap: None,
            scroll: Position::ORIGIN,
            alignment: Alignment::Left,
        }
    }
    /// Creates a 'Document' from a string slice.
    ///
    /// This is a convience constructor.
    pub fn from_str(text: &str) -> Self {
        Self {
            name: None,
            rope: Rope::from_str(text),
            modified: false,
            visible_range: ops::RangeInclusive::new(0, text.len()),

            block: None,
            style: style::Style::default(),
            wrap: None,
            scroll: Position::ORIGIN,
            alignment: Alignment::Left,
        }
    }
    /// Creates a new `Document` from a reader.
    ///
    /// # Errors
    ///
    /// - If the reader returns an error, `from_reader` stops and returns
    ///   that error.
    /// - If non-utf8 data is encountered, an IO error with kind
    ///   `InvalidData` is returned.
    /// - If the PathBuf can't be identified as a file, an IO error with
    ///    with kind 'NotFound' is returned.
    ///
    pub fn from_reader<R: io::Read>(reader: R) -> io::Result<Self> {
        let rope: ropey::Rope = ropey::Rope::from_reader(reader)?;
        let char_len = rope.len_chars();
        Ok(Self {
            name: None,
            rope,
            modified: false,
            visible_range: ops::RangeInclusive::new(0, char_len),

            block: Some(widgets::Block::<'_>::default()),
            style: style::Style::default(),
            wrap: None,
            scroll: Position::ORIGIN,
            alignment: Alignment::Left,
        })
    }
    /// Writes the contents of the underlying `Rope` to a writer.
    ///
    /// When more precise control over IO behavior, buffering, etc. is
    /// desired, you should handle IO yourself and use the `Chunks`
    /// iterator to iterate through the `Rope`'s contents.
    ///
    /// Runs in O(N) time.
    ///
    /// # Errors
    ///
    /// - If the writer returns an error, `write_file` stops and returns that
    ///   error.
    ///
    /// Note: some data may have been written even if an error is returned.
    pub fn write_file<T: io::Write>(&self, mut writer: T) -> io::Result<()> {
        for chunk in self.rope.chunks() {
            writer.write_all(chunk.as_bytes())?;
        }

        Ok(())
    }
    /// Total number of bytes in the underlying `Rope`.
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }
    /// Total number of chars in the underlying `Rope`.
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }
    /// Total number of lines in the underlying `Rope`.
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines() + 1
    }
    /// Returns true if the rope contains no elements.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::default();
    /// assert_eq!(doc.is_empty(), true);
    ///
    /// ```
    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }
    /// An iterator over the underlying `Rope`'s chars starting at a given position.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("hi");
    /// let mut iter = doc.chars_at(0);
    /// assert_eq!(iter.next(), Some('h'));
    /// assert_eq!(iter.next(), Some('i'));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn chars_at<'chr>(&'chr self, pos: usize) -> ropey::iter::Chars<'chr> {
        self.rope.chars_at(pos)
    }
    /// An iterator over a `Rope`'s lines at a given position.
    ///
    /// See 'lines' for more details.
    ///
    /// Runs in O(log N) time.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("Hello\n world!");
    /// let mut lines = doc.lines_at(1);
    /// assert_eq!(lines.next().unwrap().as_str(), Some(" world!"));
    /// assert_eq!(lines.next(), None);
    /// ```
    pub fn lines_at<'lines>(&'lines self, row: usize) -> ropey::iter::Lines<'lines> {
        self.rope.lines_at(row)
    }
    /// An iterator over a `Rope`'s lines.
    ///
    /// The returned lines include the line break at the end, if any.
    ///
    /// A rope will always start with a signle line, even if a line break is not present.
    ///
    /// The last line is returned even if blank, in which case it
    /// is returned as an empty slice.
    ///
    /// Runs in O(log N) time.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("Hello\n world!");
    /// let mut lines = doc.lines();
    ///
    /// assert_eq!(lines.next().unwrap().as_str(), Some("Hello\n"));
    /// assert_eq!(lines.next().unwrap().as_str(), Some(" world!"));
    /// assert_eq!(lines.next(), None);
    /// ```
    pub fn lines<'lines>(&'lines self) -> ropey::iter::Lines<'lines> {
        self.rope.lines()
    }
    /// An iterator over a `Rope`'s contiguous `str` chunks.
    ///
    /// Internally, each `Rope` stores text as a segemented collection of utf8
    /// strings. This iterator iterates over those segments, returning a
    /// `&str` slice for each one.  It is useful for situations such as:
    ///
    /// - Writing a rope's utf8 text data to disk (but see
    ///   [`write_to()`](crate::Document::write_file) for a convenience function that does this
    ///   for casual use-cases).
    /// - Streaming a rope's text data somewhere.
    /// - Saving a rope to a non-utf8 encoding, doing the encoding conversion
    ///   incrementally as you go.
    /// - Writing custom iterators over a rope's text data.
    ///
    /// There are precisely two guarantees about the yielded chunks:
    ///
    /// - All non-empty chunks are yielded, and they are yielded in order.
    /// - CRLF pairs are never split across chunks.
    ///
    /// There are no guarantees about the size of yielded chunks, and except for
    /// CRLF pairs and being valid `str` slices there are no guarantees about
    /// where the chunks are split.  For example, they may be zero-sized, they
    /// don't necessarily align with line breaks, etc.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("Hello world!");
    /// let mut chunk = doc.chunks();
    ///
    /// assert_eq!(chunk.next(), Some("Hello world!"));
    /// assert_eq!(chunk.next(), None);
    /// ```
    pub fn chunks<'chunks>(&'chunks self) -> ropey::iter::Chunks<'chunks> {
        self.rope.chunks()
    }
    /// Gets an immutable slice of the underlying Rope, using char indices.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("Hello\nworld!");
    /// let expected = doc.visible_slice().unwrap().as_str();
    /// assert_eq!(expected, Some("Hello\nworld!"));
    /// ```
    pub fn visible_slice(&self) -> Option<ropey::RopeSlice<'_>> {
        self.rope
            .get_slice(self.visible_range.start()..self.visible_range.end())
    }
    /// Gets an immutable slice of the underlying Rope, based on the visible range of the document.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("Hello\nworld!");
    /// let expected = doc.slice(..6).unwrap().as_str();
    /// assert_eq!(expected, Some("Hello\n"));
    /// ```
    pub fn slice<R>(&self, char_range: R) -> Option<ropey::RopeSlice<'_>>
    where
        R: RangeBounds<usize>,
    {
        self.rope.get_slice(char_range)
    }
    /// Convert cursor position to a character position in the rope.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("Hello \nworld!");
    ///
    /// assert_eq!(doc.cursor_to_char(0, 1), 1);
    /// assert_eq!(doc.cursor_to_char(1, 1), 8);
    /// ```
    pub fn cursor_to_char(&self, row: usize, col: usize) -> usize {
        self.rope.line_to_char(row) + col
    }
    /// Insert a single character at the current cursor position.
    ///
    /// Does not panic.
    ///
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let mut doc = Document::from_str("ab");
    /// doc.insert_char('c', 0, 2);
    /// let mut iter = doc.chars_at(2);
    /// assert_eq!(iter.next(), Some('c'));
    /// ```
    pub fn insert_char(&mut self, input: char, row: usize, col: usize) -> RopeyResult<()> {
        self.modified = true;
        // Rope crate handles bounds checking.
        match self
            .rope
            .try_insert_char(self.cursor_to_char(row, col), input)
        {
            Ok(()) => Ok(()),
            Err(err) => Err(err),
        }
    }

    /// Insert a string at the current cursor position.
    ///
    /// Does not panic.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let mut doc = Document::from_str("Hello beautiful world!");
    /// doc.insert(0, 22, "\nGoodbye cruel world!");
    ///
    /// let expected = doc.slice(22..).unwrap().as_str();
    /// assert_eq!(expected, Some("Goodbye cruel world!"));
    /// ```
    pub fn insert<RefStr: AsRef<str>>(
        &mut self,
        row: usize,
        col: usize,
        input: RefStr,
    ) -> RopeyResult<()> {
        self.modified = true;
        let lines: Vec<_> = input
            .as_ref()
            .split('\n')
            .map(|s| s.strip_suffix('\r').unwrap_or(s).to_string())
            .collect();
        let mut pos = self.cursor_to_char(row, col);

        for line in lines.iter() {
            match self.rope.try_insert(pos, line) {
                Ok(_) => pos += line.len(),
                Err(err) => return Err(err),
            }
        }
        Ok(())
    }
    /// Removes the text in the given char index range.
    ///
    /// Uses range syntax, e.g. `2..7`, `2..`, etc.  The range is in `char`
    /// indices.
    ///
    /// Runs in O(M + log N) time, where N is the length of the `Rope` and M
    /// is the length of the range being removed.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    /// let mut doc = Document::from_str( "Hello world!");
    /// doc.remove(5..);
    /// let expected = doc.slice(..).unwrap().as_str();
    ///
    /// assert_eq!(expected, Some("Hello"));
    /// ```
    pub fn remove<R>(&mut self, char_range: R) -> RopeyResult<()>
    where
        R: RangeBounds<usize>,
    {
        self.rope.try_remove(char_range)
    }
    /// Surrounds the [`Document`] widget with a `Block`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ratatui::widgets::{Block, Paragraph};
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("Hello, world!").block(Block::bordered().title("Document"));
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: widgets::Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
    /// Sets the wrapping configuration for the widget.
    ///
    /// See [`Wrap`](https://docs.rs/ratatui/latest/ratatui/widgets/struct.Wrap.html) for more information on the different options.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ratatui::widgets::{Paragraph, Wrap};
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("Hello, world!").wrap(Wrap { trim: true });
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn wrap(mut self, wrap: widgets::Wrap) -> Self {
        self.wrap = Some(wrap);
        self
    }
    /// Set the text alignment for the given widget.
    ///
    /// The alignment is a variant of the Ratatui [`Alignment`](https://docs.rs/ratatui/latest/ratatui/layout/enum.Alignment.html) enum which can be one of Left, Right, or
    /// Center. If no alignment is specified, the text in a paragraph will be left-aligned.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ratatui::{layout::Alignment, widgets::Paragraph};
    ///
    /// let paragraph = Paragraph::new("Hello World").alignment(Alignment::Center);
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub const fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }
    /// Sets the style of the entire widget.
    ///
    /// `style` accepts any type that is convertible to [`Style`](https://docs.rs/ratatui/latest/ratatui/style/struct.Style.html)
    /// (e.g. [`Style`](https://docs.rs/ratatui/latest/ratatui/style/struct.Style.html), [`Color`](https://docs.rs/ratatui/latest/ratatui/style/enum.Color.html) , or
    /// your own type that implements [`Into<Style>`]).
    ///
    /// This applies to the entire widget, including the block if one is present. Any style set on
    /// the block or text will be added to this style.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ratatui::style::{Style, Stylize};
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("Hello, world!").set_style(Style::new().red().on_white());
    /// ```
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn set_style<S: Into<style::Style>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }
    /// Set the name of the document.
    ///
    /// Use rename to replace the name of the document.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let mut doc = Document::from_str("Hello, world!");
    /// doc.name("example.txt");
    /// ```
    pub fn name<S: AsRef<str>>(&mut self, path: S) {
        if self.name.is_none() {
            self.name = Some(PathBuf::from(path.as_ref()))
        }
    }
    /// Rename the document.
    ///
    /// # Example
    ///
    /// ```
    /// use tui_document::Document;
    ///
    /// let mut doc = Document::from_str("Hello, world!");
    /// doc.rename("rename_example.txt");
    /// ```
    pub fn rename<S: AsRef<str>>(&mut self, path: S) {
        self.name = Some(PathBuf::from(path.as_ref()))
    }
}
impl<'a> Default for Document<'_> {
    fn default() -> Self {
        Self::new()
    }
}
impl<'a> Clone for Document<'_> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            rope: self.rope.clone(),
            modified: self.modified,
            visible_range: self.visible_range.clone(),

            block: self.block.clone(),
            style: self.style.clone(),
            wrap: self.wrap,
            scroll: self.scroll,
            alignment: self.alignment,
        }
    }
}
impl<'a> std::cmp::Ord for Document<'_> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.rope.slice(..).cmp(&other.rope.slice(..))
    }
}
impl<'a> std::cmp::PartialOrd<Document<'a>> for Document<'a> {
    fn partial_cmp(&self, other: &Document) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<'a, Iter, S> From<Iter> for Document<'_>
where
    Iter: Iterator<Item = S>,
    S: ToString,
{
    /// ```
    /// use tui_document::Document;
    ///
    /// let vec_str = vec!["test1", "test2",];
    /// let char_arr = ['a', 'b', 'c'];
    /// let str_arr = ["apple", "banana", "mango"];
    /// let rope = ropey::Rope::from_str("apple\nbanana");
    ///
    /// let from_rope = Document::from(rope.chunks());
    /// let from_str_arr = Document::from(str_arr.iter());
    /// let from_char_arr = Document::from(char_arr.iter());
    /// let from_vec = Document::from(vec_str.iter());
    /// let from_string = Document::from(String::from("hello").chars());
    /// let from_str = Document::from("hello".chars());
    /// ```
    fn from(value: Iter) -> Self {
        let mut rope_builder = ropey::RopeBuilder::new();
        for ele in value {
            rope_builder.append(&ele.to_string());
        }
        let rope = rope_builder.finish();
        Self {
            name: None,
            modified: false,
            rope,
            visible_range: ops::RangeInclusive::new(0, 0),
            block: None,
            style: style::Style::default(),
            wrap: None,
            scroll: Position::ORIGIN,
            alignment: Alignment::Left,
        }
    }
}
