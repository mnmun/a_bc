//! # Lexer errors
//!
//! Types for error handling in your lexers.
//!
//! See [`crate`] for more information.

use std::{error, fmt};

use crate::traits::KindBounds;

/// # Unified error enum
///
/// Aggregates all errors a lexer can produce:
/// - [Basic errors](basic::Error) - not tied to token creation
/// - [Token errors](token::Error) - raised while creating a new token
#[derive(PartialEq, Clone, Debug)]
pub enum Error<Kind>
where
    Kind: KindBounds,
{
    /// An error unrelated to token creation (empty source, cancellation)
    Basic(basic::Error),

    /// An error raised while creating a new token
    Token(token::Error<Kind>),
}

impl<Kind: KindBounds> From<basic::Error> for Error<Kind> {
    fn from(value: basic::Error) -> Self {
        Self::Basic(value)
    }
}

impl<Kind: KindBounds> From<token::Error<Kind>> for Error<Kind> {
    fn from(value: token::Error<Kind>) -> Self {
        Self::Token(value)
    }
}

impl<Kind: KindBounds> fmt::Display for Error<Kind> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Basic(basic) => write!(f, "Basic error: {basic}"),
            Error::Token(token) => write!(f, "Token error: {token}"),
        }
    }
}

impl<Kind: KindBounds + 'static> error::Error for Error<Kind> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Basic(basic) => Some(basic),
            Error::Token(token) => Some(token),
        }
    }
}

/// # Basic lexer errors
pub mod basic {
    use std::{error, fmt};

    /// Errors unrelated to token creation
    #[derive(PartialEq, Clone, Debug)]
    pub enum Error {
        /// The input source is empty
        SourceIsEmpty,

        /// Lexing was manually cancelled
        Cancelled,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Error::SourceIsEmpty => write!(f, "Provided source is empty"),
                Error::Cancelled => {
                    write!(f, "Lexing was cancelled")
                }
            }
        }
    }

    impl error::Error for Error {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Error::SourceIsEmpty => None,
                Error::Cancelled => None,
            }
        }
    }
}

/// # Lexer token errors
pub mod token {
    use std::{error, fmt};

    use crate::traits::KindBounds;

    /// Errors raised while creating a new token
    #[derive(PartialEq, Clone, Debug)]
    pub enum Error<Kind: KindBounds> {
        /// A closing delimiter could not be found
        PairNotFound {
            /// The opening delimiter (e.g. '{')
            opening: Box<str>,
            /// The closing delimiter (e.g. '}')
            closing: Box<str>,
            /// Row-column position of the opening delimiter in the source
            after: (usize, usize),
        },

        /// An unexpected token kind was encountered
        ExpectedButGot {
            /// The token kinds that were expected
            expected: Box<[Kind]>,
            /// The token kind that was actually found
            got: Kind,
            /// Row-column position of the offending token in the source
            after: (usize, usize),
        },

        /// A token kind that was explicitly disallowed was encountered
        NotExpectedButGot {
            /// The token kinds that were not expected
            not_expected: Box<[Kind]>,
            /// The token kind that was actually found
            got: Kind,
            /// Row-column position of the offending token in the source
            after: (usize, usize),
        },
    }

    impl<Kind: KindBounds> fmt::Display for Error<Kind> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Error::PairNotFound {
                    opening,
                    closing,
                    after,
                } => {
                    let (row, col) = after;
                    write!(
                        f,
                        "Could not find '{closing}' for '{opening}' after [{row}:{col}]"
                    )
                }
                Error::ExpectedButGot {
                    expected,
                    got,
                    after,
                } => {
                    let (row, col) = after;
                    write!(
                        f,
                        "Expected one of these tokens: {expected:?} after [{row}:{col}], but got {got:?}"
                    )
                }
                Error::NotExpectedButGot {
                    not_expected,
                    got,
                    after,
                } => {
                    let (row, col) = after;
                    write!(
                        f,
                        "Not expected these tokens: {not_expected:?} after [{row}:{col}], but got {got:?}"
                    )
                }
            }
        }
    }

    impl<Kind: KindBounds> error::Error for Error<Kind> {
        fn source(&self) -> Option<&(dyn error::Error + 'static)> {
            match self {
                Error::PairNotFound {
                    opening: _,
                    closing: _,
                    after: _,
                } => None,
                Error::ExpectedButGot {
                    expected: _,
                    got: _,
                    after: _,
                } => None,
                Error::NotExpectedButGot {
                    not_expected: _,
                    got: _,
                    after: _,
                } => None,
            }
        }
    }
}
