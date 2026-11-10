//! # Lexing essentials
//!
//! ![](https://github.com/mnmun/images/blob/main/batteries.png?raw=true)
//!
//! Provides the essentials for the custom `lexers`:
//!
//! - [`Lexer`] - the scanning engine that iterates the [`source`] and produces
//!   [`tokens`];
//! - [`Data`] - holds the the [`source`] bytes and the active scan [`range`]
//!   within which the [`lexer`] operates;
//! - [`Cursor`] - records the currently observed [`byte`] and its [`position`]
//!   in the [`source`]'
//! - [`Builder`] - an instrument for the configuration and creation of a
//!   [`lexer`].
//!
//! ---
//!
//! See the [`crate`] documentation for more information.
//!
//! [`source`]: crate::lexer::Data::source()
//! [`tokens`]: crate::Token
//! [`range`]: crate::lexer::Data::range()
//! [`lexer`]: crate::Lexer
//! [`byte`]: crate::lexer::Cursor::byte()
//! [`position`]: crate::lexer::Cursor::position()
//! [`crate`]: crate

use core::fmt::Debug;
use std::{borrow::Cow, ops::Range};

use getset::{Getters, MutGetters};

use crate::{Flag, error};

/// # [`Lexer`] `data`
///
/// ![](https://github.com/mnmun/images/blob/main/book.png?raw=true)
///
/// Holds the `data` processed by the [`lexer`] and stores the following fields:
///
/// - [`source`] - a collection of `u8` values that the [`lexer`] iterates over
///   in order to produce [`tokens`];
/// - [`range`] - the active scan `range` within [`source`] in which the [`lexer`]
///   operates.
///
/// ## Creation
///
/// Manual creation is not required; the structure is created automatically
/// when the [`Builder::build()`] method is called.
///
/// ---
///
/// See the [`lexer`] documentation for more information.
///
/// [`lexer`]: crate::Lexer
/// [`source`]: crate::lexer::Data::source()
/// [`tokens`]: crate::Token
/// [`range`]: crate::lexer::Data::range()
/// [`Builder::build()`]: crate::lexer::Builder::build()
#[derive(Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Data<'source> {
    /// # The `source` bytes provided as input
    ///
    /// See the [`data`] documentation for more information.
    ///
    /// [`data`]: crate::lexer::Data
    source: Cow<'source, [u8]>,

    /// # The active scan `range` within [`source`]
    ///
    /// See the [`data`] documentation for more information.
    ///
    /// [`source`]: crate::lexer::Data::source()
    /// [`data`]: crate::lexer::Data
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
/// - [`position`] - the position of currently observed [`byte`] in the
///   [`source`].
///
/// ## Creation
///
/// Manual creation is not required; the structure is created automatically
/// when the [`Builder::build()`] method is called.
///
/// ---
///
/// See the [`lexer`] documentation for more information.
///
/// [`source`]: crate::lexer::Data::source()
/// [`byte`]: crate::lexer::Cursor::byte()
/// [`position`]: crate::lexer::Cursor::position()
/// [`lexer`]: crate::Lexer
/// [`Builder::build()`]: crate::lexer::Builder::build()
#[derive(Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Cursor {
    /// # The `byte` currently observed in the [`source`]
    ///
    /// See the [`cursor`] documentation for more information.
    ///
    /// [`source`]: crate::lexer::Data::source()
    /// [`cursor`]: crate::lexer::Cursor
    byte: Option<u8>,

    /// # The `position` of the currently observed [`byte`] in the [`source`]
    ///
    /// See the [`cursor`] documentation for more information.
    ///
    /// [`byte`]: crate::lexer::Cursor::byte()
    /// [`source`]: crate::lexer::Data::source()
    /// [`cursor`]: crate::lexer::Cursor
    position: usize,
}

