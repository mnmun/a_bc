//! # Byte-counting utilities
//!
//! ![](https://github.com/mnmun/images/blob/main/maintenance.png?raw=true)
//!
//! Provides the following byte-counting utilities implemented on top of the
//! [`memchr`] crate:
//!
//! - [`count_needles()`];
//! - [`count_needles_considering_escapes()`];
//! - [`count_needles_considering_delimiters()`];
//! - [`count_needles_considering_escaped_delimiters()`].
//!
//! In addition, provides the helper structure required for the use of the
//! function listed above: [`Needle`] - the needle abstraction for the
//! [`memchr`] crate.
//!
//! An example of using these functions is the lexer from the [`lazy_json`]
//! crate.
//!
//! [`lazy_json`]: https://github.com/mnmun/lazy_json

use memchr::{
    Memchr, memchr_iter,
    memmem::{FindIter, Finder},
};

use std::rc::Rc;

/// # Needle abstraction for the [`memchr`] crate
///
/// ![](https://github.com/mnmun/images/blob/main/needle.png?raw=true)
///
/// Represents a search pattern in one of the following forms:
///
/// - `One` - a single byte;
/// - `Finder` - a multi-byte sequence.
///
/// ---
///
/// See the [`module`] documentation for more information.
///
/// [`module`]: crate::utils
pub enum Needle<'a> {
    /// A single-byte pattern
    One(u8),
    /// A multi-byte sequence pattern
    Finder(Rc<Finder<'a>>),
}

impl<'a> Needle<'a> {
    /// # Creates an iterator over `needle` matches in `haystack`
    fn iter(&'a self, haystack: &'a [u8]) -> Iter<'a> {
        match self {
            Needle::One(byte) => Iter::One(memchr_iter(*byte, haystack)),
            Needle::Finder(finder) => {
                Iter::Finder(Box::new(finder.find_iter(haystack)))
            }
        }
    }
}

/// # [`Needle`] iterator
///
/// ![](https://github.com/mnmun/images/blob/main/pedestrian.png?raw=true)
///
/// Provides a unified `iterator` interface over both single-byte
/// ([`Memchr`]) and multi-byte ([`FindIter`]) match iterators.
///
/// ---
///
/// See the [`module`] documentation for more information.
///
/// [`module`]: crate::utils
enum Iter<'a> {
    /// Iterator over single-byte matches
    One(Memchr<'a>),
    /// Iterator over multi-byte sequence matches
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

/// # Count occurrences of `needle` in `haystack`
///
/// Implemented on top of the [`memchr`] crate.
///
/// # Example
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use a_bc::utils::{Needle, count_needles};
///
/// let source = b"some kind of source";
/// // Count 'o'    ^        ^   ^
///
/// assert_eq!(count_needles(&Needle::One(b'o'), source), 3);
/// ```
///
/// ---
///
/// See the [`module`] documentation for more information.
///
/// [`module`]: crate::utils
pub fn count_needles<'a>(needle: &Needle<'a>, haystack: &[u8]) -> usize {
    needle.iter(haystack).count()
}

/// # Count occurrences of `needle` in `haystack`, ignoring `escaped` matches
///
/// A match is counted only when the number of consecutive `escape` bytes
/// immediately before it is even; an odd count means the match itself is
/// escaped and skipped. For example, with `br"\"` as an `escape`, in `haystack`
/// `a\\,b` counts the comma, while `a\,b` does not (backslash escapes the
/// comma).
///
/// Implemented on top of the [`memchr`] crate.
///
/// # Example
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use a_bc::utils::{Needle, count_needles_considering_escapes};
///
/// let source = br"s\ome kind of s\\ource";
/// // Count 'o'      X        ^     ^
///
/// assert_eq!(
///     count_needles_considering_escapes(
///         &Needle::One(b'o'),
///         br"\",
///         source
///     ),
///     2
/// );
/// ```
///
/// ---
///
/// See the [`module`] documentation for more information.
///
/// [`module`]: crate::utils
pub fn count_needles_considering_escapes<'a>(
    needle: &Needle<'a>,
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

