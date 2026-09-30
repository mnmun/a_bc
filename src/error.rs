//! # Error handling
//!
//! ![](https://github.com/mnmun/images/blob/main/question.png?raw=true)
//!
//! Provides the following error types:
//!
//! - [`Error`] - the unified error type that aggregates [`lexer`] and [`token`]
//!   errors;
//! - [`Lexer`] - lexer related errors;
//! - [`Token`] - errors raised while creating a new token.
//!
//! In addition, provides the helper types and function used to describe the
//! location of a [`token error`] within the [`source`] data:
//!
//! - [`Position`] - 1-indexed row-column coordinates representing a position
//!   in the [`lexer's`] [`source`] data, consistent with the convention
//!   used for text files (`[1:1]` - the first character on the first line);
//! - [`Relation`] - describes the spatial relation of an `error` location to
//!   a [`position`] within the [`source`];
//! - [`row_col_pos()`] - returns the [`position`] of the last byte in the
//!   provided source.
//!
//! ---
//!
//! See the [`crate documentation`] for usage example.
//!
//! [`lexer`]: Lexer
//! [`Token`]: Token
//! [`position`]: Position
//! [`token error`]: Token
//! [`lexer's`]: crate::Lexer
//! [`source`]: crate::lexer::Data::source()
//! [`crate documentation`]: crate

use std::{error, fmt};

use getset::Getters;
use memchr::memchr_iter;

use crate::traits::KindBounds;

/// # Unified error type
///
/// ![](https://github.com/mnmun/images/blob/main/question.png?raw=true)
///
/// Aggregates [`lexer`] and [`token`] errors.
///
/// ---
///
/// See the [`module documentation`] for more information.
///
/// [`lexer`]: Lexer
/// [`token`]: Token
/// [`module documentation`]: crate::error
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Error<Kind>
where
    Kind: KindBounds,
{
    /// Lexer related error
    Lexer(Lexer),

    /// An error raised while creating a new token
    Token(Token<Kind>),
}

impl<Kind: KindBounds> From<Lexer> for Error<Kind> {
    fn from(value: Lexer) -> Self {
        Self::Lexer(value)
    }
}

impl<Kind: KindBounds> From<Token<Kind>> for Error<Kind> {
    fn from(value: Token<Kind>) -> Self {
        Self::Token(value)
    }
}

impl<Kind: KindBounds> fmt::Display for Error<Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Lexer(lexer) => write!(f, "Lexer error: {lexer}"),
            Error::Token(token) => write!(f, "Token error: {token}"),
        }
    }
}

impl<Kind: KindBounds + 'static> error::Error for Error<Kind> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Lexer(lexer) => Some(lexer),
            Error::Token(token) => Some(token),
        }
    }
}

/// # [`Lexer`] related errors
///
/// ![](https://github.com/mnmun/images/blob/main/question.png?raw=true)
///
/// See the [`module documentation`] for more information.
///
/// [`Lexer`]: crate::Lexer
/// [`module documentation`]: crate::error
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Lexer {
    /// The [`source`] data is empty
    ///
    /// [`source`]: crate::lexer::Data::source()
    SourceIsEmpty,

    /// Lexing was manually cancelled
    Cancelled,
}

impl fmt::Display for Lexer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Lexer::SourceIsEmpty => write!(f, "The source data is empty"),
            Lexer::Cancelled => {
                write!(f, "Lexing was manually cancelled")
            }
        }
    }
}

impl error::Error for Lexer {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Lexer::SourceIsEmpty => None,
            Lexer::Cancelled => None,
        }
    }
}

/// # Errors raised while creating a new [`token`]
///
/// ![](https://github.com/mnmun/images/blob/main/question.png?raw=true)
///
/// See the [`module documentation`] for more information.
///
/// [`token`]: crate::Token
/// [`module documentation`]: crate::error
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum Token<Kind: KindBounds> {
    /// A closing delimiter could not be found
    PairNotFound {
        /// The opening delimiter (e.g. '{')
        opening: Box<str>,
        /// The closing delimiter (e.g. '}')
        closing: Box<str>,
        /// The expected location of the missing closing delimiter,
        /// expressed as a [`relation`] to a [`position`] in the source
        ///
        /// [`relation`]: Relation
        /// [`position`]: Position
        location: (Relation, Position),
    },

    /// An unexpected [`token`] kind was encountered
    ///
    /// [`token`]: crate::Token
    ExpectedButGot {
        /// The [`token`] kinds that were expected
        ///
        /// [`token`]: crate::Token
        expected: Box<[Kind]>,
        /// The [`token`] kind that was actually found
        ///
        /// [`token`]: crate::Token
        got: Option<Kind>,
        /// [`Position`] of the offending [`token`] in the source, paired with the
        /// [`Relation::At`]
        ///
        /// [`token`]: crate::Token
        location: (Relation, Position),
    },

    /// A [`token`] kind that was explicitly disallowed was encountered
    ///
    /// [`token`]: crate::Token
    NotExpectedButGot {
        /// The [`token`] kinds that were not expected
        ///
        /// [`token`]: crate::Token
        not_expected: Box<[Kind]>,
        /// The [`token`] kind that was actually found
        ///
        /// [`token`]: crate::Token
        got: Option<Kind>,
        /// [`Position`] of the offending [`token`] in the source, paired with the
        /// [`Relation::At`]
        ///
        /// [`token`]: crate::Token
        location: (Relation, Position),
    },
}

