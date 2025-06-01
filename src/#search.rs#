#![allow(dead_code)]
#![allow(unused_imports)]
use ropey::*;
use std::collections::HashMap;
pub type SearchResult<'a, T> = std::result::Result<std::option::Option<T>, SearchError<'a>>;

pub struct SearchIter<'a> {
    char_iter: iter::Chars<'a>,
    slice: RopeSlice<'a>,
    slice_len: usize,
    pattern: &'a str,
    pattern_char_len: usize,
    cur_index: usize,
    bad_char_map: HashMap<char, usize>,
}
impl<'a> SearchIter<'a> {
    pub(crate) fn from_rope_slice<'b>(slice: RopeSlice<'b>, pattern: &'b str) -> SearchIter<'b> {
        let pattern_len = pattern.len();
        let pattern_len_dec = pattern_len - 1;

        let mut bad_char_map: HashMap<char, usize> = HashMap::with_capacity(pattern_len_dec);

        for (i, c) in pattern.chars().enumerate() {
            bad_char_map.insert(c, pattern_len_dec - i);
        }

        SearchIter {
            char_iter: slice.chars(),
            slice_len: slice.chars().count(),
            slice,
            pattern,
            pattern_char_len: pattern.chars().count(),
            cur_index: 0,
            bad_char_map,
        }
    }
    pub(crate) fn find<Txt: AsRef<str>>(
        rope_slice: ropey::RopeSlice,
        search_pattern: Txt,
        bad_char_shift_map: &HashMap<char, usize>,
    ) -> Option<usize> {
        let search_pattern = search_pattern.as_ref();

        let text_len = rope_slice.len_chars();
        let pattern_len = search_pattern.len();

        if text_len == 0 || pattern_len == 0 || text_len < pattern_len {
            return None;
        }

        let pattern_len_dec = pattern_len - 1;
        let pattern_len_inc = pattern_len + 1;

        let last_pattern_char = if let Some(last_letter) = search_pattern.chars().last() {
            last_letter
        } else {
            return None;
        };
        let mut shift = 0;

        let end_index = text_len - pattern_len;

        for (idx, cur_pattern_char) in search_pattern.chars().rev().enumerate() {
            if rope_slice.char(shift + idx) != cur_pattern_char {
                let pat_idx = shift + pattern_len;
                if pat_idx == text_len {
                    break;
                }
                shift += bad_char_shift_map
                    .get(&rope_slice.char(shift + pattern_len_dec))
                    .copied()
                    .unwrap_or(pattern_len)
                    .max({
                        let cur_char = rope_slice.char(pat_idx);

                        if cur_char == last_pattern_char {
                            1
                        } else {
                            bad_char_shift_map
                                .get(&cur_char)
                                .map(|&letter| letter + 1)
                                .unwrap_or(pattern_len_inc)
                        }
                    });
                if shift > end_index {
                    return None;
                }
            }

            if shift == end_index {
                return None;
            }

            shift += pattern_len;
            if shift > end_index {
                return None;
            }
        }
        if shift != 0 { Some(shift) } else { None }
    }
}

impl<'a> Iterator for SearchIter<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        if let Some(idx) = SearchIter::find(
            self.slice.slice(self.cur_index..self.slice_len),
            &self.pattern,
            &self.bad_char_map,
        ) {
            self.cur_index = idx;
            return Some((self.cur_index - self.pattern_char_len, idx));
        } else {
            return None;
        }
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
    use ropey;
    use std::collections::HashMap;

    static PATTERN: &str = "abc";
    static TEXT: &str = "adcabcafg";
    #[test]
    fn test_boyer_moore_search_algo_1() {
        let rope = ropey::Rope::from_str(TEXT);
        let slice = rope.slice(..);

        let pattern_len = PATTERN.len();
        let pattern_len_dec = pattern_len - 1;

        let mut bad_char_map: HashMap<char, usize> = HashMap::with_capacity(pattern_len_dec);

        for (i, c) in PATTERN.chars().enumerate() {
            bad_char_map.insert(c, pattern_len_dec - i);
        }
        let val: Option<usize> = SearchIter::find(slice, PATTERN, &bad_char_map);
        assert_eq!(Some(6), val);
    }
    #[test]
    fn test_boyer_moore_search_algo_2() {
        let rope = ropey::Rope::from_str(TEXT);
        let slice = rope.slice(..);

        let pattern_len = PATTERN.len();
        let pattern_len_dec = pattern_len - 1;

        let mut bad_char_map: HashMap<char, usize> = HashMap::with_capacity(pattern_len_dec);

        for (i, c) in PATTERN.chars().enumerate() {
            bad_char_map.insert(c, pattern_len_dec - i);
        }
        let val = SearchIter::find(slice, "", &bad_char_map);
        assert_eq!(val, None);
    }
    #[test]
    fn test_boyer_moore_search_algo_3() {
        let rope = ropey::Rope::from_str("");
        let slice = rope.slice(..);

        let pattern_len = PATTERN.len();
        let pattern_len_dec = pattern_len - 1;

        let mut bad_char_map: HashMap<char, usize> = HashMap::with_capacity(pattern_len_dec);

        for (i, c) in PATTERN.chars().enumerate() {
            bad_char_map.insert(c, pattern_len_dec - i);
        }
        let val = SearchIter::find(slice, PATTERN, &bad_char_map);
        assert_eq!(val, None);
    }
}