/// # Count occurrences of `needle` in `haystack`, ignoring `delimited` matches
///
/// A `delimiter` toggles the "inside a delimited region" state (e.g. a region
/// delimited by `"`); matches found while inside are not counted. The running
/// `delimiter_counter` is updated in place and must be kept across calls so a
/// region opened in one chunk is still recognized in the next.
///
/// Implemented on top of the [`memchr`] crate.
///
/// # Example
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use a_bc::utils::{
///     Needle,
///     count_needles_considering_delimiters
/// };
///
/// let source = br#"some "kind of" "source""#;
/// // Count 'o'      ^         X     X
/// let mut delimiter_count = 0;
///
/// assert_eq!(
///     count_needles_considering_delimiters(
///         &Needle::One(b'o'),
///         &Needle::One(b'"'),
///         &mut delimiter_count,
///         source
///     ),
///     1
/// );
/// assert_eq!(delimiter_count, 4);
/// ```
///
/// ---
///
/// See the [`module`] documentation for more information.
///
/// [`module`]: crate::utils
pub fn count_needles_considering_delimiters<'a>(
    needle: &Needle<'a>,
    delimiter: &Needle<'a>,
    delimiter_counter: &mut usize,
    mut haystack: &[u8],
) -> usize {
    let mut count = 0;

    loop {
        let mut it = needle.iter(haystack);

        if let Some(position) = it.next() {
            *delimiter_counter +=
                count_needles(delimiter, &haystack[..position]);

            if delimiter_counter.is_multiple_of(2) {
                count += 1;
            }

            if position + 1 >= haystack.len() {
                break;
            } else {
                haystack = &haystack[position + 1..];
            }
        } else {
            *delimiter_counter += count_needles(delimiter, haystack);

            break;
        }
    }

    count
}