impl<Kind: KindBounds> fmt::Display for Token<Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::PairNotFound {
                opening: _,
                closing,
                location,
            } => {
                let (relation, position) = location;

                write!(f, "Could not find '{closing}' {relation} {position}")
            }
            Token::ExpectedButGot {
                expected,
                got,
                location,
            } => {
                let (relation, position) = location;
                let expected = format!(
                    "[{}]",
                    expected
                        .into_iter()
                        .map(|k| k.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                write!(
                    f,
                    "Expected one of these tokens: {expected} {relation} {position}, but got "
                )?;

                match *got {
                    Some(kind) => {
                        write!(f, "{kind}")
                    }
                    None => {
                        write!(f, "nothing")
                    }
                }
            }
            Token::NotExpectedButGot {
                not_expected,
                got,
                location,
            } => {
                let (relation, position) = location;
                let not_expected = format!(
                    "[{}]",
                    not_expected
                        .into_iter()
                        .map(|k| k.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                write!(
                    f,
                    "Not expected these tokens: {not_expected} {relation} {position}, but got "
                )?;

                match *got {
                    Some(kind) => {
                        write!(f, "{kind}")
                    }
                    None => {
                        write!(f, "nothing")
                    }
                }
            }
        }
    }
}

impl<Kind: KindBounds> error::Error for Token<Kind> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Token::PairNotFound {
                opening: _,
                closing: _,
                location: _,
            } => None,
            Token::ExpectedButGot {
                expected: _,
                got: _,
                location: _,
            } => None,
            Token::NotExpectedButGot {
                not_expected: _,
                got: _,
                location: _,
            } => None,
        }
    }
}

/// # Row-column position in the text file
///
/// ![](https://github.com/mnmun/images/blob/main/xy.png?raw=true)
///
/// Represents a 1-indexed row-column coordinates within the `source` byte
/// string, consistent with the convention used by most text editors (`[1:1]` -
/// the first character on the first line).
///
/// ---
///
/// See the [`module documentation`] for more information.
///
/// [`module documentation`]: crate::error
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Getters)]
#[getset(get = "pub")]
pub struct Position {
    /// The 1-indexed row number within the `source` data, treated as a text
    /// file
    row: usize,
    /// The 1-indexed column number within the `source` data, treated as a text
    /// file
    col: usize,
}

impl Position {
    /// # Creates a new [`position`] with the given `row` and `col`
    ///
    /// [`position`]: Position
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}:{}]", self.row, self.col)
    }
}

impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Self {
            row: value.0,
            col: value.1,
        }
    }
}

/// # Spatial relation to a [`position`]
///
/// ![](https://github.com/mnmun/images/blob/main/arrows.png?raw=true)
///
/// Describes the spatial relation of an `error` location to a [`position`]
/// within the [`source`] data.
///
/// ---
///
/// See the [`module documentation`] for more information.
///
/// [`position`]: Position
/// [`source`]: crate::lexer::Data::source()
/// [`module documentation`]: crate::error
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Relation {
    Before,
    At,
    After,
}

impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Before => write!(f, "before"),
            Self::At => write!(f, "at"),
            Self::After => write!(f, "after"),
        }
    }
}

/// # [`Position`] of the last byte in the `source`
///
/// Both row and column are 1-indexed, consistent with the convention used by
/// common text editors.
///
/// Is intended to be used in error messages when the position within a source
/// file must be determined.
///
/// # Example
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use a_bc::error::row_col_pos;
///
/// // Four-row source
/// let source =
/// b"some
/// kind
/// of
/// source";
///
/// // The entire source is provided. The returned position describes the last
/// // byte ('e' in "source")
/// let position = row_col_pos(source);
/// assert_eq!(*position.row(), 4); // Fourth row
/// assert_eq!(*position.col(), 6); // Sixth column
///
/// // A slice of the source is provided. The returned position describes the
/// // last byte ('d' in "kind")
/// let position = row_col_pos(&source[..9]);
/// assert_eq!(*position.row(), 2); // Second row
/// assert_eq!(*position.col(), 4); // Fourth column
/// ```
pub fn row_col_pos(source: &[u8]) -> Position {
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

    Position::new(row, col)
}
