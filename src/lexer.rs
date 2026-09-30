//! # Lexing essentials
//!
//! ![](https://github.com/mnmun/images/blob/main/batteries.png?raw=true)
//!
//! Provides the essentials for the custom lexers:
//!
//! - [`Lexer`] - the scanning engine that traverses the [`source`] and produces
//!   [`tokens`];
//! - [`Data`] - holds the input data: the [`source bytes`] and the
//!   [`active scan range`] within which the [`lexer`] operates;
//! - [`Cursor`] - records the [`currently observed byte`] and its [`position`]
//!   in the [`source`]'
//! - [`Builder`] - an instrument for the configuration and creation of a
//!   [`lexer`].
//!
//! ---
//!
//! See the [`crate documentation`] for more information.
//!
//! [`source`]: Data::source()
//! [`source bytes`]: Data::source()
//! [`tokens`]: crate::Token
//! [`active scan range`]: Data::range()
//! [`lexer`]: Lexer
//! [`currently observed byte`]: Cursor::byte()
//! [`position`]: Cursor::position()
//! [`crate documentation`]: crate

use core::fmt::Debug;
use std::{borrow::Cow, ops::Range};

use getset::{Getters, MutGetters};

use crate::{Cancel, error};

/// # [`Lexer`] `data`
///
/// ![](https://github.com/mnmun/images/blob/main/book.png?raw=true)
///
/// Holds the data processed by the [`lexer`] and stores the following fields:
///
/// - [`source`] - a collection of `u8` values that the [`lexer`] iterates over
///   in order to produce [`tokens`];
/// - [`range`] - the active scan range within [`source`] in which the [`lexer`]
///   operates.
///
/// Is created automatically during the [`build()`] call.
///
/// ---
///
/// See the [`module documentation`] for more information.
///
/// [`lexer`]: Lexer
/// [`source`]: Data::source()
/// [`tokens`]: crate::Token
/// [`range`]: Data::range()
/// [`build()`]: Builder::build()
/// [`module documentation`]: crate::lexer
#[derive(Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Data<'a> {
    /// # The source bytes provided as input
    source: Cow<'a, [u8]>,

    /// # The active scan range within [`source`]
    ///
    /// [`source`]: Data::source()
    range: Range<usize>,
}

/// # [`Lexer`] `cursor`
///
/// ![](https://github.com/mnmun/images/blob/main/index_left.png?raw=true)
///
/// Represents a `cursor` that moves through the [`source`] and tracks the
/// following state:
///
/// - [`byte`] - the `u8` value currently observed in the [`source`];
/// - [`position`] - the position of [`currently observed byte`] in the
///   [`source`].
///
/// Is created automatically during the [`build()`] call.
///
/// ---
///
/// See the [`module documentation`] for more information.
///
/// [`source`]: Data::source()
/// [`byte`]: Cursor::byte()
/// [`currently observed byte`]: Cursor::byte()
/// [`position`]: Cursor::position()
/// [`lexer`]: Lexer
/// [`build()`]: Builder::build()
/// [`module documentation`]: crate::lexer
#[derive(Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Cursor {
    /// # The byte currently observed in the [`source`]
    ///
    /// [`source`]: Data::source()
    byte: Option<u8>,

    /// # The position of the [`currently observed byte`] in the [`source`]
    ///
    /// [`currently observed byte`]: Cursor::byte()
    /// [`source`]: Data::source()
    position: usize,
}

/// # [`Lexer`] `builder`
///
/// ![](https://github.com/mnmun/images/blob/main/road_work.png?raw=true)
///
/// Holds the data used for the [`lexer`] creation.
///
/// Mandatory attributes:
///
/// - [`source`] - the collection of `u8` values to be iterated over and used
///   in [`token`] creation.
///
/// Optional attributes:
///
/// - [`range`] - the active scan range within [`source`]; if not specified,
///   the full extent of [`source`] is used;
/// - [`position`] - the initial position of the [`cursor`]; if not specified,
///   defaults to the start of the [`active scan range`].
///
/// After setting all the necessary fields, call the [`build()`] method to get
/// a [`lexer`] instance.
///
/// ---
///
/// See the [`module documentation`] for more information.
///
/// [`lexer`]: Lexer
/// [`source`]: Data::source()
/// [`token`]: crate::Token
/// [`range`]: Data::range()
/// [`position`]: Cursor::position()
/// [`cursor`]: Cursor
/// [`active scan range`]: Data::range()
/// [`build()`]: Builder::build()
/// [`module documentation`]: crate::lexer
#[derive(Debug, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Builder<'a> {
    /// # The source bytes used as input
    source: Cow<'a, [u8]>,

    /// # Optional active range
    ///
    /// If not specified, the full extent of [`source`] is used.
    ///
    /// [`source`]: Builder::source()
    range: Option<Range<usize>>,

    /// # Optional starting position
    ///
    /// If not specified, defaults to the start of the [`active scan range`].
    ///
    /// [`active scan range`]: Builder::range()
    position: Option<usize>,

    /// # Optional [`cancellation flag`]
    ///
    /// If not specified the separate [`flag`] will be created automatically.
    ///
    /// [`cancellation flag`]: Cancel
    /// [`flag`]: Cancel
    flag: Option<Cancel>,
}

