#![allow(dead_code)]
#![allow(unused_imports)]
use ropey::*;

pub type SearchResult<'a, T> = std::result::Result<std::option::Option<T>, SearchError<'a>>;

pub(crate) trait SearchAlgorithms {}

pub struct SearchIter<'a> {
    char_iter: iter::Chars<'a>,
    search_pattern: &'a str,
    search_pattern_char_len: usize,
    cur_index: usize, // The current char index of the search head.
    possible_matches: Vec<std::str::Chars<'a>>, // Tracks where we are in the search pattern for the current possible matches.
}

impl<'a> SearchIter<'a> {
    fn from_rope_slice<'b>(slice: &'b RopeSlice, search_pattern: &'b str) -> SearchIter<'b> {
        assert!(
            !search_pattern.is_empty(),
            "Can't search using an empty search pattern."
        );
        SearchIter {
            char_iter: slice.chars(),
            search_pattern,
            search_pattern_char_len: search_pattern.chars().count(),
            cur_index: 0,
            possible_matches: Vec::new(),
        }
    }
}
impl<'a> Iterator for SearchIter<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        #[allow(clippy::while_let_on_iterator)]
        while let Some(_next_char) = self.char_iter.next() {}
        None
    }
}
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum SearchError<'a> {
    /// Indicates that the search pattern has 0 elements.
    EmptyPattern,
    /// Indicates the search pattern did not meet expectations.
    ///
    /// Contains the pattern.
    InvalidPattern(&'a str),
}

impl<'a> std::error::Error for SearchError<'_> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    // Deprecated in std.
    fn description(&self) -> &str {
        ""
    }

    // Deprecated in std.
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}
impl<'a> std::fmt::Debug for SearchError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SearchError::EmptyPattern => write!(f, "Can't search on empty pattern."),
            SearchError::InvalidPattern(_pat) => write!(f, "Pattern can no be used to search."),
        }
    }
}
impl<'a> std::fmt::Display for SearchError<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_algo() {}
}
