//! # Byte-counting utilities
//!
//! Helpers built on the [`memchr`] crate for counting occurrences of a byte or
//! byte sequence in a haystack, with optional handling of escape sequences and
//! quoted delimiters. Also provides [`row_col_pos`] for computing 1-indexed
//! positions used in error messages.

use memchr::memchr_iter;

/// # Needle abstraction over the [`memchr`] crate
pub mod memchr_needle {
    use std::rc::Rc;

    use memchr::{
        Memchr, memchr_iter,
        memmem::{FindIter, Finder},
    };

    /// # Search pattern
    ///
    /// Either a single byte (`One`) or a byte-sequence matcher (`Finder`)
    pub enum Kind<'a> {
        /// Matches a single byte
        One(u8),
        /// Matches a byte sequence
        Finder(Rc<Finder<'a>>),
    }

    impl<'a> Kind<'a> {
        /// Creates an iterator over `needle` matches in `haystack`
        pub fn iter(&'a self, haystack: &'a [u8]) -> Iter<'a> {
            match self {
                Kind::One(byte) => Iter::One(memchr_iter(*byte, haystack)),
                Kind::Finder(finder) => {
                    Iter::Finder(Box::new(finder.find_iter(haystack)))
                }
            }
        }
    }

    /// # Unified needle iterator
    ///
    /// Wraps either kind of [`Kind`] behind a single `Iterator`, hiding
    /// the `memchr` / `memmem` difference from the caller.
    pub enum Iter<'a> {
        /// Iterator over single-byte matches
        One(Memchr<'a>),
        /// Iterator over byte-sequence matches
        Finder(Box<FindIter<'a, 'a>>),
    }

    impl<'a> Iterator for Iter<'a> {
        type Item = usize;

        fn next(&mut self) -> Option<Self::Item> {
            match self {
                Iter::One(it) => it.next(),
                Iter::Finder(it) => it.next(),
            }
        }
    }
}

/// # Row-column position for the last byte at the given `source`
///
/// Both row and column are 1-indexed, as in most of text editors. This function
/// is useful for error messages when there is a need to locate a position in a
/// text file.
///
/// # Example
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use a_bc::utils::row_col_pos;
///
/// // Four rows
/// let source =
/// b"some
/// kind
/// of
/// source";
///
/// // Here is given entire source, so returned position describes position
/// // of the last byte in source ('e' in the word "source")
/// assert_eq!(
///     row_col_pos(source),
///     (4, 6) // Fourth row and sixth column
/// );
///
/// // Here is given part of source, so returned data describes position
/// // of the last byte in slice ('d' in the word "kind")
/// assert_eq!(
///     row_col_pos(&source[..9]),
///     (2, 4) // Second row and fourth column
/// );
/// ```
pub fn row_col_pos(source: &[u8]) -> (usize, usize) {
    let it = memchr_iter(b'\n', source);

    let mut row = 1; // because rows start from 1

    let mut previous_newline_position = 0;
    let mut current_newline_position = 0;
    for i in it {
        row += 1;
        previous_newline_position = current_newline_position;
        current_newline_position = i + 1; // + 1 because columns start from 1
    }

    let mut col = source.len() - current_newline_position;

    if col == 0 {
        row -= 1;
        col = current_newline_position - previous_newline_position;
    }

    (row, col)
}

/// # Count occurrences of `needle` in `haystack`
///
/// Under the hood this function uses [`memchr`] crate to iterate through
/// `haystack`, so it uses [`memchr_needle::Kind`].
///
/// # Example
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use a_bc::utils::{memchr_needle, memchr_count_needles};
///
/// let source = b"some kind of source";
/// // Count 'o'    ^        ^   ^
///
/// assert_eq!(
///     memchr_count_needles(
///         &memchr_needle::Kind::One(b'o'),
///         source
///     ),
///     3
/// );
/// ```
pub fn memchr_count_needles<'a>(
    needle: &memchr_needle::Kind<'a>,
    haystack: &[u8],
) -> usize {
    needle.iter(haystack).count()
}