/// # Count occurrences of `needle` in `haystack`, considering `escaped` `delimiters`
///
/// Combines [`count_needles_considering_delimiters()`] with
/// [`count_needles_considering_escapes()`]: delimiters themselves can be
/// escaped, and only unescaped ones toggle the "inside a delimited region"
/// state. `delimiter_counter` is updated in place and must be kept across
/// calls.
///
/// Implemented on top of the [`memchr`] crate.
///
/// # Example
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use a_bc::utils::{
///     Needle,
///     count_needles_considering_escaped_delimiters
/// };
///
/// let source = br#"some \"kind of\" "source""#;
/// // Count 'o'      ^          ^      X
/// let mut delimiter_count = 0;
///
/// assert_eq!(
///     count_needles_considering_escaped_delimiters(
///         &Needle::One(b'o'),
///         br"\",
///         &Needle::One(b'"'),
///         &mut delimiter_count,
///         source
///     ),
///     2
/// );
/// assert_eq!(delimiter_count, 2);
/// ```
///
/// ---
///
/// See the [`module`] documentation for more information.
///
/// [`module`]: crate::utils
pub fn count_needles_considering_escaped_delimiters<'a>(
    needle: &Needle<'a>,
    delimiter_escape: &'a [u8],
    delimiter: &Needle<'a>,
    delimiter_counter: &mut usize,
    mut haystack: &[u8],
) -> usize {
    let mut count = 0;

    loop {
        let mut it = needle.iter(haystack);

        if let Some(position) = it.next() {
            *delimiter_counter += count_needles_considering_escapes(
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
            *delimiter_counter += count_needles_considering_escapes(
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
        Needle, count_needles_considering_delimiters,
        count_needles_considering_escaped_delimiters,
        count_needles_considering_escapes,
    };

    #[test]
    fn count_needles_considering_backslashes_no_matches() {
        let haystack = b"hello";

        assert_eq!(
            count_needles_considering_escapes(
                &Needle::One(b'x'),
                br"\",
                haystack
            ),
            0
        );

        let finder = Rc::new(Finder::new(b"x"));
        assert_eq!(
            count_needles_considering_escapes(
                &Needle::Finder(finder),
                br"\",
                haystack
            ),
            0
        );
    }

    #[test]
    fn count_needles_considering_backslashes_no_escape() {
        let haystack = b"a,b,c";

        assert_eq!(
            count_needles_considering_escapes(
                &Needle::One(b','),
                br"\",
                haystack
            ),
            2
        );

        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_escapes(
                &Needle::Finder(finder),
                br"\",
                haystack
            ),
            2
        );
    }

    #[test]
    fn count_needles_considering_backslashes_single_escape() {
        let haystack = br"a,b\,c";

        assert_eq!(
            count_needles_considering_escapes(
                &Needle::One(b','),
                br"\",
                haystack
            ),
            1
        );

        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_escapes(
                &Needle::Finder(finder),
                br"\",
                haystack
            ),
            1
        );
    }

    #[test]
    fn count_needles_considering_backslashes_double_escape() {
        let haystack = br"a,b\\,c";

        assert_eq!(
            count_needles_considering_escapes(
                &Needle::One(b','),
                br"\",
                haystack
            ),
            2
        );

        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_escapes(
                &Needle::Finder(finder),
                br"\",
                haystack
            ),
            2
        );
    }

    #[test]
    fn count_needles_considering_backslashes_mixed() {
        let haystack = br"x,\,x,,\\,";

        assert_eq!(
            count_needles_considering_escapes(
                &Needle::One(b','),
                br"\",
                haystack
            ),
            4
        );

        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_escapes(
                &Needle::Finder(finder),
                br"\",
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
            count_needles_considering_escapes(
                &Needle::One(b','),
                br"\",
                &haystack
            ),
            500
        );

        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_escapes(
                &Needle::Finder(finder),
                br"\",
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
            count_needles_considering_delimiters(
                &Needle::One(b','),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            0
        );
        assert_eq!(quotes, 0);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::Finder(finder),
                &Needle::One(b'"'),
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
            count_needles_considering_delimiters(
                &Needle::One(b','),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            0
        );
        assert_eq!(quotes, 0);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::Finder(finder),
                &Needle::One(b'"'),
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
            count_needles_considering_delimiters(
                &Needle::One(b','),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            2
        );
        assert_eq!(quotes, 0);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::Finder(finder),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            2
        );
        assert_eq!(quotes, 0);
    }

    #[test]
    fn count_needles_considering_quotes_inside_quotes() {
        let haystack = br#"  "a,b,c"  "#;

        let mut quotes = 0;
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::One(b','),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            0
        );
        assert_eq!(quotes, 2);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::Finder(finder),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            0
        );
        assert_eq!(quotes, 2);
    }

    #[test]
    fn count_needles_considering_quotes_mixed() {
        let haystack = br#"a,"b,c",d,e"#;

        let mut quotes = 0;
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::One(b','),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            3
        );
        assert_eq!(quotes, 2);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::Finder(finder),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            3
        );
        assert_eq!(quotes, 2);
    }

    #[test]
    fn count_needles_considering_quotes_escaped_quotes() {
        let haystack = br#"a,"b\",c"\",d"#;

        let mut quotes = 0;
        assert_eq!(
            count_needles_considering_escaped_delimiters(
                &Needle::One(b','),
                br"\",
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            2
        );
        assert_eq!(quotes, 2);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_escaped_delimiters(
                &Needle::Finder(finder),
                br"\",
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            2
        );
        assert_eq!(quotes, 2);
    }

    #[test]
    fn count_needles_considering_quotes_complex() {
        let haystack = br#"a,b,"c,d",e,"f",g,h"#;

        let mut quotes = 0;
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::One(b','),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            6
        );
        assert_eq!(quotes, 4);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::Finder(finder),
                &Needle::One(b'"'),
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
            count_needles_considering_delimiters(
                &Needle::One(b','),
                &Needle::One(b'"'),
                &mut quotes,
                &haystack,
            ),
            50
        );
        assert_eq!(quotes, 100);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::Finder(finder),
                &Needle::One(b'"'),
                &mut quotes,
                &haystack,
            ),
            50
        );
        assert_eq!(quotes, 100);
    }

    #[test]
    fn count_needles_considering_quotes_custom_chunk_size() {
        let haystack = br#"a,b,"c,d",e,f"#;

        let mut quotes = 0;
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::One(b','),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            4
        );
        assert_eq!(quotes, 2);

        let mut quotes = 0;
        let finder = Rc::new(Finder::new(b","));
        assert_eq!(
            count_needles_considering_delimiters(
                &Needle::Finder(finder),
                &Needle::One(b'"'),
                &mut quotes,
                haystack,
            ),
            4
        );
        assert_eq!(quotes, 2);
    }
}