/// # [`Lexer`] `builder`
///
/// ![](https://github.com/mnmun/images/blob/main/road_work.png?raw=true)
///
/// Holds the data used for the [`lexer`] creation.
///
/// ## Creation
///
/// Can be created using the [`Builder::new()`] method:
///
/// ```rust
/// use a_bc::{Lexer, lexer::Builder};
///
/// let source: &'static [u8] = b"some source";
///
/// let lexer: Lexer = Builder::new(source)
///     .with_range(1..5)
///     .with_position(3)
///     .build()
///     .unwrap();
/// ```
///
/// ### Mandatory attributes:
///
/// - [`source`] - the collection of `u8` values to be iterated over and used
///   in [`token`] creation.
///
/// ### Optional attributes:
///
/// - [`range`] - the active scan `range` within [`source`]; if not specified,
///   the full extent of [`source`] is used;
/// - [`position`] - the initial `position` of the [`cursor`]; if not specified,
///   defaults to the start of the active scan [`range`];
/// - [`flag`] - the cancellation [`flag`] that can be used during the token
///   creation process for an emergency stop; if not specified, the separate
///   [`flag`] for the [`lexer`] will be created automatically during the
///   [`build()`] call.
///
/// ## Build
///
/// After setting all the necessary fields, call the [`build()`] method to get
/// a [`lexer`] instance.
///
/// ---
///
/// See the [`lexer`] documentation for more information.
///
/// [`lexer`]: crate::Lexer
/// [`source`]: crate::lexer::Data::source()
/// [`token`]: crate::Token
/// [`range`]: crate::lexer::Data::range()
/// [`position`]: crate::lexer::Cursor::position()
/// [`flag`]: crate::Lexer::flag()
/// [`cursor`]: crate::lexer::Cursor
/// [`build()`]: crate::lexer::Builder::build()
#[derive(Debug, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Builder<'source> {
    /// # The `source` bytes used as input
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`builder`]: crate::lexer::Builder
    source: Cow<'source, [u8]>,

    /// # Optional active `range`
    ///
    /// If not specified, the full extent of [`source`] is used.
    ///
    /// ---
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`source`]: crate::lexer::Builder::source()
    /// [`builder`]: crate::lexer::Builder
    range: Option<Range<usize>>,

    /// # Optional [`cursor`] starting `position`
    ///
    /// If not specified, defaults to the start of the active scan [`range`].
    ///
    /// ---
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`cursor`]: crate::lexer::Cursor
    /// [`range`]: crate::lexer::Builder::range()
    /// [`builder`]: crate::lexer::Builder
    position: Option<usize>,

    /// # Optional cancellation [`flag`]
    ///
    /// If not specified, the separate [`flag`] for the [`lexer`] will be
    /// created automatically during the [`Builder::build()`] call.
    ///
    /// ---
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`cancellation flag`]: crate::Flag
    /// [`flag`]: crate::Flag
    /// [`builder`]: crate::lexer::Builder
    /// [`lexer`]: crate::Lexer
    /// [`Builder::build()`]: crate::lexer::Builder::build()
    flag: Option<Flag>,
}

impl Builder<'_> {
    /// # Optional active `range`
    ///
    /// If not specified, the full extent of [`source`] is used.
    ///
    /// ---
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`source`]: crate::lexer::Builder::source()
    /// [`builder`]: crate::lexer::Builder
    pub fn set_range(
        &mut self,
        value: impl Into<Option<Range<usize>>>,
    ) -> &mut Self {
        self.range = value.into();
        self
    }

    /// # Optional [`cursor`] starting `position`
    ///
    /// If not specified, defaults to the start of the active scan [`range`].
    ///
    /// ---
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`cursor`]: crate::lexer::Cursor
    /// [`range`]: crate::lexer::Builder::range()
    /// [`builder`]: crate::lexer::Builder
    pub fn set_position(
        &mut self,
        value: impl Into<Option<usize>>,
    ) -> &mut Self {
        self.position = value.into();
        self
    }

    /// # Optional cancellation [`flag`]
    ///
    /// If not specified, the separate [`flag`] for the [`lexer`] will be
    /// created automatically during the [`Builder::build()`] call.
    ///
    /// ---
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`cancellation flag`]: crate::Flag
    /// [`flag`]: crate::Flag
    /// [`builder`]: crate::lexer::Builder
    /// [`lexer`]: crate::Lexer
    /// [`Builder::build()`]: crate::lexer::Builder::build()
    pub fn set_flag(&mut self, value: impl Into<Option<Flag>>) -> &mut Self {
        self.flag = value.into();
        self
    }
}