impl<'a> Builder<'a> {
    pub fn set_range(
        &mut self,
        value: impl Into<Option<Range<usize>>>,
    ) -> &mut Self {
        self.range = value.into();
        self
    }

    pub fn set_position(
        &mut self,
        value: impl Into<Option<usize>>,
    ) -> &mut Self {
        self.position = value.into();
        self
    }

    pub fn set_flag(&mut self, value: impl Into<Option<Cancel>>) -> &mut Self {
        self.flag = value.into();
        self
    }
}

impl<'a> Builder<'a> {
    #[must_use = "method returns the modified value"]
    pub fn with_range(
        mut self,
        value: impl Into<Option<Range<usize>>>,
    ) -> Self {
        self.range = value.into();
        self
    }

    #[must_use = "method returns the modified value"]
    pub fn with_position(mut self, value: impl Into<Option<usize>>) -> Self {
        self.position = value.into();
        self
    }

    #[must_use = "method returns the modified value"]
    pub fn with_flag(mut self, value: impl Into<Option<Cancel>>) -> Self {
        self.flag = value.into();
        self
    }
}

impl<'a> Builder<'a> {
    /// # Creates a new [`builder`] over `source`
    ///
    /// [`builder`]: Builder
    pub fn new(source: impl Into<Cow<'a, [u8]>>) -> Self {
        let source = source.into();
        Self {
            source,
            range: None,
            position: None,
            flag: None,
        }
    }

    /// # Builds a configured [`lexer`]
    ///
    /// Consumes `self` and moves all configuration fields into the resulting
    /// structure.
    ///
    /// If during the configuration an incorrect [`range`] was specified (empty
    /// or the end is less than the start), the [`cursor`] [`position`] will be
    /// moved to the end of the [`range`] and the [`byte`] will be set to
    /// `None`.
    ///
    /// ---
    ///
    /// See the [`builder documentation`] for more information.
    ///
    /// [`lexer`]: Lexer
    /// [`range`]: Data::range()
    /// [`cursor`]: Cursor
    /// [`position`]: Cursor::position()
    /// [`byte`]: Cursor::byte()
    /// [`builder documentation`]: Builder
    pub fn build(self) -> Result<Lexer<'a>, error::Lexer> {
        if self.source.is_empty() {
            return Err(error::Lexer::SourceIsEmpty);
        }

        let range = self.range.unwrap_or(0..self.source.len());

        let mut position = self.position.unwrap_or(range.start);
        let mut byte = self.source.get(position).cloned();

        if range.is_empty() || range.end < range.start {
            position = range.end.max(range.start);
            byte = None;
        }

        let flag = self.flag.unwrap_or_default();

        Ok(Lexer {
            data: Data {
                source: self.source,
                range,
            },
            cursor: Cursor { position, byte },
            flag,
        })
    }
}