/// # Count occurrences of `needle` in `haystack`, ignoring `escaped` ones
///
/// A match is counted only when the number of consecutive `escape` bytes
/// immediately before it is even; an odd count means the match itself is
/// escaped and skipped. For example, with `b"\"` as an `escape`, in `haystack`
/// `a\\,b` counts the comma, while `a\,b` does not (backslash escapes the
/// comma).
///
/// Under the hood this function uses [`memchr`] crate to iterate through
/// `haystack`, so it uses [`memchr_needle::Kind`].
///
/// # Example
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use a_bc::utils::{memchr_needle, memchr_count_needles_considering_escapes};
///
/// let source = b"s\\ome kind of s\\ource";
/// // Count 'o'               ^
///
/// assert_eq!(
///     memchr_count_needles_considering_escapes(
///         &memchr_needle::Kind::One(b'o'),
///         b"\\",
///         source
///     ),
///     1
/// );
/// ```
pub fn memchr_count_needles_considering_escapes<'a>(
    needle: &memchr_needle::Kind<'a>,
    escape: &'a [u8],
    haystack: &[u8],
) -> usize {
    let it = needle.iter(haystack);

    let mut counter = 0;

    for mut i in it {
        let mut escape_counter = 0;

        while i > 0 {
            let old_i = i;
            i = if let Some(i) = i.checked_sub(escape.len()) {
                i
            } else {
                break;
            };

            if &haystack[i..old_i] == escape {
                escape_counter += 1;
            } else {
                break;
            }
        }

        if escape_counter % 2 == 0 {
            counter += 1;
        }
    }

    counter
}

/// # Count occurrences of `needle` in `haystack`, ignoring `delimited` ones
///
/// A `delimiter` toggles the "inside a delimited region" state (e.g. a
/// `"`-quoted string); matches found while inside are not counted. The running
/// `delimiter_counter` is updated in place and must be kept across calls so a
/// region opened in one chunk is still recognized in the next.
///
/// Under the hood this function uses [`memchr`] crate to iterate through
/// `haystack`, so it uses [`memchr_needle::Kind`].
///
/// # Example
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use a_bc::utils::{
///     memchr_needle,
///     memchr_count_needles_considering_delimiters
/// };
///
/// let source = b"some \"kind of\" \"source\"";
/// // Count 'o'    ^
/// let mut delimiter_count = 0;
///
/// assert_eq!(
///     memchr_count_needles_considering_delimiters(
///         &memchr_needle::Kind::One(b'o'),
///         &memchr_needle::Kind::One(b'"'),
///         &mut delimiter_count,
///         source
///     ),
///     1
/// );
/// assert_eq!(delimiter_count, 4);
/// ```
pub fn memchr_count_needles_considering_delimiters<'a>(
    needle: &memchr_needle::Kind<'a>,
    delimiter: &memchr_needle::Kind<'a>,
    delimiter_counter: &mut usize,
    mut haystack: &[u8],
) -> usize {
    let mut count = 0;

    loop {
        let mut it = needle.iter(haystack);

        if let Some(position) = it.next() {
            *delimiter_counter +=
                memchr_count_needles(delimiter, &haystack[..position]);

            if delimiter_counter.is_multiple_of(2) {
                count += 1;
            }

            if position + 1 >= haystack.len() {
                break;
            } else {
                haystack = &haystack[position + 1..];
            }
        } else {
            *delimiter_counter += memchr_count_needles(delimiter, haystack);

            break;
        }
    }

    count
}