impl Builder<'_> {
    /// # Optional active `range`
    ///
    /// If not specified, the full extent of [`source`] is used.
    ///
    /// ---
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`source`]: crate::lexer::Builder::source()
    /// [`builder`]: crate::lexer::Builder
    #[must_use = "method returns the modified value"]
    pub fn with_range(
        mut self,
        value: impl Into<Option<Range<usize>>>,
    ) -> Self {
        self.range = value.into();
        self
    }

    /// # Optional [`cursor`] starting `position`
    ///
    /// If not specified, defaults to the start of the active scan [`range`].
    ///
    /// ---
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`cursor`]: crate::lexer::Cursor
    /// [`range`]: crate::lexer::Builder::range()
    /// [`builder`]: crate::lexer::Builder
    #[must_use = "method returns the modified value"]
    pub fn with_position(mut self, value: impl Into<Option<usize>>) -> Self {
        self.position = value.into();
        self
    }

    /// # Optional cancellation [`flag`]
    ///
    /// If not specified, the separate [`flag`] for the [`lexer`] will be
    /// created automatically during the [`Builder::build()`] call.
    ///
    /// ---
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`cancellation flag`]: crate::Flag
    /// [`flag`]: crate::Flag
    /// [`builder`]: crate::lexer::Builder
    /// [`lexer`]: crate::Lexer
    /// [`Builder::build()`]: crate::lexer::Builder::build()
    #[must_use = "method returns the modified value"]
    pub fn with_flag(mut self, value: impl Into<Option<Flag>>) -> Self {
        self.flag = value.into();
        self
    }
}