/// # `Lexer`
///
/// ![](https://github.com/mnmun/images/blob/main/eye.png?raw=true)
///
/// Represents the scanning engine for the custom lexers and exposes the
/// following scanning methods:
///
/// - [`read_next_byte()`] - advances the [`cursor`] by one byte;
/// - [`peek_next_byte()`] - returns the next [`byte`] without advancing the
///   [`cursor`];
/// - [`skip_whitespace()`] - skips over [`ASCII whitespace bytes`].
///
/// All operations listed above are constrained to the [`active scan range`].
///
/// Use the following methods to inspect current `lexer` state:
///
/// - [`data()`] - the [`source bytes`] and the [`active scan range`];
/// - [`cursor()`] - the [`currently observed byte`] and its [`position`] within
///   the [`source`];
/// - [`flag()`] - a cloneable thread-safe [`cancellation flag`].
///
/// Use [`builder`] to create an instance.
///
/// ---
///
/// See the [`module documentation`] for more information.
///
/// [`read_next_byte()`]: Lexer::read_next_byte()
/// [`peek_next_byte()`]: Lexer::peek_next_byte()
/// [`skip_whitespace()`]: Lexer::skip_whitespace()
/// [`ASCII whitespace bytes`]: u8::is_ascii_whitespace
/// [`byte`]: Cursor::byte()
/// [`cursor`]: Cursor
/// [`active scan range`]: Data::range()
/// [`data()`]: Lexer::data()
/// [`cursor()`]: Lexer::cursor()
/// [`flag()`]: Lexer::flag()
/// [`cancellation flag`]: Cancel
/// [`source bytes`]: Data::source()
/// [`currently observed byte`]: Cursor::byte()
/// [`position`]: Cursor::position()
/// [`source`]: Data::source()
/// [`builder`]: Builder
/// [`module documentation`]: crate::lexer
#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct Lexer<'a> {
    /// The [`source bytes`] and the [`active scan range`]
    ///
    /// [`source bytes`]: Data::source()
    /// [`active scan range`]: Data::range()
    data: Data<'a>,

    /// # The [`currently observed byte`] and its [`position`] within the [`source`]
    ///
    /// [`currently observed byte`]: Cursor::byte()
    /// [`position`]: Cursor::position()
    /// [`source`]: Data::source()
    cursor: Cursor,

    /// # The shared [`cancellation flag`]
    ///
    /// [`cancellation flag`]: Cancel
    /// [`token`]: crate::Token
    flag: Cancel,
}

impl<'a> PartialEq for Lexer<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.data != other.data {
            return false;
        }

        if self.cursor != other.cursor {
            return false;
        }

        if self.flag.is_cancelled() != other.flag.is_cancelled() {
            return false;
        }

        true
    }
}

impl<'a> Lexer<'a> {
    /// # Moves the [`cursor`] to the specified `position` and updates the [`currently observed byte`] accordingly
    ///
    /// [`cursor`]: Cursor
    /// [`currently observed byte`]: Cursor::byte()
    pub fn set_position(&mut self, value: usize) -> &mut Self {
        self.cursor.position = value;
        self.cursor.byte = self.data.source.get(value).cloned();
        self
    }
}

impl<'a> Lexer<'a> {
    /// # Advances the [`cursor`] by one byte and returns the observed [`byte`]
    ///
    /// [`cursor`]: Cursor
    /// [`byte`]: Cursor::byte()
    pub fn read_next_byte(&mut self) -> Option<u8> {
        if self.cursor.byte.is_some() {
            let next_position = self.cursor.position + 1;
            let next_byte = self.data.source.get(next_position).cloned();

            self.cursor = Cursor {
                position: next_position,
                byte: next_byte,
            };
        }

        if self.cursor.position >= self.data.range.end {
            self.cursor.byte = None;
        }

        self.cursor.byte
    }

    /// # Returns the next [`byte`] without advancing the [`cursor`]
    ///
    /// [`byte`]: Cursor::byte()
    /// [`cursor`]: Cursor
    pub fn peek_next_byte(&mut self) -> Option<u8> {
        let next_position = self.cursor.position + 1;

        if next_position >= self.data.range.end {
            None
        } else {
            self.data.source.get(next_position).cloned()
        }
    }

    /// # Moves the [`cursor`] past any [`ASCII whitespace bytes`]
    ///
    /// [`cursor`]: Cursor
    /// [`ASCII whitespace bytes`]: u8::is_ascii_whitespace
    pub fn skip_whitespace(&mut self) {
        while let Some(current_byte) = &self.cursor.byte {
            if current_byte.is_ascii_whitespace() {
                self.read_next_byte();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::lexer::Builder;

    #[test]
    fn builder() {
        let source = "      ".as_bytes();
        let lexer = Builder::new(source).with_range(2..2).build().unwrap();
        assert_eq!(lexer.cursor.position, 2);
        assert_eq!(lexer.cursor.byte, None);

        #[allow(clippy::reversed_empty_ranges)]
        let lexer = Builder::new(source).with_range(5..2).build().unwrap();
        assert_eq!(lexer.cursor.position, 5);
        assert_eq!(lexer.cursor.byte, None);
    }
}
