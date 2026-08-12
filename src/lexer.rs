//! # Lexer
//!
//! Core building blocks for custom lexers:
//!
//! - [`Data`] - the input source with the active scan range
//! - [`Cursor`] - the current position and the byte being observed
//! - [`Builder`] - configures and builds a [`Lexer`] instance
//! - [`Lexer`] - the main scanning engine
//!
//! See [`crate`] for more information.

use core::fmt::Debug;
use std::{borrow::Cow, ops::Range};

use getset::{Getters, MutGetters};

use crate::{cancel::Cancel, error::basic::Error};

/// # [`Lexer`] data
///
/// Holds the input `source` - either a borrowed `&[u8]` slice or an owned
/// `Vec<u8>` - together with the active `range` that limits this [`Lexer`]'s
/// scanning window.
#[derive(Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Data<'a> {
    /// Input source bytes
    source: Cow<'a, [u8]>,

    /// Active scan range within `source`
    range: Range<usize>,
}

/// # [`Lexer`] cursor
///
/// Stores the byte currently being observed from the [`Data::source()`] and
/// its `position`.
#[derive(Debug, PartialEq, Clone, Getters)]
#[getset(get = "pub")]
pub struct Cursor {
    /// Current position in the [`Data::source()`]
    position: usize,

    /// Byte at the current [`Cursor::position()`]
    byte: Option<u8>,
}

/// # [`Lexer`] builder
///
/// Configures and builds a [`Lexer`] instance over `source` with optionally
/// specified `range` and `position`; see [`Builder::set_range()`] and
/// [`Builder::set_position()`]. When not set, the `range` defaults to the full
/// `source` (`[0..source.len()]`) and the `position` to the `range` start.
///
/// After setting all the necessary fields, call [`Builder::build()`] to get
/// a [`Lexer`] instance.
#[derive(Debug, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Builder<'a> {
    /// Input source for the [`Lexer`]
    source: Cow<'a, [u8]>,

    /// Optional active range; defaults to the full `source`
    /// (`[0..source.len()]`)
    range: Option<Range<usize>>,

    /// Optional starting position; defaults to the `range` start
    position: Option<usize>,
}

impl<'a> Builder<'a> {
    /// Sets the active `range` for the [`Lexer`]; `None` - full `source`
    pub fn set_range(
        &mut self,
        value: impl Into<Option<Range<usize>>>,
    ) -> &mut Self {
        self.range = value.into();
        self
    }

    /// Sets the starting `position` for the [`Lexer`]; `None` - `range` start
    pub fn set_position(
        &mut self,
        value: impl Into<Option<usize>>,
    ) -> &mut Self {
        self.position = value.into();
        self
    }
}

impl<'a> Builder<'a> {
    /// Sets the active `range` for the [`Lexer`]; `None` - full `source`
    #[must_use = "method returns the modified value"]
    pub fn with_range(
        mut self,
        value: impl Into<Option<Range<usize>>>,
    ) -> Self {
        self.range = value.into();
        self
    }

    /// Sets the starting `position` for the [`Lexer`]; `None` - `range` start
    #[must_use = "method returns the modified value"]
    pub fn with_position(mut self, value: impl Into<Option<usize>>) -> Self {
        self.position = value.into();
        self
    }
}

impl<'a> Builder<'a> {
    /// Creates a new [`Builder`] over `source`
    pub fn new(source: impl Into<Cow<'a, [u8]>>) -> Self {
        let source = source.into();
        Self {
            source,
            range: None,
            position: None,
        }
    }

    /// Consumes `self` and builds a fully configured [`Lexer`], moving all
    /// configuration fields into the resulting struct.
    ///
    /// Defaults:
    /// - `range` - the full `source` range (`[0..source.len()]`)
    /// - `position` - the `range` start
    pub fn build(self) -> Result<Lexer<'a>, Error> {
        if self.source.is_empty() {
            return Err(Error::SourceIsEmpty);
        }

        let range = self.range.unwrap_or(0..self.source.len());
        let position = self.position.unwrap_or(range.start);
        let byte = self.source.get(position).cloned();

        Ok(Lexer {
            data: Data {
                source: self.source,
                range,
            },
            cursor: Cursor { position, byte },
            cancel: Cancel::new(),
        })
    }
}

/// # Lexer
///
/// The main scanning engine for custom lexers:
///
/// - [`Lexer::read_next_byte()`] - advance the cursor by one byte
/// - [`Lexer::peek_next_byte()`] - peek the next byte without advancing cursor
/// - [`Lexer::skip_whitespace()`] - skip over ASCII whitespace bytes
///
/// All operations stay within the active [`Data::range()`].
#[derive(Debug, Clone, Getters)]
#[getset(get = "pub")]
pub struct Lexer<'a> {
    /// Source bytes and the active scan range
    data: Data<'a>,

    /// Current position and byte in the source
    cursor: Cursor,

    /// Shared cancellation token
    cancel: Cancel,
}

impl<'a> PartialEq for Lexer<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.data != other.data {
            return false;
        }

        if self.cursor != other.cursor {
            return false;
        }

        if self.cancel.is_cancelled() != other.cancel.is_cancelled() {
            return false;
        }

        true
    }
}

impl<'a> Lexer<'a> {
    /// Moves the `cursor` to `value` and updates the current `byte` accordingly
    pub fn set_position(&mut self, value: usize) -> &mut Self {
        self.cursor.position = value;
        self.cursor.byte = self.data.source.get(value).cloned();
        self
    }
}

impl<'a> Lexer<'a> {
    /// Advances the `cursor` by one byte and returns the new current byte
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

    /// Returns the next byte without advancing, or `None` at the end of the
    /// range
    pub fn peek_next_byte(&mut self) -> Option<u8> {
        let next_position = self.cursor.position + 1;

        if next_position >= self.data.range.end {
            None
        } else {
            self.data.source.get(next_position).cloned()
        }
    }

    /// Moves the cursor past any ASCII whitespace bytes
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