impl<'source> Builder<'source> {
    /// # Creates a new [`builder`] over `source`
    ///
    /// The [`range`], [`position`] and [`flag`] parameters are set to `None`
    /// by default.
    ///
    /// ---
    ///
    /// See the [`builder`] documentation for more information.
    ///
    /// [`builder`]: crate::lexer::Builder
    /// [`range`]: crate::lexer::Builder::range()
    /// [`position`]: crate::lexer::Builder::position()
    /// [`flag`]: crate::lexer::Builder::flag()
    pub fn new(source: impl Into<Cow<'source, [u8]>>) -> Self {
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
    /// See the [`builder`] documentation for more information.
    ///
    /// [`lexer`]: crate::Lexer
    /// [`range`]: crate::lexer::Data::range()
    /// [`cursor`]: crate::lexer::Cursor
    /// [`position`]: crate::lexer::Cursor::position()
    /// [`byte`]: crate::lexer::Cursor::byte()
    /// [`builder`]: crate::lexer::Builder
    pub fn build(self) -> Result<Lexer<'source>, error::Lexer> {
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
/// The scanning engine for the custom `lexers` that exposes the following
/// methods:
///
/// - [`read_next_byte()`] - advances the [`cursor`] by one byte;
/// - [`peek_next_byte()`] - returns the next [`byte`] without advancing the
///   [`cursor`];
/// - [`skip_whitespace()`] - skips over ASCII [whitespace bytes].
///
/// All operations listed above are constrained to the active scan [`range`].
///
/// ## Creation
///
/// Can be created using the [`builder`]:
///
/// ```rust
/// use a_bc::{Lexer, lexer::Builder};
///
/// let source: &'static [u8] = b"some source";
///
/// let lexer: Lexer = Builder::new(source)
///     .with_range(1..5)
///     .with_position(3)
///     .build()
///     .unwrap();
/// ```
///
/// ## State
///
/// Use the following methods to inspect a `lexer` state:
///
/// - [`data()`] - the [`source`] bytes and the active scan [`range`];
/// - [`cursor()`] - the currently observed [`byte`] and its [`position`] within
///   the [`source`];
/// - [`flag()`] - a cloneable thread-safe cancellation [`flag`].
///
/// ---
///
/// See the [`module`] documentation for more information.
///
/// [`read_next_byte()`]: crate::Lexer::read_next_byte()
/// [`peek_next_byte()`]: crate::Lexer::peek_next_byte()
/// [`skip_whitespace()`]: crate::Lexer::skip_whitespace()
/// [whitespace bytes]: u8::is_ascii_whitespace
/// [`byte`]: crate::lexer::Cursor::byte()
/// [`cursor`]: crate::lexer::Cursor
/// [`range`]: crate::lexer::Data::range()
/// [`data()`]: crate::Lexer::data()
/// [`cursor()`]: crate::Lexer::cursor()
/// [`flag()`]: crate::Lexer::flag()
/// [`flag`]: crate::Flag
/// [`position`]: crate::lexer::Cursor::position()
/// [`source`]: crate::lexer::Data::source()
/// [`builder`]: crate::lexer::Builder
/// [`module`]: crate::lexer
#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct Lexer<'source> {
    /// # The [`source`] bytes and the active scan [`range`]
    ///
    /// See the [`lexer`] documentation for more information.
    ///
    /// [`lexer`]: crate::Lexer
    /// [`source`]: crate::lexer::Data::source()
    /// [`range`]: crate::lexer::Data::range()
    data: Data<'source>,

    /// # The currently observed [`byte`] and its [`position`] within the [`source`]
    ///
    /// See the [`lexer`] documentation for more information.
    ///
    /// [`lexer`]: crate::Lexer
    /// [`byte`]: crate::lexer::Cursor::byte()
    /// [`position`]: crate::lexer::Cursor::position()
    /// [`source`]: crate::lexer::Data::source()
    cursor: Cursor,

    /// # The shared cancellation [`flag`]
    ///
    /// See the [`lexer`] documentation for more information.
    ///
    /// [`lexer`]: crate::Lexer
    /// [`flag`]: crate::Flag
    /// [`token`]: crate::Token
    flag: Flag,
}

impl PartialEq for Lexer<'_> {
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

impl Lexer<'_> {
    /// # Moves the [`cursor`] to the specified `position` and updates the currently observed [`byte`]
    ///
    /// See the [`lexer`] documentation for more information.
    ///
    /// [`lexer`]: crate::Lexer
    /// [`cursor`]: crate::lexer::Cursor
    /// [`byte`]: crate::lexer::Cursor::byte()
    pub fn set_position(&mut self, value: usize) -> &mut Self {
        self.cursor.position = value;
        self.cursor.byte = self.data.source.get(value).cloned();
        self
    }
}

impl Lexer<'_> {
    /// # Advances the [`cursor`] by one byte and returns the observed [`byte`]
    ///
    /// See the [`lexer`] documentation for more information.
    ///
    /// [`lexer`]: crate::Lexer
    /// [`cursor`]: crate::lexer::Cursor
    /// [`byte`]: crate::lexer::Cursor::byte()
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
    /// See the [`lexer`] documentation for more information.
    ///
    /// [`lexer`]: crate::Lexer
    /// [`byte`]: crate::lexer::Cursor::byte()
    /// [`cursor`]: crate::lexer::Cursor
    pub fn peek_next_byte(&mut self) -> Option<u8> {
        let next_position = self.cursor.position + 1;

        if next_position >= self.data.range.end {
            None
        } else {
            self.data.source.get(next_position).cloned()
        }
    }

    /// # Moves the [`cursor`] past any ASCII [whitespace bytes]
    ///
    /// See the [`lexer`] documentation for more information.
    ///
    /// [`lexer`]: crate::Lexer
    /// [`cursor`]: crate::lexer::Cursor
    /// [whitespace bytes]: u8::is_ascii_whitespace
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
