#![allow(dead_code)]
#![allow(unused_imports)]
use ratatui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Widget},
};
use ropey::{Error, Rope, RopeBuilder};
use std::{
    cmp, error, fmt, io,
    ops::{Bound, RangeBounds},
    path,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::RopeyResult;

#[derive(Debug, PartialEq, Eq)]
pub struct Document<'a> {
    name: std::path::PathBuf,
    pub(crate) rope: ropey::Rope,
    doc_pos: usize,
    pub(crate) modified: bool,
    tab_len: u8,
    hard_tab_indent: bool,
    pub(crate) alignment: Alignment,

    pub(crate) block: Option<Block<'a>>,
    pub(crate) style: Style,
    pub(crate) line_number_style: Option<Style>,
}

impl<'a> Document<'_> {
    /// Creates an empty 'Document'.
    pub fn new(name: &str) -> Self {
        Self {
            name: path::PathBuf::from(name),
            rope: Rope::new(),
            doc_pos: 0,
            modified: false,
            tab_len: 4,
            hard_tab_indent: false,
            alignment: Alignment::Left,

            block: None,
            style: Style::default(),
            line_number_style: None,
        }
    }
    /// Creates a 'Document' from a string slice.    
    pub fn from_str(name: &str, text: &str) -> Self {
        Self {
            name: path::PathBuf::from(name),
            rope: Rope::from_str(text),
            doc_pos: 0,
            modified: false,
            tab_len: 4,
            hard_tab_indent: false,
            alignment: Alignment::Left,

            block: None,
            style: Style::default(),
            line_number_style: None,
            //            viewport: Viewport::default(),
        }
    }
    /// Creates a 'Document' from a Vec of strings.
    pub fn new_lines(name: &str, mut lines: Vec<String>) -> Self {
        if lines.is_empty() {
            lines.push(String::new());
        }
        let mut builder = RopeBuilder::new();
        let _ = lines
            .iter()
            .map(|line| builder.append(line))
            .collect::<Vec<_>>();
        let rope = builder.finish();
        Self {
            name: path::PathBuf::from(name),
            rope,
            doc_pos: 0,
            modified: false,
            tab_len: 4,
            hard_tab_indent: false,
            alignment: Alignment::Left,

            block: None,
            style: Style::default(),
            line_number_style: None,
        }
    }
    /// Creates a `Document` from a PathBuf.
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
    pub fn open_file(path: std::path::PathBuf) -> Result<Self, io::Error> {
        let rope = match path.metadata() {
            Ok(md) => {
                if md.is_file() {
                    let file = std::fs::File::open(&path)?;
                    let rope: ropey::Rope = ropey::Rope::from_reader(io::BufReader::new(&file))?;
                    rope
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "{path:?} is not a file",
                    ));
                }
            }
            Err(_) => ropey::Rope::new(),
        };
        Ok(Self {
            name: path,
            rope,
            doc_pos: 0,
            modified: false,
            tab_len: 4,
            hard_tab_indent: false,
            alignment: Alignment::Left,

            block: None,
            style: Style::default(),
            line_number_style: None,
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
    /// - If the writer returns an error, `write_to` stops and returns that
    ///   error.
    ///
    /// Note: some data may have been written even if an error is returned.
    pub fn write_file<T: io::Write>(&self, mut writer: T) -> io::Result<()> {
        for chunk in self.rope.chunks() {
            writer.write_all(chunk.as_bytes())?;
        }

        Ok(())
    }
    /// Total number of bytes in the `Rope`.
    ///
    /// Runs in O(1) time.
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }
    /// Total number of chars in the `Rope`.
    ///
    /// Runs in O(1) time.
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }
    /// Total number of lines in the `Rope`.
    ///
    /// Runs in O(1) time.
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines() + 1
    }
    /// Returns true if the rope contains no elements.
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
    //
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("doc_name_goes_here", "hi");
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
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("doc_name_goes_here", "Hello\n world!");
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
    /// A rope will always start with a signle line, even if a line break is not present. // confirm me
    ///
    /// The last line is returned even if blank, in which case it
    /// is returned as an empty slice.
    ///
    /// Runs in O(log N) time.
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("doc_name_goes_here", "Hello\n world!");
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

    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("doc_name_goes_here", "Hello world!");
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
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("doc_name_goes_here", "Hello\nworld!");
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
    /// ```
    /// use tui_document::Document;
    ///
    /// let doc = Document::from_str("doc_name_goes_here", "Hello \nworld!");
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
    /// ```
    /// use tui_document::Document;
    ///
    /// let mut doc = Document::from_str("doc_name_goes_here", "ab");
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
    /// ```
    /// use tui_document::Document;
    ///
    /// let mut doc = Document::from_str("doc_name_goes_here", "Hello beautiful world!");
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
    /// let mut doc = Document::from_str("doc_name_goes_here", "Hello world!");
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
    /// Set the style of textarea. By default, textarea is not styled.
    /// ```
    /// use ratatui::style::{Style, Color};
    /// use tui_document::Document;
    ///
    /// let mut doc = Document::from_str("doc_name_goes_here", "Ropes rule!");
    /// let style = Style::default().fg(Color::Red);
    /// doc.set_style(style);
    /// assert_eq!(doc.style(), style);
    /// ```
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    /// Get the current style of textarea.
    pub fn style(&self) -> Style {
        self.style
    }

    /// Get the block of textarea if exists.
    pub fn block<'b>(&'b self) -> Option<&'b Block<'b>> {
        self.block.as_ref()
    }
}
impl<'a> Default for Document<'_> {
    fn default() -> Self {
        Self::new("tui_document.txt")
    }
}
impl<Iter> From<Iter> for Document<'_>
where
    Iter: IntoIterator,
    Iter::Item: Into<String>,
{
    fn from(i: Iter) -> Self {
        Self::new_lines(
            "tui_document.txt",
            i.into_iter().map(|s| s.into()).collect::<Vec<String>>(),
        )
    }
}
impl<Str: Into<String>> FromIterator<Str> for Document<'_> {
    fn from_iter<IntoIter: IntoIterator<Item = Str>>(iter: IntoIter) -> Self {
        iter.into()
    }
}
impl<'a> Clone for Document<'_> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            rope: self.rope.clone(),
            doc_pos: self.doc_pos,
            modified: self.modified,
            tab_len: self.tab_len,
            hard_tab_indent: self.hard_tab_indent,
            alignment: self.alignment,

            block: self.block.clone(),
            style: self.style,
            line_number_style: self.line_number_style,
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