/// # Count occurrences of `needle` in `haystack`, considering `escaped` `delimiters`
///
/// Combines [`memchr_count_needles_considering_delimiters()`] with
/// [`memchr_count_needles_considering_escapes()`]: delimiters themselves can be
/// escaped, and only unescaped ones toggle the "inside a delimited region"
/// state. `delimiter_counter` is updated in place and must be kept across
/// calls.
///
/// Under the hood this function uses [`memchr`] crate to iterate through
/// `haystack`, so it uses [`memchr_needle::Kind`].
///
/// # Example
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use a_bc::utils::{
///     memchr_needle,
///     memchr_count_needles_considering_escaped_delimiters
/// };
///
/// let source = b"some \\\"kind of\\\" \"source\"";
/// // Count 'o'    ^            ^
/// let mut delimiter_count = 0;
///
/// assert_eq!(
///     memchr_count_needles_considering_escaped_delimiters(
///         &memchr_needle::Kind::One(b'o'),
///         b"\\",
///         &memchr_needle::Kind::One(b'"'),
///         &mut delimiter_count,
///         source
///     ),
///     2
/// );
/// assert_eq!(delimiter_count, 2);
/// ```
pub fn memchr_count_needles_considering_escaped_delimiters<'a>(
    needle: &memchr_needle::Kind<'a>,
    delimiter_escape: &'a [u8],
    delimiter: &memchr_needle::Kind<'a>,
    delimiter_counter: &mut usize,
    mut haystack: &[u8],
) -> usize {
    let mut count = 0;

    loop {
        let mut it = needle.iter(haystack);

        if let Some(position) = it.next() {
            *delimiter_counter += memchr_count_needles_considering_escapes(
                delimiter,
                delimiter_escape,
                &haystack[..position],
            );

            if delimiter_counter.is_multiple_of(2) {
                count += 1;
            }

            if position + 1 >= haystack.len() {
                break;
            } else {
                haystack = &haystack[position + 1..];
            }
        } else {
            *delimiter_counter += memchr_count_needles_considering_escapes(
                delimiter,
                delimiter_escape,
                haystack,
            );

            break;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use memchr::memmem::Finder;
    use pretty_assertions::assert_eq;

    use crate::utils::{
        memchr_count_needles_considering_delimiters,
        memchr_count_needles_considering_escaped_delimiters,
        memchr_count_needles_considering_escapes, memchr_needle,
    };

    #[test]
    fn count_needles_considering_backslashes_no_matches() {
        let haystack = b"hello";

        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::One(b'x'),
                b"\\",
                haystack
            ),
            0
        );

        let finder = Rc::new(Finder::new(b"x"));
        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::Finder(finder),
                b"\\",
                haystack
            ),
            0
        );
    }

    #[test]
    fn count_needles_considering_backslashes_no_escape() {
        let haystack = b"a,b,c";

        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::One(b','),
                b"\\",
                haystack
            ),
            2
        );

        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::Finder(finder),
                b"\\",
                haystack
            ),
            2
        );
    }

    #[test]
    fn count_needles_considering_backslashes_single_escape() {
        let haystack = b"a,b\\,c";

        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::One(b','),
                b"\\",
                haystack
            ),
            1
        );

        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::Finder(finder),
                b"\\",
                haystack
            ),
            1
        );
    }

    #[test]
    fn count_needles_considering_backslashes_double_escape() {
        let haystack = b"a,b\\\\,c";

        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::One(b','),
                b"\\",
                haystack
            ),
            2
        );

        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::Finder(finder),
                b"\\",
                haystack
            ),
            2
        );
    }

    #[test]
    fn count_needles_considering_backslashes_mixed() {
        let haystack = b"x,\\,x,,\\\\,";

        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::One(b','),
                b"\\",
                haystack
            ),
            4
        );

        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::Finder(finder),
                b"\\",
                haystack
            ),
            4
        );
    }

    #[test]
    fn count_needles_considering_backslashes_large() {
        let mut haystack = Vec::new();
        for i in 0..1000 {
            haystack.push(b'a');
            if i % 2 == 0 {
                haystack.push(b',');
            } else {
                haystack.push(b'\\');
                haystack.push(b',');
            }
        }

        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::One(b','),
                b"\\",
                &haystack
            ),
            500
        );

        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_escapes(
                &memchr_needle::Kind::Finder(finder),
                b"\\",
                &haystack
            ),
            500
        );
    }

    #[test]
    fn count_needles_considering_quotes_empty() {
        let haystack = b"";

        let mut quotes = 0;
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::One(b','),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            0
        );
        assert_eq!(quotes, 0);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::Finder(finder),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            0
        );
        assert_eq!(quotes, 0);
    }

    #[test]
    fn count_needles_considering_quotes_no_matches() {
        let haystack = b"hello world";

        let mut quotes = 0;
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::One(b','),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            0
        );
        assert_eq!(quotes, 0);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::Finder(finder),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            0
        );
        assert_eq!(quotes, 0);
    }

    #[test]
    fn count_needles_considering_quotes_outside_quotes() {
        let haystack = b"a,b,c";

        let mut quotes = 0;
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::One(b','),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            2
        );
        assert_eq!(quotes, 0);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::Finder(finder),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            2
        );
        assert_eq!(quotes, 0);
    }

    #[test]
    fn count_needles_considering_quotes_inside_quotes() {
        let haystack = b"\"a,b,c\"";

        let mut quotes = 0;
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::One(b','),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            0
        );
        assert_eq!(quotes, 2);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::Finder(finder),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            0
        );
        assert_eq!(quotes, 2);
    }

    #[test]
    fn count_needles_considering_quotes_mixed() {
        let haystack = b"a,\"b,c\",d,e";

        let mut quotes = 0;
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::One(b','),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            3
        );
        assert_eq!(quotes, 2);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::Finder(finder),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            3
        );
        assert_eq!(quotes, 2);
    }

    #[test]
    fn count_needles_considering_quotes_escaped_quotes() {
        let haystack = b"a,\"b,\\\"c\",d";

        let mut quotes = 0;
        assert_eq!(
            memchr_count_needles_considering_escaped_delimiters(
                &memchr_needle::Kind::One(b','),
                b"\\",
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            2
        );
        assert_eq!(quotes, 2);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_escaped_delimiters(
                &memchr_needle::Kind::Finder(finder),
                b"\\",
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            2
        );
        assert_eq!(quotes, 2);
    }

    #[test]
    fn count_needles_considering_quotes_complex() {
        let haystack = b"a,b,\"c,d\",e,\"f\",g,h";

        let mut quotes = 0;
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::One(b','),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            6
        );
        assert_eq!(quotes, 4);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::Finder(finder),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            6
        );
        assert_eq!(quotes, 4);
    }

    #[test]
    fn count_needles_considering_quotes_large() {
        let mut haystack = Vec::new();
        for i in 0..500 {
            if i % 10 == 0 {
                haystack.extend_from_slice(b"\"");
            }
            haystack.push(b'a');
            haystack.push(b',');
            if i % 10 == 9 {
                haystack.extend_from_slice(b"\"");
            }
            haystack.push(b',');
        }

        let mut quotes = 0;
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::One(b','),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                &haystack,
            ),
            50
        );
        assert_eq!(quotes, 100);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::Finder(finder),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                &haystack,
            ),
            50
        );
        assert_eq!(quotes, 100);
    }

    #[test]
    fn count_needles_considering_quotes_custom_chunk_size() {
        let haystack = b"a,b,\"c,d\",e,f";

        let mut quotes = 0;
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::One(b','),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            4
        );
        assert_eq!(quotes, 2);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            memchr_count_needles_considering_delimiters(
                &memchr_needle::Kind::Finder(finder),
                &memchr_needle::Kind::One(b'"'),
                &mut quotes,
                haystack,
            ),
            4
        );
        assert_eq!(quotes, 2);
    }
}
